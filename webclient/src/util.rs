use crate::error::*;
use reqwest::{Response, StatusCode};

/// Complete given link into fully qualified URL removing trailling slash.
pub fn complete_url(link: &str, host: &str) -> String {
    let link = link.trim_end_matches("/");
    if link.starts_with("/") {
        format!("https://{}{}", host, link)
    } else {
        assert!(link.starts_with("https://"));
        link.to_owned()
    }
}

pub fn extract_302_location_header(
    resp: &Response,
    requested_url: impl Into<String>,
) -> Result<String> {
    let got = resp.status();
    let expected = StatusCode::FOUND;
    if got != expected {
        return Err(Error::UnexpectedResponseCode {
            got,
            expected,
            requested_url: requested_url.into(),
        });
    };
    let bytes = resp.headers().get("Location").unwrap();
    Ok(bytes.to_str().unwrap().to_owned())
}

pub fn starts_with_anyone<'a, S, I, T>(s: S, prefixes: I) -> bool
where
    S: AsRef<str>,
    I: IntoIterator<Item = &'a T>,
    T: AsRef<str> + 'a,
{
    let s = s.as_ref();
    prefixes
        .into_iter()
        .any(|prefix| s.starts_with(prefix.as_ref()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_complete_url() {
        let url = complete_url("/login", "atcoder.jp");
        assert_eq!(url, "https://atcoder.jp/login");

        // If `link` is already fully qualified, 2nd argument is ignored:
        let url = complete_url("https://atcoder.jp/login", "example.com");
        assert_eq!(url, "https://atcoder.jp/login");

        // Trailling slash will be removed:
        let url = complete_url("/login/", "atcoder.jp");
        assert_eq!(url, "https://atcoder.jp/login");
    }
}
