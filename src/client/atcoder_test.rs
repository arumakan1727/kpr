use chrono::Local;
use chrono::TimeZone;

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
        "https://atcoder.jp/contests/typical90/tasks/typical90_a/"
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
