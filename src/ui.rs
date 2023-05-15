use std::io;

use dialoguer::{theme::ColorfulTheme, Input, Password};

pub fn theme() -> ColorfulTheme {
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
