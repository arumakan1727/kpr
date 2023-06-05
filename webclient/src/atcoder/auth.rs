use ::serde::{Deserialize, Serialize};

use crate::CredMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthCookie {
    pub session_id: Option<String>,
}

impl Default for AuthCookie {
    fn default() -> Self {
        AuthCookie { session_id: None }
    }
}

impl AuthCookie {
    pub fn from_json(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn revoke(&mut self) {
        self.session_id = None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtCoderCred {
    pub username: String,
    pub password: String,
}

impl From<AtCoderCred> for CredMap {
    fn from(c: AtCoderCred) -> Self {
        let mut h = CredMap::new();
        h.insert("username", c.username);
        h.insert("password", c.password);
        h
    }
}
