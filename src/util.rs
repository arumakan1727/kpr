use crate::errors::*;
use reqwest::{Response, StatusCode};

/// Complete given link into fully qualified URL removing trailling slash.
///
/// ## Examples
/// ```
/// use kyopro_cli::util;
///
/// let url = util::complete_url("/login", "atcoder.jp");
/// assert_eq!(url, "https://atcoder.jp/login");
///
/// // If `link` is already fully qualified, 2nd argument is ignored:
/// let url = util::complete_url("https://atcoder.jp/login", "example.com");
/// assert_eq!(url, "https://atcoder.jp/login");
///
/// // Trailling slash will be removed:
/// let url = util::complete_url("/login/", "atcoder.jp");
/// assert_eq!(url, "https://atcoder.jp/login");
/// ```
pub fn complete_url(link: &str, host: &str) -> String {
    let link = link.trim_end_matches("/");
    if link.starts_with("/") {
        format!("https://{}{}", host, link)
    } else {
        assert!(link.starts_with("https://"));
        link.to_owned()
    }
}

pub fn extract_location_header(resp: &Response, expected: StatusCode) -> Result<String> {
    let got = resp.status();
    ensure!(
        got == expected,
        "Unexpected response code: {} (expected {})",
        got,
        expected
    );
    let bytes = resp.headers().get("Location").unwrap();
    Ok(bytes.to_str().unwrap().to_owned())
}
