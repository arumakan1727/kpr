use std::{collections::HashMap, fmt::Debug};

/// Credential field name.
/// e.g. "username", "password"
pub type CredName = &'static str;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CredFieldKind {
    Text,
    Password,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CredFieldMeta {
    pub name: CredName,
    pub kind: CredFieldKind,
}

/// Credential table.
/// e.g. `[ "username" => "Bob", "password" => "***" ]`
pub type CredMap = HashMap<CredName, String>;
