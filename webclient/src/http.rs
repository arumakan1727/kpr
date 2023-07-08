use std::{sync::Arc, time::Duration};

use ::reqwest::cookie::Jar;
use ::tokio::sync::Mutex;
use ::tokio::time::{Interval, MissedTickBehavior};
use reqwest::header::{HeaderName, HeaderValue};
use serde::Serialize;

pub use ::reqwest::{cookie, redirect};
pub use ::reqwest::{Error, IntoUrl, Request, Response};
pub type UrlGlob = ::glob::Pattern;

#[derive(Clone)]
pub struct Client {
    inner: ::reqwest::Client,
    req_intervals: Vec<(UrlGlob, Arc<Mutex<Interval>>)>,
    pub cookie_jar: Arc<Jar>,
}

pub struct RequestBuilder {
    inner: ::reqwest::RequestBuilder,
    client: Client,
    disable_sleep: bool,
}

macro_rules! emit_request_fn {
    ($method:ident) => {
        pub fn $method(&self, u: impl IntoUrl) -> RequestBuilder {
            RequestBuilder::new(self.inner.$method(u), self.clone())
        }
    };
}

impl Client {
    pub fn new(
        redirection: self::redirect::Policy,
        url_wise_req_interval: impl IntoIterator<Item = (UrlGlob, Duration)>,
    ) -> Self {
        let cookie_jar = Arc::new(Jar::default());
        let req_intervals = url_wise_req_interval
            .into_iter()
            .map(|(pat, dur)| {
                let mut interval = ::tokio::time::interval(dur);
                interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
                (pat, Arc::new(Mutex::new(interval)))
            })
            .collect();
        Self {
            inner: reqwest::Client::builder()
                .cookie_store(true)
                .cookie_provider(cookie_jar.clone())
                .redirect(redirection)
                .gzip(true)
                .build()
                .unwrap(),
            req_intervals,
            cookie_jar,
        }
    }

    emit_request_fn!(get);
    emit_request_fn!(head);
    emit_request_fn!(post);
    emit_request_fn!(patch);
    emit_request_fn!(put);
    emit_request_fn!(delete);

    pub(super) async fn execute_request(
        &self,
        req: Request,
        disable_sleep: bool,
    ) -> Result<Response, Error> {
        let url_str = req.url().as_str();
        if let Some(interval) = self
            .req_intervals
            .iter()
            .find(|(pat, _)| pat.matches(url_str))
            .map(|(_, interval)| interval)
        {
            if disable_sleep {
                interval.lock().await.reset();
            } else {
                interval.lock().await.tick().await;
            }
        }

        self.inner.execute(req).await
    }
}

impl RequestBuilder {
    fn new(b: ::reqwest::RequestBuilder, client: Client) -> Self {
        Self {
            inner: b,
            client,
            disable_sleep: false,
        }
    }

    pub async fn send(self) -> Result<Response, Error> {
        let req = self.inner.build()?;
        self.client.execute_request(req, self.disable_sleep).await
    }

    pub fn disable_sleep(mut self) -> Self {
        self.disable_sleep = true;
        self
    }

    pub fn form<T: Serialize + ?Sized>(mut self, form: &T) -> Self {
        self.inner = self.inner.form(form);
        self
    }

    pub fn json<T: Serialize + ?Sized>(mut self, json: &T) -> Self {
        self.inner = self.inner.json(json);
        self
    }

    pub fn header<K, V>(self, key: K, value: V) -> RequestBuilder
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<::http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<::http::Error>,
    {
        Self::new(self.inner.header(key, value), self.client)
    }
}
