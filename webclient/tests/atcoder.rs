use std::time::Duration;

use chrono::Local;
use chrono::TimeZone;
use once_cell::sync::Lazy;

use kpr_webclient::atcoder::*;
use kpr_webclient::*;

mod testconfig;
use testconfig::TestConfig;

#[tokio::test]
async fn should_be_contest_home_url() {
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
        "https://atcoder.jp/contests/tenka1-2012-qualB"
    ));
    assert!(is_contest_url(
        "https://atcoder.jp/contests/abc001/?lang=ja"
    ));
    assert!(is_contest_url("https://atcoder.jp/contests/abc001/?hoge"));

    // virtual contest
    assert!(is_contest_url(
        "https://kenkoooo.com/atcoder/#/contest/show/10f53d61-58e1-411e-9bfe-5fd0690a31f7"
    ));
}

#[tokio::test]
async fn should_not_be_contest_home_url() {
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

#[tokio::test]
async fn extract_problem_id_ok() {
    let cli = AtCoderClient::new();
    let problem_id = move |url: &str| {
        cli.extract_problem_id(&Url::parse(url).unwrap())
            .map(|problem_id| problem_id.to_string())
    };

    assert_eq!(
        problem_id("https://atcoder.jp/contests/abc001/tasks/abc001_1").unwrap(),
        "abc001_1"
    );
    assert_eq!(
        problem_id("https://atcoder.jp/contests/abc334/tasks/abc334_f").unwrap(),
        "abc334_f"
    );
    assert_eq!(
        problem_id("https://atcoder.jp/contests/typical90/tasks/typical90_a").unwrap(),
        "typical90_a"
    );
    assert_eq!(
        problem_id("https://atcoder.jp/contests/practice2/tasks/practice2_a").unwrap(),
        "practice2_a"
    );
}
#[tokio::test]
async fn extract_problem_id_ng() {
    let cli = AtCoderClient::new();
    let problem_id_err = move |url: &str| {
        cli.extract_problem_id(&Url::parse(url).unwrap())
            .unwrap_err()
    };
    use problem_id::Error;
    assert!(matches!(
        problem_id_err("https://atcoder.jp/contests/abc334"),
        Error::NotProblemUrl(_, Platform::AtCoder)
    ));
    assert!(matches!(
        problem_id_err("https://atcoder.com/contests/abc334"),
        Error::UnknownOrigin(_)
    ));
}

#[tokio::test]
async fn should_be_problem_url() {
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
    assert!(is_problem_url(
        "https://atcoder.jp/contests/tenka1-2012-qualB/tasks/tenka1_2012_7"
    ));
}

#[tokio::test]
async fn should_not_be_problem_url() {
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
    let url = Url::parse("https://atcoder.jp/contests/abc001").unwrap();
    let cli = AtCoderClient::new();
    let info = cli.fetch_contest_info(&url).await.unwrap();

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
                url: Url::parse("https://atcoder.jp/contests/abc001/tasks/abc001_1").unwrap(),
                ord: "A".to_owned(),
            },
            ContestProblemOutline {
                url: Url::parse("https://atcoder.jp/contests/abc001/tasks/abc001_2").unwrap(),
                ord: "B".to_owned(),
            },
            ContestProblemOutline {
                url: Url::parse("https://atcoder.jp/contests/abc001/tasks/abc001_3").unwrap(),
                ord: "C".to_owned(),
            },
            ContestProblemOutline {
                url: Url::parse("https://atcoder.jp/contests/abc001/tasks/abc001_4").unwrap(),
                ord: "D".to_owned(),
            },
        ]
    )
}

#[tokio::test]
async fn fetch_problems_virtual_contest_info_あさかつ6月8日() {
    let url = Url::parse(
        "https://kenkoooo.com/atcoder/#/contest/show/10f53d61-58e1-411e-9bfe-5fd0690a31f7",
    )
    .unwrap();
    let cli = AtCoderClient::new();
    let info = cli.fetch_contest_info(&url).await.unwrap();

    assert_eq!(info.url, url);
    assert_eq!(info.short_title, "AtCoderProblems_10f53d61");
    assert_eq!(info.long_title, "あさかつ6/8");
    assert_eq!(
        info.start_at,
        Local.with_ymd_and_hms(2023, 6, 8, 7, 30, 0).unwrap()
    );
    assert_eq!(
        info.end_at,
        Local.with_ymd_and_hms(2023, 6, 8, 8, 30, 0).unwrap()
    );
    assert_eq!(info.problems.len(), 6);

    assert_eq!(
        info.problems[0],
        ContestProblemOutline {
            url: Url::parse("https://atcoder.jp/contests/abc183/tasks/abc183_b").unwrap(),
            ord: "1".to_owned(),
        },
    );
    assert_eq!(
        info.problems[1],
        ContestProblemOutline {
            url: Url::parse("https://atcoder.jp/contests/tenka1_2017/tasks/tenka1_2017_b").unwrap(),
            ord: "2".to_owned(),
        },
    );
    assert_eq!(
        info.problems[2],
        ContestProblemOutline {
            url: Url::parse("https://atcoder.jp/contests/abc253/tasks/abc253_c").unwrap(),
            ord: "3".to_owned(),
        },
    );
    assert_eq!(
        info.problems[3],
        ContestProblemOutline {
            url: Url::parse("https://atcoder.jp/contests/arc051/tasks/arc051_b").unwrap(),
            ord: "4".to_owned(),
        },
    );
    assert_eq!(
        info.problems[4],
        ContestProblemOutline {
            url: Url::parse("https://atcoder.jp/contests/arc062/tasks/arc062_a").unwrap(),
            ord: "5".to_owned(),
        },
    );
    assert_eq!(
        info.problems[5],
        ContestProblemOutline {
            url: Url::parse("https://atcoder.jp/contests/agc060/tasks/agc060_b").unwrap(),
            ord: "6".to_owned(),
        },
    );
}

#[tokio::test]
async fn fetch_problem_detail_abc003_4() {
    let url_str = "https://atcoder.jp/contests/abc003/tasks/abc003_4";
    let url = Url::parse(url_str).unwrap();
    let cli = AtCoderClient::new();
    let (problem_info, testcases) = cli.fetch_problem_detail(&url).await.unwrap();

    assert_eq!(
        problem_info,
        ProblemInfo {
            platform: Platform::AtCoder,
            url: url.clone(),
            problem_id: ProblemId::try_from(&url).unwrap(),
            title: "AtCoder社の冬".to_owned(),
            execution_time_limit: Duration::from_secs(2),
            memory_limit_kb: 64 * 1024,
        },
    );
    assert_eq!(
        testcases,
        vec![
            SampleTestcase {
                ord: 1,
                input: ["3 2", "2 2", "2 2\n"].join("\n"),
                output: "12\n".to_owned(),
            },
            SampleTestcase {
                ord: 2,
                input: ["4 5", "3 1", "3 0\n"].join("\n"),
                output: "10\n".to_owned(),
            },
            SampleTestcase {
                ord: 3,
                input: ["23 18", "15 13", "100 95\n"].join("\n"),
                output: "364527243\n".to_owned(),
            },
            SampleTestcase {
                ord: 4,
                input: ["30 30", "24 22", "145 132\n"].join("\n"),
                output: "976668549\n".to_owned(),
            },
        ]
    );
}

#[tokio::test]
async fn fetch_problem_detail_abc086_a() {
    let url_str = "https://atcoder.jp/contests/abs/tasks/abc086_a";
    let url = Url::parse(url_str).unwrap();
    let cli = AtCoderClient::new();
    let (problem_info, testcases) = cli.fetch_problem_detail(&url).await.unwrap();

    assert_eq!(
        problem_info,
        ProblemInfo {
            platform: Platform::AtCoder,
            url: url.clone(),
            problem_id: ProblemId::try_from(&url).unwrap(),
            title: "Product".to_owned(),
            execution_time_limit: Duration::from_secs(2),
            memory_limit_kb: 256 * 1024,
        }
    );
    assert_eq!(
        testcases,
        vec![
            SampleTestcase {
                ord: 1,
                input: "3 4\n".to_owned(),
                output: "Even\n".to_owned(),
            },
            SampleTestcase {
                ord: 2,
                input: "1 21\n".to_owned(),
                output: "Odd\n".to_owned(),
            },
        ]
    )
}

#[tokio::test]
async fn fetch_problem_detail_typical90_b_which_contains_empty_pre_tag() {
    let url_str = "https://atcoder.jp/contests/typical90/tasks/typical90_b/";
    let url = Url::parse(url_str).unwrap();
    let cli = AtCoderClient::new();
    let (problem_info, testcases) = cli.fetch_problem_detail(&url).await.unwrap();

    assert_eq!(
        problem_info,
        ProblemInfo {
            platform: Platform::AtCoder,
            url: url.clone(),
            problem_id: ProblemId::try_from(&url).unwrap(),
            title: "Encyclopedia of Parentheses（★3）".to_owned(),
            execution_time_limit: Duration::from_secs(2),
            memory_limit_kb: 1024 * 1024,
        }
    );
    assert_eq!(
        testcases,
        vec![
            SampleTestcase {
                ord: 1,
                input: "2\n".to_owned(),
                output: "()\n".to_owned(),
            },
            SampleTestcase {
                ord: 2,
                input: "3\n".to_owned(),
                output: "".to_owned(),
            },
            SampleTestcase {
                ord: 3,
                input: "4\n".to_owned(),
                output: "(())\n()()\n".to_owned(),
            },
            SampleTestcase {
                ord: 4,
                input: "10\n".to_owned(),
                output: "\
((((()))))
(((()())))
(((())()))
(((()))())
(((())))()
((()(())))
((()()()))
((()())())
((()()))()
((())(()))
((())()())
((())())()
((()))(())
((()))()()
(()((())))
(()(()()))
(()(())())
(()(()))()
(()()(()))
(()()()())
(()()())()
(()())(())
(()())()()
(())((()))
(())(()())
(())(())()
(())()(())
(())()()()
()(((())))
()((()()))
()((())())
()((()))()
()(()(()))
()(()()())
()(()())()
()(())(())
()(())()()
()()((()))
()()(()())
()()(())()
()()()(())
()()()()()
"
                .to_owned(),
            },
        ]
    )
}

#[tokio::test]
async fn serialize_auth_data() {
    let cli = AtCoderClient::new().with_auth(AuthCookie {
        session_id: Some("test_session_id".to_owned()),
    });
    let json = cli.export_authtoken_as_json();
    assert_eq!(json, r#"{"session_id":"test_session_id"}"#);
}

#[tokio::test]
async fn serialize_null_auth_data() {
    let cli = AtCoderClient::new();
    let json = cli.export_authtoken_as_json();
    assert_eq!(json, r#"{"session_id":null}"#);
}

static PYTHON: Lazy<PgLang> = Lazy::new(|| PgLang::new("Python (3.8.2)", "4006"));

const URL_ABS_ABC086_A: &str = "https://atcoder.jp/contests/abs/tasks/abc086_a";

async fn submit_abc086_a(cli: &AtCoderClient) -> Result<Url> {
    cli.submit(
        &Url::parse(URL_ABS_ABC086_A).unwrap(),
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
async fn fetch_language_list_ok() {
    let cli = {
        let cli = AtCoderClient::new();
        let TestConfig {
            atcoder_username: username,
            atcoder_password: password,
        } = TestConfig::from_env();
        cli.login(AtCoderCred { username, password }.into())
            .await
            .unwrap_or_else(|e| panic!("{:?}", e));
        cli
    };

    let langs = dbg!(cli.fetch_submittable_language_list().await).unwrap();
    assert!(langs.len() > 65);
    langs.iter().find(|x| x.name == "C++ (GCC 9.2.1)").unwrap();
    langs.iter().find(|x| x.name == "Python (3.8.2)").unwrap();
    langs.iter().find(|x| x.name == "PyPy3 (7.3.0)").unwrap();
    langs.iter().find(|x| x.name == "Rust (1.42.0)").unwrap();
    langs
        .iter()
        .find(|x| x.name == "Java (OpenJDK 1.8.0)")
        .unwrap();
}

#[tokio::test]
async fn senario_login_submit_logout() {
    let auth_json = {
        let cli1 = AtCoderClient::new();
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

    let cli2 = AtCoderClient::new();
    cli2.load_authtoken_json(&auth_json).unwrap();

    let submission_status_url = dbg!(submit_abc086_a(&cli2).await).unwrap();
    assert_eq!(
        submission_status_url,
        Url::parse("https://atcoder.jp/contests/abs/submissions/me").unwrap()
    );

    cli2.logout().await.unwrap_or_else(|e| panic!("{:?}", e));

    let json = cli2.export_authtoken_as_json();
    assert_eq!(json, r#"{"session_id":null}"#);
}

#[tokio::test]
async fn login_with_wrong_password_should_be_fail() {
    let cli = AtCoderClient::new();
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
    let cli = AtCoderClient::new();
    let err = submit_abc086_a(&cli).await.err().unwrap();
    match err {
        Error::NeedLogin { requested_url } => {
            assert_eq!(requested_url, URL_ABS_ABC086_A);
        }
        _ => panic!("Want ClientError::WrongCredential, but got {:?}", err),
    }
}
