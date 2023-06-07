use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

use indicatif::ProgressBar;
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

pub trait SpinnerExt {
    fn with_ticking(self) -> Arc<Mutex<Self>>;
}

impl SpinnerExt for ProgressBar {
    fn with_ticking(self) -> Arc<Mutex<Self>> {
        let mutex_spinner = Arc::new(Mutex::new(self));
        let spinner = mutex_spinner.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(30)).await;
                let spinner = spinner.lock().await;
                if spinner.is_finished() {
                    break;
                }
                spinner.tick();
            }
        });
        mutex_spinner
    }
}
