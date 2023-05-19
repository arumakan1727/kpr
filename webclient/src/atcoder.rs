use async_trait::async_trait;
use chrono::DateTime;
use cookie::Cookie;
use lazy_regex::{lazy_regex, Lazy, Regex};
use reqwest::cookie::{CookieStore as _, Jar};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

use crate::{error::*, model::*, util};

macro_rules! bail {
    ($e:expr) => {
        return Err($e.into())
    };
}

macro_rules! ensure {
    ($cond:expr, $e:expr) => {
        if !($cond) {
            bail!($e);
        }
    };
}

pub struct AtCoderClient {
    http: reqwest::Client,
    jar: Arc<Jar>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthCookie {
    pub session_id: Option<String>,
}

impl AuthCookie {
    pub fn empty() -> Self {
        AuthCookie { session_id: None }
    }
    pub fn from_json(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn revoke(&mut self) {
        self.session_id = None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtCoderCred {
    pub username: String,
    pub password: String,
}

impl From<AtCoderCred> for CredMap {
    fn from(c: AtCoderCred) -> Self {
        let mut h = CredMap::new();
        h.insert("username", c.username);
        h.insert("password", c.password);
        h
    }
}

static RE_CONTEST_URL_PATH: Lazy<Regex> = lazy_regex!(r"^/contests/([[:alnum:]]+)/?$");
static RE_PROBLEM_URL_PATH: Lazy<Regex> =
    lazy_regex!(r"^/contests/([[:alnum:]]+)/tasks/(([[:alnum:]]+)_([[:alnum:]]+))/?$");

pub const DOMAIN: &'static str = "atcoder.jp";
pub const HOME_URL: &'static str = "https://atcoder.jp/home";
pub const LOGIN_URL: &'static str = "https://atcoder.jp/login";
pub const LOGOUT_URL: &'static str = "https://atcoder.jp/logout";
pub static URL: Lazy<Url> = Lazy::new(|| Url::parse("https://atcoder.jp").unwrap());

const COOKIE_KEY_SESSION_ID: &'static str = "REVEL_SESSION";

fn extract_testcase(pre: ElementRef) -> String {
    let node = pre.first_child().unwrap().value();
    node.as_text().unwrap().trim().to_owned()
}

impl AtCoderClient {
    pub fn new() -> Self {
        let jar = Arc::new(Jar::default());
        Self {
            http: reqwest::Client::builder()
                .cookie_store(true)
                .cookie_provider(jar.clone())
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap(),
            jar,
        }
    }

    pub fn with_auth(mut self, a: AuthCookie) -> Self {
        match a.session_id {
            Some(sid) => self.set_auth(&sid),
            None => self.revoke_auth(),
        }
        self
    }

    pub fn get_auth(&self) -> AuthCookie {
        let raw_cookies = match self.jar.cookies(&URL) {
            Some(s) => s,
            None => return AuthCookie { session_id: None },
        };
        let raw_cookies = raw_cookies.to_str().unwrap();
        let cookie = Cookie::split_parse(raw_cookies).find_map(|c| match c {
            Ok(c) if c.name() == COOKIE_KEY_SESSION_ID && !c.value().is_empty() => Some(c),
            _ => None,
        });
        AuthCookie {
            session_id: cookie.map(|c| c.value().to_owned()),
        }
    }

    pub fn set_auth(&mut self, session_id: &str) {
        let cookie = format!(
            "{}={}; Path=/; HttpOnly; Secure; Domain={}",
            COOKIE_KEY_SESSION_ID, session_id, DOMAIN,
        );
        self.jar.add_cookie_str(&cookie, &URL);
    }

    pub fn revoke_auth(&mut self) {
        let cookie = format!("{}=", COOKIE_KEY_SESSION_ID);
        self.jar.add_cookie_str(&cookie, &URL);
    }
}

#[async_trait]
impl Client for AtCoderClient {
    fn platform(&self) -> Platform {
        Platform::AtCoder
    }

    fn is_contest_url(&self, url: &Url) -> bool {
        url.scheme() == "https"
            && url.host_str() == Some(DOMAIN)
            && RE_CONTEST_URL_PATH.is_match(url.path())
    }

    fn is_problem_url(&self, url: &Url) -> bool {
        url.scheme() == "https"
            && url.host_str() == Some(DOMAIN)
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
                    let url = util::complete_url(el1.value().attr("href").unwrap(), DOMAIN);
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

    fn credential_fields(&self) -> &'static [CredField] {
        use CredFieldKind::*;
        &[
            CredField {
                name: "username",
                kind: Text,
            },
            CredField {
                name: "password",
                kind: Password,
            },
        ]
    }

    async fn login(&mut self, cred: CredMap) -> Result<()> {
        let csrf_token = {
            let html = self.http.get(LOGIN_URL).send().await?.text().await?;
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("#main-container form > input[name='csrf_token']").unwrap();
            let el = doc.select(&sel).next().unwrap().value();
            el.attr("value").unwrap().to_owned()
        };
        let resp = {
            let mut params = cred;
            params.insert("csrf_token", csrf_token);
            self.http.post(LOGIN_URL).form(&params).send().await?
        };
        let location = util::extract_302_location_header(&resp, LOGIN_URL)?;
        match location.as_str() {
            "/home" => (),
            path if path.starts_with("/login") => bail!(Error::WrongCredential {
                fields: "username or password",
            }),
            _ => bail!(Error::UnexpectedRedirectPath {
                got: location,
                expected: "/home".to_owned(),
                requested_url: LOGIN_URL.to_owned(),
            }),
        };
        Ok(())
    }

    fn is_logged_in(&self) -> bool {
        self.get_auth().session_id.is_some()
    }

    fn export_authtoken_as_json(&self) -> String {
        self.get_auth().to_json()
    }

    fn load_authtoken_json(&mut self, serialized_auth: &str) -> Result<()> {
        AuthCookie::from_json(serialized_auth)?
            .session_id
            .map(|sid| self.set_auth(&sid));
        Ok(())
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
        self.revoke_auth();
        let location = util::extract_302_location_header(&resp, LOGOUT_URL)?;
        match location.as_str() {
            "/home" => Ok(()),
            _ => Err(Error::UnexpectedRedirectPath {
                got: location,
                expected: "/home".to_owned(),
                requested_url: LOGOUT_URL.to_owned(),
            }),
        }
    }

    async fn submit(&self, problem_url: &Url, lang: &PgLang, source_code: &str) -> Result<()> {
        ensure!(
            self.get_auth().session_id.is_some(),
            Error::NeedLogin {
                requested_url: problem_url.to_string(),
            }
        );
        let csrf_token = {
            let url = problem_url.clone();
            let html = self.http.get(url).send().await?.text().await?;
            let doc = Html::parse_document(&html);
            let sel = Selector::parse("#main-div form > input[name='csrf_token']").unwrap();
            let el = doc.select(&sel).next().unwrap().value();
            el.attr("value").unwrap().to_owned()
        };
        let (contest_name, task_name) = {
            let caps = RE_PROBLEM_URL_PATH.captures(problem_url.path()).unwrap();
            (caps[1].to_owned(), caps[2].to_owned())
        };
        let submit_url = {
            let mut url = problem_url.clone();
            url.set_path(&format!("/contests/{}/submit", contest_name));
            url
        };
        let resp = {
            let mut params = HashMap::new();
            params.insert("sourceCode", source_code);
            params.insert("data.LanguageId", &lang.id);
            params.insert("data.TaskScreenName", &task_name);
            params.insert("csrf_token", &csrf_token);
            self.http
                .post(submit_url.clone())
                .form(&params)
                .send()
                .await?
        };
        let location = util::extract_302_location_header(&resp, submit_url)?;
        let submissions_path = format!("/contests/{}/submissions/me", contest_name);
        match location.as_str() {
            path if path == submissions_path => Ok(()),
            path if path.starts_with("/login") => Err(Error::NeedLogin {
                requested_url: problem_url.to_string(),
            }),
            _ => Err(Error::UnexpectedRedirectPath {
                got: location,
                expected: submissions_path,
                requested_url: problem_url.to_string(),
            }),
        }
    }
}
