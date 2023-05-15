use crate::errors::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub(crate) struct TestConfig {
    pub atcoder_username: String,
    pub atcoder_password: String,
}

impl TestConfig {
   pub fn from_env() -> Result<Self> {
        envy::from_env::<Self>().context("TestConfig::from_env(): Failed to load from env")
    }
}
