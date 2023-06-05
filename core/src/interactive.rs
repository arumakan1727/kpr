use kpr_webclient::{CredFieldKind, CredFieldMeta, CredMap};

pub mod util {
    use dialoguer::{theme::ColorfulTheme, Input, Password};
    use std::io;

    fn theme() -> ColorfulTheme {
        ColorfulTheme::default()
    }

    pub fn ask_text(prompt: &str) -> io::Result<String> {
        Input::with_theme(&theme())
            .with_prompt(prompt)
            .interact_text()
    }

    pub fn ask_password(prompt: &str) -> io::Result<String> {
        Password::with_theme(&theme())
            .with_prompt(prompt)
            .interact()
    }
}

pub fn ask_credential(fields: &[CredFieldMeta]) -> CredMap {
    let mut map = CredMap::new();

    for CredFieldMeta { name, kind } in fields {
        use CredFieldKind::*;

        let value = match kind {
            Text => util::ask_text(name),
            Password => util::ask_password(name),
        }
        .unwrap_or_else(|e| panic!("{:?}", e));

        map.insert(name, value);
    }
    map
}
