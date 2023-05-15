use std::collections::HashMap;

use super::common::*;
use crate::{errors::*, util};
use anyhow::anyhow;
use async_trait::async_trait;
use chrono::DateTime;
use lazy_regex::{lazy_regex, Lazy, Regex};
use reqwest::StatusCode;
use scraper::{ElementRef, Html, Selector};

pub struct AtCoderClient {
    http: reqwest::Client,
}

pub struct Cred {
    pub username: String,
    pub password: String,
}

static RE_CONTEST_URL_PATH: Lazy<Regex> = lazy_regex!(r"^/contests/([[:alnum:]]+)/?$");
static RE_PROBLEM_URL_PATH: Lazy<Regex> =
    lazy_regex!(r"^/contests/([[:alnum:]]+)/tasks/([[:alnum:]]+)_([[:alnum:]]+)/?$");

pub const HOST: &'static str = "atcoder.jp";
pub const HOME_URL: &'static str = "https://atcoder.jp/home";
pub const LOGIN_URL: &'static str = "https://atcoder.jp/login";
pub const LOGOUT_URL: &'static str = "https://atcoder.jp/logout";

fn extract_testcase(pre: ElementRef) -> String {
    let node = pre.first_child().unwrap().value();
    node.as_text().unwrap().trim().to_owned()
}

impl AtCoderClient {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::builder()
                .cookie_store(true)
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl Client for AtCoderClient {
    type Credential = Cred;

    fn is_contest_url(&self, url: &Url) -> bool {
        url.scheme() == "https"
            && url.host_str() == Some(HOST)
            && RE_CONTEST_URL_PATH.is_match(url.path())
    }

    fn is_problem_url(&self, url: &Url) -> bool {
        url.scheme() == "https"
            && url.host_str() == Some(HOST)
            && RE_PROBLEM_URL_PATH.is_match(url.path())
    }

    async fn fetch_contest_info(&self, contest_url: &Url) -> Result<ContestInfo> {
        let tasks_en_url = {
            let mut url = contest_url.clone();
            url.set_path(&format!(
                "{}/tasks",
                contest_url.path().trim_end_matches('/')
            ));
            url.set_query(Some("lang=en"));
            url
        };
        let tasks_html = self.http.get(tasks_en_url).send().await?.text().await?;
        let doc = Html::parse_document(&tasks_html);

        let short_title = {
            let caps = RE_CONTEST_URL_PATH.captures(contest_url.path()).unwrap();
            caps[1].to_owned()
        };
        let long_title = {
            let sel = Selector::parse("#navbar-collapse .contest-title").unwrap();
            let node = doc.select(&sel).next().unwrap();
            node.text().next().unwrap().to_owned()
        };
        let (start_at, end_at) = {
            let sel = Selector::parse("#contest-nav-tabs .contest-duration>a>time").unwrap();
            let mut itr = doc.select(&sel);
            let node1 = itr.next().unwrap();
            let node2 = itr.next().unwrap();
            let (s1, s2) = (node1.text().next().unwrap(), node2.text().next().unwrap());

            const FMT: &str = "%Y-%m-%d %H:%M:%S%z";
            use chrono::Local;
            let t1 = DateTime::parse_from_str(s1.trim(), FMT).unwrap();
            let t2 = DateTime::parse_from_str(s2.trim(), FMT).unwrap();
            (t1.with_timezone(&Local), t2.with_timezone(&Local))
        };
        let problems: Vec<ProblemInfo> = {
            let sel_tr = Selector::parse("#main-container table > tbody > tr").unwrap();
            let sel_short_title = Selector::parse("td:first-child > a").unwrap();
            let sel_long_title = Selector::parse("td:nth-child(2) > a").unwrap();
            doc.select(&sel_tr)
                .enumerate()
                .map(|(i, node)| {
                    let el1 = node.select(&sel_short_title).next().unwrap();
                    let el2 = node.select(&sel_long_title).next().unwrap();
                    let url = util::complete_url(el1.value().attr("href").unwrap(), HOST);
                    ProblemInfo {
                        url,
                        ord: (i + 1) as u32,
                        short_title: el1.text().next().unwrap().trim().to_owned(),
                        long_title: el2.text().next().unwrap().trim().to_owned(),
                    }
                })
                .collect()
        };

        Ok(ContestInfo {
            url: contest_url.to_string(),
            short_title,
            long_title,
            problems,
            start_at,
            end_at,
        })
    }

    async fn fetch_testcases(&self, problem_url: &Url) -> Result<Vec<Testcase>> {
        let html = self
            .http
            .get(problem_url.clone())
            .send()
            .await?
            .text()
            .await?;
        let doc = Html::parse_document(&html);

        let sel_parts_current_ver = Selector::parse("#task-statement .lang-en > .part").unwrap();
        let sel_parts_old_ver = Selector::parse("#task-statement > .part").unwrap();
        let sel_h3 = Selector::parse("h3").unwrap();
        let sel_pre = Selector::parse("pre").unwrap();

        let mut in_cases = Vec::with_capacity(5);
        let mut out_cases = Vec::with_capacity(5);

        for node in doc
            .select(&sel_parts_current_ver)
            .chain(doc.select(&sel_parts_old_ver))
        {
            let h3 = node.select(&sel_h3).next().unwrap();
            let title = h3.text().next().unwrap().trim().to_lowercase();
            if title.starts_with("入力例") || title.starts_with("sample input") {
                let pre = node.select(&sel_pre).next().unwrap();
                in_cases.push(extract_testcase(pre));
            } else if title.starts_with("出力例") || title.starts_with("sample output") {
                let pre = node.select(&sel_pre).next().unwrap();
                out_cases.push(extract_testcase(pre));
            }
        }
        let cases: Vec<_> = in_cases
            .into_iter()
            .zip(out_cases)
            .enumerate()
            .map(|(i, (input, expected))| Testcase {
                ord: (i + 1) as u32,
                input,
                expected,
            })
            .collect();
        Ok(cases)
    }

    async fn login(&mut self, cred: Self::Credential) -> Result<()> {
        let csrf_token = {
            let html = self.http.get(LOGIN_URL).send().await?.text().await?;
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("#main-container form > input[name='csrf_token']").unwrap();
            let el = doc.select(&sel).next().unwrap().value();
            el.attr("value").unwrap().to_owned()
        };
        let resp = {
            let mut params = HashMap::new();
            params.insert("username", cred.username);
            params.insert("password", cred.password);
            params.insert("csrf_token", csrf_token);
            self.http.post(LOGIN_URL).form(&params).send().await?
        };
        let location = util::extract_location_header(&resp, StatusCode::FOUND)?;
        let redirected_url = util::complete_url(&location, HOST);
        match redirected_url.as_str() {
            HOME_URL => Ok(()),
            LOGIN_URL => Err(anyhow!("Wrong username or password")),
            _ => Err(anyhow!("Unexpected redirect url: {}", redirected_url)),
        }
    }

    fn ask_credential(&self) -> Result<&Self::Credential> {
        todo!()
    }

    async fn logout(&mut self) -> Result<()> {
        let csrf_token = {
            let html = self.http.get(HOME_URL).send().await?.text().await?;
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("#main-div form > input[name='csrf_token']").unwrap();
            let el = doc.select(&sel).next().unwrap().value();
            el.attr("value").unwrap().to_owned()
        };
        let resp = {
            let mut params = HashMap::new();
            params.insert("csrf_token", csrf_token);
            self.http.post(LOGOUT_URL).form(&params).send().await?
        };
        let location = util::extract_location_header(&resp, StatusCode::FOUND)?;
        let redirected_url = util::complete_url(&location, HOST);
        match redirected_url.as_str() {
            HOME_URL => Ok(()),
            _ => Err(anyhow!("Unexpected redirect url: {}", redirected_url)),
        }
    }

    async fn submit(
        &self,
        problem_url: &Url,
        lang: &PgLang,
        source_code: &str,
    ) -> Result<SubmissionID> {
        todo!()
    }
}
