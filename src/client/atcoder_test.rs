use chrono::Local;
use chrono::TimeZone;

use crate::testconfig::TestConfig;

use super::atcoder::*;
use super::common::*;

#[test]
fn should_be_contest_url() {
    let cli = AtCoderClient::new();
    let is_contest_url = move |url: &str| cli.is_contest_url(&Url::parse(url).unwrap());

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
fn should_not_be_contest_url() {
    let cli = AtCoderClient::new();
    let is_contest_url = move |url: &str| cli.is_contest_url(&Url::parse(url).unwrap());

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
            ProblemInfo {
                url: "https://atcoder.jp/contests/abc001/tasks/abc001_1".to_owned(),
                ord: 1,
                short_title: "A".to_owned(),
                long_title: "積雪深差".to_owned(),
            },
            ProblemInfo {
                url: "https://atcoder.jp/contests/abc001/tasks/abc001_2".to_owned(),
                ord: 2,
                short_title: "B".to_owned(),
                long_title: "視程の通報".to_owned(),
            },
            ProblemInfo {
                url: "https://atcoder.jp/contests/abc001/tasks/abc001_3".to_owned(),
                ord: 3,
                short_title: "C".to_owned(),
                long_title: "風力観測".to_owned(),
            },
            ProblemInfo {
                url: "https://atcoder.jp/contests/abc001/tasks/abc001_4".to_owned(),
                ord: 4,
                short_title: "D".to_owned(),
                long_title: "感雨時刻の整理".to_owned(),
            },
        ]
    )
}

#[tokio::test]
async fn fetch_abc003_4_testcases() {
    let url = "https://atcoder.jp/contests/abc003/tasks/abc003_4";
    let cli = AtCoderClient::new();
    let testcases = cli
        .fetch_testcases(&Url::parse(&url).unwrap())
        .await
        .unwrap();
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
async fn fetch_abc086_a_testcases() {
    let url = "https://atcoder.jp/contests/abs/tasks/abc086_a";
    let cli = AtCoderClient::new();
    let testcases = cli
        .fetch_testcases(&Url::parse(&url).unwrap())
        .await
        .unwrap();
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
    );
}

#[test]
fn serialize_auth_data() {
    let cli = AtCoderClient::new().with_auth(AuthCookie {
        session_id: Some("test_session_id".to_owned()),
    });
    let json = cli.auth_data().to_json();
    assert_eq!(json, r#"{"session_id":"test_session_id"}"#);
}

#[test]
fn serialize_null_auth_data() {
    let cli = AtCoderClient::new();
    let json = cli.auth_data().to_json();
    assert_eq!(json, r#"{"session_id":null}"#);
}

#[tokio::test]
async fn login_and_submit_and_logout() {
    let TestConfig {
        atcoder_username: username,
        atcoder_password: password,
    } = TestConfig::from_env().unwrap_or_else(|e| panic!("{:?}", e));

    let mut cli = AtCoderClient::new();

    cli.login(Box::new(Cred { username, password }))
        .await
        .unwrap_or_else(|e| panic!("{:?}", e));

    cli.submit(
        &Url::parse("https://atcoder.jp/contests/abs/tasks/abc086_a").unwrap(),
        &PgLang::new("Python (3.8.2)", "4006"),
        [
            "a, b = map(int, input().split())",
            "print(('Even', 'Odd')[a * b & 1])",
        ]
        .join("\n")
        .as_ref(),
    )
    .await
    .unwrap_or_else(|e| panic!("{:?}", e));

    cli.logout().await.unwrap_or_else(|e| panic!("{:?}", e));
}
