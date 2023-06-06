// exported modules
pub mod error;
pub mod model;

// client impls
pub mod atcoder;

// re-exports
pub use atcoder::{AtCoderClient, AtCoderUrlAnalyzer};
pub use error::*;
pub use model::*;

pub fn new_client(platform: Platform) -> Box<dyn Client> {
    use Platform::*;
    match platform {
        AtCoder => Box::new(AtCoderClient::new()),
    }
}

/// ```
/// use kpr_webclient::{detect_platform, Platform, Url};
///
/// let platform = detect_platform("https://atcoder.jp");
/// assert_eq!(platform, Some(Platform::AtCoder));
///
/// let platform = detect_platform("https://atcoder.jp/contests/abc001/tasks/");
/// assert_eq!(platform, Some(Platform::AtCoder));
///
/// let platform = detect_platform("https://example.com");
/// assert_eq!(platform, None);
/// ```
pub fn detect_platform(url: impl TryInto<Url>) -> Option<Platform> {
    url.try_into()
        .ok()
        .as_ref()
        .and_then(detect_platform_from_url)
}

pub fn detect_platform_from_url(url: &Url) -> Option<Platform> {
    if AtCoderUrlAnalyzer::is_supported_url(url) {
        Some(Platform::AtCoder)
    } else {
        None
    }
}

// internal modules
mod util;
