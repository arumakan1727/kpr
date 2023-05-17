pub mod client;
pub mod cmd;
pub mod config;
pub mod ui;
pub mod util;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, clap::ValueEnum, strum::Display, strum::EnumIter,
)]
#[clap(rename_all = "lower")]
pub enum Platform {
    AtCoder,
}

#[cfg(test)]
mod testconfig;

/// The error types used through out this crate.
pub mod errors {
    #[allow(unused_imports)]
    pub(crate) use anyhow::{anyhow, bail, ensure, Context as _};
    pub use anyhow::{Error, Result};
}
