use reqwest::StatusCode;
use scraper::{node::Element, ElementRef, Html, Selector};
use serde::de;
use url::Url;

use crate::error::*;
use crate::http::{Client, Response};

/// Complete given link into fully qualified URL removing trailling slash.
pub fn complete_url(link: impl AsRef<str>, host: impl AsRef<str>) -> Result<Url> {
    let link = link.as_ref().trim_end_matches("/");
    if link.starts_with("/") {
        self::parse_url(format!("https://{}{}", host.as_ref(), link))
    } else {
        assert!(link.starts_with("https://"));
        self::parse_url(link)
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

pub fn starts_with_oneof<'a, S, I, T>(s: S, prefixes: I) -> bool
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

pub fn parse_url(url: impl AsRef<str>) -> Result<Url> {
    match Url::parse(url.as_ref()) {
        Ok(url) => Ok(url),
        Err(e) => Err(Error::InvalidSyntaxUrl {
            url: url.as_ref().to_owned(),
            source: e,
        }),
    }
}

pub async fn fetch_html_with_parse_url(c: &Client, url: impl AsRef<str>) -> Result<Html> {
    let url = self::parse_url(url)?;
    self::fetch_html(c, url).await
}

pub async fn fetch_json_with_parse_url<T>(c: &Client, url: impl AsRef<str>) -> Result<T>
where
    T: de::DeserializeOwned,
{
    let url = self::parse_url(url)?;
    self::fetch_json(c, url).await
}

pub async fn fetch_text(c: &Client, url: Url) -> Result<String> {
    let url_str = url.to_string();
    let resp = c.get(url).send().await?;

    let status = resp.status();
    if status != StatusCode::OK {
        return Err(Error::UnexpectedResponseCode {
            got: status,
            expected: StatusCode::OK,
            requested_url: url_str,
        });
    }
    let s = resp.text().await?;
    Ok(s)
}

pub async fn fetch_html(c: &Client, url: Url) -> Result<Html> {
    let html = self::fetch_text(c, url).await?;
    Ok(Html::parse_document(&html))
}

pub async fn fetch_json<T>(c: &Client, url: Url) -> Result<T>
where
    T: de::DeserializeOwned,
{
    let json = self::fetch_text(c, url).await?;
    serde_json::from_str(&json).map_err(|e| Error::Json(e))
}

pub fn selector_must_parsed(sel: &'static str) -> Selector {
    Selector::parse(sel).expect("Failed to parse  `&'static str`  selector")
}

pub trait DocExt {
    fn select_first(&self, sel: &Selector) -> Result<ElementRef>;
    fn select_double(&self, sel: &Selector) -> Result<(ElementRef, ElementRef)>;
}

impl DocExt for Html {
    fn select_first(&self, sel: &Selector) -> Result<ElementRef> {
        match self.select(&sel).next() {
            Some(el) => Ok(el),
            None => Err(Error::NoSuchElementMatchesToSelector(sel.to_owned())),
        }
    }

    fn select_double(&self, sel: &Selector) -> Result<(ElementRef, ElementRef)> {
        let mut iter = self.select(&sel);

        let el1 = match iter.next() {
            Some(el) => el,
            None => return Err(Error::NoSuchElementMatchesToSelector(sel.to_owned())),
        };
        let el2 = match iter.next() {
            Some(el) => el,
            None => return Err(Error::NoSuchElementMatchesToSelector(sel.to_owned())),
        };

        Ok((el1, el2))
    }
}

impl<'a> DocExt for ElementRef<'a> {
    fn select_first(&self, sel: &Selector) -> Result<ElementRef> {
        match self.select(&sel).next() {
            Some(el) => Ok(el),
            None => Err(Error::NoSuchElementMatchesToSelector(sel.to_owned())),
        }
    }

    fn select_double(&self, sel: &Selector) -> Result<(ElementRef, ElementRef)> {
        let mut iter = self.select(&sel);

        let el1 = match iter.next() {
            Some(el) => el,
            None => return Err(Error::NoSuchElementMatchesToSelector(sel.to_owned())),
        };
        let el2 = match iter.next() {
            Some(el) => el,
            None => return Err(Error::NoSuchElementMatchesToSelector(sel.to_owned())),
        };

        Ok((el1, el2))
    }
}

pub trait ElementRefExt {
    fn first_text(&self, ctx_selector: &Selector) -> Result<&str>;
}

impl<'a> ElementRefExt for ElementRef<'a> {
    fn first_text(&self, ctx_selector: &Selector) -> Result<&str> {
        match self.text().next() {
            Some(s) => Ok(s),
            None => Err(Error::NoInnerText(ctx_selector.to_owned())),
        }
    }
}

pub trait ElementExt {
    fn get_attr(&self, name: &'static str, ctx_selector: &Selector) -> Result<&str>;
}

impl ElementExt for Element {
    fn get_attr(&self, name: &'static str, ctx_selector: &Selector) -> Result<&str> {
        match self.attr(name) {
            Some(value) => Ok(value),
            None => Err(Error::NoSuchAttr(name, ctx_selector.to_owned())),
        }
    }
}

pub fn is_invalid_char_for_path(c: char) -> bool {
    match c {
        '\\' | '/' | ':' | '*' | '!' | '?' | '"' | '\'' | '<' | '>' | '|' => true,
        _ => false,
    }
}

pub fn sanitize_for_path_str(s: impl AsRef<str>) -> String {
    return s
        .as_ref()
        .trim()
        .trim_matches(is_invalid_char_for_path)
        .replace(is_invalid_char_for_path, "-")
        .replace(char::is_whitespace, "_")
        .to_owned();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_complete_url() {
        let url = complete_url("/login", "atcoder.jp").unwrap();
        assert_eq!(url, Url::parse("https://atcoder.jp/login").unwrap());

        // If `link` is already fully qualified, 2nd argument is ignored:
        let url = complete_url("https://atcoder.jp/login", "example.com").unwrap();
        assert_eq!(url, Url::parse("https://atcoder.jp/login").unwrap());

        // Trailling slash will be removed:
        let url = complete_url("/login/", "atcoder.jp").unwrap();
        assert_eq!(url, Url::parse("https://atcoder.jp/login").unwrap());
    }
}
