use ::async_trait::async_trait;
use ::chrono::DateTime;
use ::cookie::Cookie;
use ::reqwest::cookie::CookieStore as _;
use ::std::{collections::HashMap, time::Duration};
use chrono::TimeZone;
use serde::Deserialize;

use super::{auth::AuthCookie, helper, urls::*};
use crate::{
    error::*,
    model::*,
    util::{self, DocExt as _, ElementExt as _, ElementRefExt as _},
};

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

//---------------------------------------------------------
// Virtual Contest Data on AtCoderProblems
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ProblemsVirtualContest {
    pub info: ProblemsVirtualContestInfo,
    pub problems: Vec<ProblemsVirtualContestProblem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ProblemsVirtualContestInfo {
    pub id: String,
    pub title: String,
    pub duration_second: i64,
    pub start_epoch_second: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ProblemsVirtualContestProblem {
    pub id: String,
    pub order: u32,
}

//---------------------------------------------------------

pub struct AtCoderClient {
    http: crate::http::Client,
}

const COOKIE_KEY_SESSION_ID: &str = "REVEL_SESSION";

impl AtCoderClient {
    pub fn new() -> Self {
        use ::glob::Pattern;
        Self {
            http: crate::http::Client::new(
                crate::http::redirect::Policy::none(),
                [
                    (
                        Pattern::new("https://atcoder.jp*").unwrap(),
                        Duration::from_millis(600),
                    ),
                    (
                        Pattern::new("https://kenkoooo.com*").unwrap(),
                        Duration::from_millis(200),
                    ),
                ],
            ),
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
        let raw_cookies = match self.http.cookie_jar.cookies(&TOP_URL) {
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
        self.http.cookie_jar.add_cookie_str(&cookie, &TOP_URL);
    }

    pub fn revoke_auth(&mut self) {
        let cookie = format!("{}=", COOKIE_KEY_SESSION_ID);
        self.http.cookie_jar.add_cookie_str(&cookie, &TOP_URL);
    }

    async fn fetch_atcoder_contest_info(&self, contest_url: &Url) -> Result<ContestInfo> {
        let tasks_en_url = {
            let mut url = contest_url.clone();
            url.set_path(&format!(
                "{}/tasks",
                contest_url.path().trim_end_matches('/')
            ));
            url.set_query(Some("lang=en"));
            url
        };
        let doc = util::fetch_html(&self.http, tasks_en_url).await?;

        let short_title = {
            let caps = RE_CONTEST_URL_PATH.captures(contest_url.path()).unwrap();
            caps[1].to_owned()
        };
        let long_title = {
            let sel = util::selector_must_parsed("#navbar-collapse .contest-title");
            let node = doc.select_first(&sel)?;
            node.first_text(&sel)?.to_owned()
        };
        let (start_at, end_at) = {
            let sel = util::selector_must_parsed("#contest-nav-tabs .contest-duration>a>time");
            let (node1, node2) = doc.select_double(&sel)?;
            let (s1, s2) = (node1.first_text(&sel)?, node2.first_text(&sel)?);

            const FMT: &str = "%Y-%m-%d %H:%M:%S%z";
            use chrono::Local;
            let t1 = DateTime::parse_from_str(s1.trim(), FMT).unwrap();
            let t2 = DateTime::parse_from_str(s2.trim(), FMT).unwrap();
            (t1.with_timezone(&Local), t2.with_timezone(&Local))
        };
        let problems: Vec<ContestProblemOutline> = {
            let sel_tr = util::selector_must_parsed("#main-container table > tbody > tr");
            let sel_title = util::selector_must_parsed("td:nth-child(2) > a");
            let res: Result<Vec<_>> = doc
                .select(&sel_tr)
                .enumerate()
                .map(|(i, node)| {
                    let title_el = node.select_first(&sel_title)?;
                    let url_path = title_el.value().get_attr("href", &sel_title)?;
                    let url = util::complete_url(url_path, DOMAIN)?;
                    Ok(ContestProblemOutline {
                        url,
                        ord: (i + 1) as u32,
                    })
                })
                .collect();
            res?
        };
        Ok(ContestInfo {
            url: contest_url.to_owned(),
            short_title,
            long_title,
            problems,
            start_at,
            end_at,
        })
    }

    pub async fn fetch_problems_virtual_contest_info(&self, url: &Url) -> Result<ContestInfo> {
        let Some(caps) = RE_PROBLEMS_VIRTUAL_CONTEST_URL_FRAGMENT.captures(url.fragment().unwrap_or("")) else {
            return Err(Error::NotContestUrl(url.to_owned()));
        };
        let contest_id = &caps[1];
        let api_url = format!(
            "https://{}/atcoder/internal-api/contest/get/{}",
            DOMAIN_KENKOOOO, contest_id
        );

        let contest: ProblemsVirtualContest =
            util::fetch_json_with_parse_url(&self.http, &api_url).await?;

        let short_title = format!("problems-bacha-{}", &contest_id[..8]);
        let long_title = contest.info.title;

        let start_at = {
            let nano_secs = 0;
            chrono::Local
                .timestamp_opt(contest.info.start_epoch_second, nano_secs)
                .unwrap()
        };
        let end_at = start_at + chrono::Duration::seconds(contest.info.duration_second);

        let problems: Vec<_> = contest
            .problems
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let ord = i as u32 + 1;
                // "abc001_a" => "abc001"
                let contest_name = x.id.rsplit_once('_').unwrap().0;
                let url = util::complete_url(
                    format!("/contests/{}/tasks/{}", contest_name, x.id),
                    DOMAIN,
                )
                .unwrap();
                ContestProblemOutline { url, ord }
            })
            .collect();

        Ok(ContestInfo {
            url: url.to_owned(),
            short_title,
            long_title,
            problems,
            start_at,
            end_at,
        })
    }
}

#[async_trait]
impl Client for AtCoderClient {
    fn platform(&self) -> Platform {
        Platform::AtCoder
    }

    fn is_contest_home_url(&self, url: &Url) -> bool {
        AtCoderUrlAnalyzer::is_contest_home_url(url)
    }

    fn is_problem_url(&self, url: &Url) -> bool {
        AtCoderUrlAnalyzer::is_problem_url(url)
    }

    fn extract_problem_id(&self, url: &Url) -> problem_id::Result<ProblemId> {
        AtCoderUrlAnalyzer::extract_problem_id(url)
    }

    async fn fetch_contest_info(&self, url: &Url) -> Result<ContestInfo> {
        if AtCoderUrlAnalyzer::is_atcoder_contest_home_url(url) {
            self.fetch_atcoder_contest_info(url).await
        } else if AtCoderUrlAnalyzer::is_problems_virtual_contest_url(url) {
            self.fetch_problems_virtual_contest_info(url).await
        } else {
            Err(Error::NotContestUrl(url.to_owned()))
        }
    }

    async fn fetch_problem_detail(
        &self,
        problem_url: &Url,
    ) -> Result<(ProblemInfo, Vec<SampleTestcase>)> {
        let doc = util::fetch_html(&self.http, problem_url.clone()).await?;
        let title = {
            let sel = util::selector_must_parsed("#main-container > div .h2");
            let node = doc.select_first(&sel)?;
            let s = node.first_text(&sel)?.trim().to_owned();
            let (_, title) = s.split_once("-").unwrap();
            title.trim().to_owned()
        };
        let (execution_time_limit, memory_limit_kb) = {
            let sel = util::selector_must_parsed("#main-container > div > div:nth-child(2) > p");
            let node = doc.select_first(&sel)?;
            let text = node.first_text(&sel)?;
            let xs: Vec<_> = text.split("/").map(str::trim).collect();

            let (time_limit, memory_limit) = if xs[0].starts_with("Time") {
                (
                    xs[0].strip_prefix("Time Limit:").unwrap().trim(),
                    xs[1].strip_prefix("Memory Limit:").unwrap().trim(),
                )
            } else {
                (
                    xs[0].strip_prefix("実行時間制限:").unwrap().trim(),
                    xs[1].strip_prefix("メモリ制限:").unwrap().trim(),
                )
            };
            (
                parse_duration_str(time_limit),
                parse_memory_str_as_kb(memory_limit),
            )
        };
        let problem_id = unsafe { self.extract_problem_id(problem_url).unwrap_unchecked() };
        let testcases = helper::scrape_testcases(&doc)?;
        let info = ProblemInfo {
            platform: self.platform(),
            url: problem_url.to_owned(),
            problem_id,
            title,
            execution_time_limit,
            memory_limit_kb,
        };
        Ok((info, testcases))
    }

    fn credential_fields(&self) -> &'static [CredFieldMeta] {
        use CredFieldKind::*;
        &[
            CredFieldMeta {
                name: "username",
                kind: Text,
            },
            CredFieldMeta {
                name: "password",
                kind: Password,
            },
        ]
    }

    async fn login(&mut self, cred: CredMap) -> Result<()> {
        let csrf_token = {
            let doc = util::fetch_html_with_parse_url(&self.http, LOGIN_URL).await?;
            let sel = util::selector_must_parsed("#main-container form > input[name='csrf_token']");
            let el = doc.select_first(&sel)?.value();
            el.get_attr("value", &sel)?.to_owned()
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
            let doc = util::fetch_html_with_parse_url(&self.http, HOME_URL).await?;
            let sel = util::selector_must_parsed("#main-div form > input[name='csrf_token']");
            let el = doc.select_first(&sel)?.value();
            el.get_attr("value", &sel)?.to_owned()
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

    async fn fetch_submittable_language_list(&self) -> Result<Vec<PgLang>> {
        let url = Url::parse("https://atcoder.jp/contests/practice/custom_test").unwrap();
        ensure!(
            self.get_auth().session_id.is_some(),
            Error::NeedLogin {
                requested_url: url.to_string(),
            }
        );
        let doc = util::fetch_html(&self.http, url).await?;

        let sel = util::selector_must_parsed("#select-lang select > option[value]");

        let langs = doc
            .select(&sel)
            .map(|el| {
                Ok(PgLang {
                    id: el.value().get_attr("value", &sel)?.to_owned(),
                    name: el.first_text(&sel)?.trim().to_owned(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        if langs.is_empty() {
            return Err(Error::NoSuchElementMatchesToSelector(sel));
        }
        Ok(langs)
    }

    async fn submit(&self, problem_url: &Url, lang: &PgLang, source_code: &str) -> Result<Url> {
        ensure!(
            self.get_auth().session_id.is_some(),
            Error::NeedLogin {
                requested_url: problem_url.to_string(),
            }
        );
        let csrf_token = {
            let url = problem_url.clone();
            let doc = util::fetch_html(&self.http, url).await?;
            let sel = util::selector_must_parsed("#main-div form > input[name='csrf_token']");
            let el = doc.select_first(&sel)?.value();
            el.get_attr("value", &sel)?.to_owned()
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
            path if path == submissions_path => Ok(util::complete_url(path, DOMAIN)?),
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

fn parse_duration_str(s: &str) -> Duration {
    let s = s.trim();

    if s.ends_with("sec") {
        let n = s.strip_suffix("sec").unwrap().trim().parse().unwrap();
        Duration::from_secs(n)
    } else if s.ends_with("secs") {
        let n = s.strip_suffix("secs").unwrap().trim().parse().unwrap();
        Duration::from_secs(n)
    } else if s.ends_with("ms") {
        let n = s.strip_suffix("ms").unwrap().trim().parse().unwrap();
        Duration::from_millis(n)
    } else if s.ends_with("s") {
        let n = s.strip_suffix("s").unwrap().trim().parse().unwrap();
        Duration::from_secs(n)
    } else {
        panic!("Cannot parse as duration: {}", s);
    }
}

fn parse_memory_str_as_kb(s: &str) -> u32 {
    let s = s.trim().to_lowercase();

    if s.ends_with("gb") {
        let n: u32 = s.strip_suffix("gb").unwrap().trim().parse().unwrap();
        n * 1024 * 1024
    } else if s.ends_with("mb") {
        let n: u32 = s.strip_suffix("mb").unwrap().trim().parse().unwrap();
        n * 1024
    } else if s.ends_with("kb") {
        let n: u32 = s.strip_suffix("kb").unwrap().trim().parse().unwrap();
        n
    } else {
        panic!("Cannot parse as memory units: {}", s);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_duration_str_ok() {
        assert_eq!(parse_duration_str("3 sec"), Duration::from_secs(3));
        assert_eq!(parse_duration_str("3 secs"), Duration::from_secs(3));
        assert_eq!(parse_duration_str("3 s"), Duration::from_secs(3));
        assert_eq!(parse_duration_str("3sec"), Duration::from_secs(3));
        assert_eq!(parse_duration_str("100ms"), Duration::from_millis(100));
    }

    #[test]
    fn parse_memory_str_as_kb_ok() {
        assert_eq!(parse_memory_str_as_kb("1 GB"), 1024 * 1024);
        assert_eq!(parse_memory_str_as_kb("5 MB"), 5 * 1024);
        assert_eq!(parse_memory_str_as_kb("512 KB"), 512);
        assert_eq!(parse_memory_str_as_kb("3 kb"), 3);
        assert_eq!(parse_memory_str_as_kb("1GB"), 1024 * 1024);
    }
}
