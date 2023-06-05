use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TestConfig {
    pub atcoder_username: String,
    pub atcoder_password: String,
}

impl TestConfig {
    pub fn from_env() -> Self {
        envy::from_env::<Self>().expect("TestConfig::from_env(): Failed to load from env")
    }
}
