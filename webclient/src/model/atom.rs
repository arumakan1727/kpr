use serde::{Deserialize, Serialize};

pub use reqwest::Url;

pub type LocalDateTime = chrono::DateTime<chrono::Local>;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display, strum::EnumIter, Serialize, Deserialize,
)]
pub enum Platform {
    AtCoder,
}

impl Platform {
    pub const fn lowercase(&self) -> &'static str {
        use Platform::*;
        match self {
            AtCoder => "atcoder",
        }
    }
}
