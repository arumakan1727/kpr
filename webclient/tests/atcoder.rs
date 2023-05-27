use std::thread;
use std::time::Duration;

use chrono::Local;
use chrono::TimeZone;
use once_cell::sync::Lazy;
use rand::Rng;

use kpr_webclient::atcoder::*;
use kpr_webclient::*;

mod testconfig;
use testconfig::TestConfig;

fn sleep_random_ms() {
    let mut rng = rand::thread_rng();
    let ms = Duration::from_millis(rng.gen_range(200..1000));
    thread::sleep(ms);
}

#[test]
fn should_be_contest_home_url() {
    let cli = AtCoderClient::new();
    let is_contest_url = move |url: &str| cli.is_contest_home_url(&Url::parse(url).unwrap());

    assert!(is_contest_url("https://atcoder.jp/contests/abc001"));
    assert!(is_contest_url("https://atcoder.jp/contests/abc001/"));
    assert!(is_contest_url("https://atcoder.jp/contests/typical90"));
    assert!(is_contest_url("https://atcoder.jp/contests/typical90/"));
    assert!(is_contest_url("https://atcoder.jp/contests/typical90#"));
    assert!(is_contest_url("https://atcoder.jp/contests/typical90#a"));
    assert!(is_contest_url("https://atcoder.jp/contests/typical90/#a"));
    assert!(is_contest_url("https://atcoder.jp/contests/abc001#a?x")); // fragment is 'a?x'
    assert!(is_contest_url("https://atcoder.jp/contests/abc001?lang=en"));
    assert!(is_contest_url(
        "https://atcoder.jp/contests/abc001/?lang=ja"
    ));
    assert!(is_contest_url("https://atcoder.jp/contests/abc001/?hoge"));
}

#[test]
fn should_not_be_contest_home_url() {
    let cli = AtCoderClient::new();
    let is_contest_url = move |url: &str| cli.is_contest_home_url(&Url::parse(url).unwrap());

    assert!(!is_contest_url("https://atcoder.jp/contests"));
    assert!(!is_contest_url("https://atcoder.jp/contests/"));
    assert!(
        !is_contest_url("https://atcoder.com/contests/abc001"),
        "'atcoder.com' must be invalid"
    );
    assert!(
        !is_contest_url("http://atcoder.jp/contests/"),
        "'http' must be invalid"
    );
    assert!(
        !is_contest_url("https://atcoder.jp/contests/abc001/abc001_a"),
        "problem url is invalid for contest url"
    );
}

#[test]
fn test_problem_global_id() {
    let cli = AtCoderClient::new();
    let problem_global_id = move |url: &str| {
        cli.problem_global_id(&Url::parse(url).unwrap())
            .map(|global_id| global_id.to_string())
    };

    assert_eq!(
        problem_global_id("https://atcoder.jp/contests/abc001/tasks/abc001_1").unwrap(),
        "abc001_1"
    );
    assert_eq!(
        problem_global_id("https://atcoder.jp/contests/abc334/tasks/abc334_f").unwrap(),
        "abc334_f"
    );
    assert!(problem_global_id("https://atcoder.jp/contests/abc334").is_err());
}

#[test]
fn should_be_problem_url() {
    let cli = AtCoderClient::new();
    let is_problem_url = move |url: &str| cli.is_problem_url(&Url::parse(url).unwrap());

    assert!(is_problem_url(
        "https://atcoder.jp/contests/abc001/tasks/abc001_1"
    ));
    assert!(is_problem_url(
        "https://atcoder.jp/contests/abc001/tasks/abc001_a/"
    ));
    assert!(is_problem_url(
        "https://atcoder.jp/contests/typical90/tasks/typical90_a"
    ));
    assert!(is_problem_url(
        "https://atcoder.jp/contests/abs/tasks/abc086_a/"
    ));
    assert!(is_problem_url(
        "https://atcoder.jp/contests/abc001/tasks/abc001_a#a?x"
    )); // fragment is 'a?x'
    assert!(is_problem_url(
        "https://atcoder.jp/contests/abc001/tasks/abc001_a/#a"
    ));
    assert!(is_problem_url(
        "https://atcoder.jp/contests/abc001/tasks/abc001_1?lang=en"
    ));
    assert!(is_problem_url(
        "https://atcoder.jp/contests/abc001/tasks/abc001_1/?lang=ja"
    ));
    assert!(is_problem_url(
        "https://atcoder.jp/contests/abc001/tasks/typical90_a/?hoge"
    ));
}

#[test]
fn should_not_be_problem_url() {
    let cli = AtCoderClient::new();
    let is_problem_url = move |url: &str| cli.is_problem_url(&Url::parse(url).unwrap());
    assert!(!is_problem_url("https://atcoder.jp/contests/abc001"));
    assert!(!is_problem_url("https://atcoder.jp/contests/abc001/"));
    assert!(!is_problem_url("https://atcoder.jp/contests/abc001/abc001"));
    assert!(!is_problem_url("https://atcoder.jp/contests/abc001/tasks"));
    assert!(!is_problem_url("https://atcoder.jp/contests/abc001/tasks/"));
    assert!(!is_problem_url("https://atcoder.jp/contests/abc001/submit"));
    assert!(!is_problem_url(
        "https://atcoder.jp/contests/abc001/abc001_1"
    ));
    assert!(!is_problem_url(
        "https://atcoder.jp/contests/abc001/abc001_a"
    ));
    assert!(!is_problem_url(
        "https://atcoder.jp/contests/abc001/abc001_"
    ));
    assert!(!is_problem_url(
        "https://atcoder.jp/contests/abc001/clarifications"
    ));
    assert!(!is_problem_url(
        "https://atcoder.jp/contests/abc001/submissions"
    ));
    assert!(!is_problem_url(
        "https://atcoder.jp/contests/abc001/editorial"
    ));

    assert!(!is_problem_url(
        "http://atcoder.jp/contests/abc001/abc001_a"
    ));
    assert!(!is_problem_url(
        "https://atcoder.com/contests/abc001/abc001_a"
    ));
}

#[tokio::test]
async fn fetch_abc001_info() {
    // Avoid DDos attack
    sleep_random_ms();

    let url = "https://atcoder.jp/contests/abc001";
    let cli = AtCoderClient::new();
    let info = cli
        .fetch_contest_info(&Url::parse(&url).unwrap())
        .await
        .unwrap();

    assert_eq!(info.url, url);
    assert_eq!(info.short_title, "abc001");
    assert_eq!(info.long_title, "AtCoder Beginner Contest 001");
    assert_eq!(
        info.start_at,
        Local.with_ymd_and_hms(2013, 10, 12, 21, 0, 0).unwrap()
    );
    assert_eq!(
        info.end_at,
        Local.with_ymd_and_hms(2013, 10, 12, 23, 0, 0).unwrap()
    );
    assert_eq!(
        info.problems,
        vec![
            ContestProblemOutline {
                url: "https://atcoder.jp/contests/abc001/tasks/abc001_1".to_owned(),
                ord: 1,
                title: "積雪深差".to_owned(),
            },
            ContestProblemOutline {
                url: "https://atcoder.jp/contests/abc001/tasks/abc001_2".to_owned(),
                ord: 2,
                title: "視程の通報".to_owned(),
            },
            ContestProblemOutline {
                url: "https://atcoder.jp/contests/abc001/tasks/abc001_3".to_owned(),
                ord: 3,
                title: "風力観測".to_owned(),
            },
            ContestProblemOutline {
                url: "https://atcoder.jp/contests/abc001/tasks/abc001_4".to_owned(),
                ord: 4,
                title: "感雨時刻の整理".to_owned(),
            },
        ]
    )
}

#[tokio::test]
async fn fetch_abc003_4_detail() {
    // Avoid DDos attack
    sleep_random_ms();

    let url_str = "https://atcoder.jp/contests/abc003/tasks/abc003_4";
    let url = Url::parse(url_str).unwrap();
    let cli = AtCoderClient::new();
    let (problem_meta, testcases) = cli.fetch_problem_detail(&url).await.unwrap();

    assert_eq!(
        problem_meta,
        ProblemMeta {
            platform: Platform::AtCoder,
            url: url_str.to_owned(),
            global_id: GlobalId::try_from(&url).unwrap(),
            title: "AtCoder社の冬".to_owned(),
            execution_time_limit: Duration::from_secs(2),
            memory_limit_kb: 64 * 1024,
        },
    );
    assert_eq!(
        testcases,
        vec![
            Testcase {
                ord: 1,
                input: ["3 2", "2 2", "2 2"].join("\n"),
                expected: "12".to_owned(),
            },
            Testcase {
                ord: 2,
                input: ["4 5", "3 1", "3 0"].join("\n"),
                expected: "10".to_owned(),
            },
            Testcase {
                ord: 3,
                input: ["23 18", "15 13", "100 95"].join("\n"),
                expected: "364527243".to_owned(),
            },
            Testcase {
                ord: 4,
                input: ["30 30", "24 22", "145 132"].join("\n"),
                expected: "976668549".to_owned(),
            },
        ]
    );
}

#[tokio::test]
async fn fetch_abc086_a_detail() {
    // Avoid DDos attack
    sleep_random_ms();

    let url_str = "https://atcoder.jp/contests/abs/tasks/abc086_a";
    let url = Url::parse(url_str).unwrap();
    let cli = AtCoderClient::new();
    let (problem_meta, testcases) = cli.fetch_problem_detail(&url).await.unwrap();

    assert_eq!(
        problem_meta,
        ProblemMeta {
            platform: Platform::AtCoder,
            url: url_str.to_owned(),
            global_id: GlobalId::try_from(&url).unwrap(),
            title: "Product".to_owned(),
            execution_time_limit: Duration::from_secs(2),
            memory_limit_kb: 256 * 1024,
        }
    );
    assert_eq!(
        testcases,
        vec![
            Testcase {
                ord: 1,
                input: "3 4".to_owned(),
                expected: "Even".to_owned(),
            },
            Testcase {
                ord: 2,
                input: "1 21".to_owned(),
                expected: "Odd".to_owned(),
            },
        ]
    )
}

#[test]
fn serialize_auth_data() {
    let cli = AtCoderClient::new().with_auth(AuthCookie {
        session_id: Some("test_session_id".to_owned()),
    });
    let json = cli.export_authtoken_as_json();
    assert_eq!(json, r#"{"session_id":"test_session_id"}"#);
}

#[test]
fn serialize_null_auth_data() {
    let cli = AtCoderClient::new();
    let json = cli.export_authtoken_as_json();
    assert_eq!(json, r#"{"session_id":null}"#);
}

static PYTHON: Lazy<PgLang> = Lazy::new(|| PgLang::new("Python (3.8.2)", "4006"));

const URL_ABC086_A: &str = "https://atcoder.jp/contests/abs/tasks/abc086_a";

async fn submit_abc086_a(cli: &AtCoderClient) -> Result<()> {
    // Avoid DDos attack
    sleep_random_ms();

    cli.submit(
        &Url::parse(URL_ABC086_A).unwrap(),
        &PYTHON,
        [
            "a, b = map(int, input().split())",
            "print(('Even', 'Odd')[a * b & 1])",
        ]
        .join("\n")
        .as_ref(),
    )
    .await
}

#[tokio::test]
async fn senario_login_submit_logout() {
    // Avoid DDos attack
    sleep_random_ms();

    let auth_json = {
        let mut cli1 = AtCoderClient::new();
        let TestConfig {
            atcoder_username: username,
            atcoder_password: password,
        } = TestConfig::from_env();
        cli1.login(AtCoderCred { username, password }.into())
            .await
            .unwrap_or_else(|e| panic!("{:?}", e));

        let auth_json = cli1.export_authtoken_as_json();
        assert_ne!(auth_json, r#"{"session_id":null}"#);
        auth_json
    };

    let mut cli2 = AtCoderClient::new();
    cli2.load_authtoken_json(&auth_json).unwrap();

    submit_abc086_a(&cli2)
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    cli2.logout().await.unwrap_or_else(|e| panic!("{:?}", e));

    let json = cli2.export_authtoken_as_json();
    assert_eq!(json, r#"{"session_id":null}"#);
}

#[tokio::test]
async fn login_with_wrong_password_should_be_fail() {
    // Avoid DDos attack
    sleep_random_ms();

    let mut cli = AtCoderClient::new();
    let username = "test";
    let password = "test";
    let err = cli
        .login(
            AtCoderCred {
                username: username.to_owned(),
                password: password.to_owned(),
            }
            .into(),
        )
        .await
        .err()
        .unwrap();
    match err {
        Error::WrongCredential { fields } => {
            assert_eq!(fields, "username or password");
            let errmsg = err.to_string();
            assert!(!errmsg.contains(username));
            assert!(!errmsg.contains(password));
        }
        _ => panic!("Want ClientError::WrongCredential, but got {:?}", err),
    };
}

#[tokio::test]
async fn submit_without_logined_should_be_fail() {
    // Avoid DDos attack
    sleep_random_ms();

    let cli = AtCoderClient::new();
    let err = submit_abc086_a(&cli).await.err().unwrap();
    match err {
        Error::NeedLogin { requested_url } => {
            assert_eq!(requested_url, URL_ABC086_A);
        }
        _ => panic!("Want ClientError::WrongCredential, but got {:?}", err),
    }
}
