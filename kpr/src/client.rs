use crate::config::Config;
use kpr_core::storage::{self, ActionKind};
use kpr_webclient::{Client, Platform};

pub fn new_client_with_authtoken_autoload(p: Platform, cfg: &Config) -> Box<dyn Client> {
    let mut cli = kpr_webclient::new_client(p);

    storage::load_authtoken(&mut cli, &cfg.cache_dir).unwrap_or_else(|err| {
        // ファイルが存在しなかった場合は無視
        if err.action != ActionKind::ReadFile {
            eprintln!("[Warn] {}", err)
        }
    });

    cli
}
