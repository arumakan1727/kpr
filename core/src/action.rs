use std::path::Path;

pub mod error {
    #[allow(unused_imports)]
    pub(crate) use anyhow::{anyhow, bail, ensure, Context as _};
    pub use anyhow::{Error, Result};
}

use crate::{interactive::ask_credential, storage};
use error::*;
use kpr_webclient::Client;

type DynClient = Box<dyn Client>;

pub async fn login<P>(cli: &mut DynClient, authtoken_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    ensure!(
        !cli.is_logged_in(),
        anyhow!("Already logged in to {}", cli.platform())
    );

    let cred = ask_credential(cli.credential_fields());

    cli.login(cred)
        .await
        .with_context(|| format!("Failed to login to {}", cli.platform()))?;

    storage::save_authtoken(&cli, authtoken_dir).context("Failed to save login authtoken")
}

pub async fn logout<P>(cli: &mut DynClient, authtoken_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    ensure!(
        cli.is_logged_in(),
        anyhow!("Already logged out from {}", cli.platform())
    );

    let _ = storage::erase_authtoken(cli.platform(), authtoken_dir);

    cli.logout()
        .await
        .with_context(|| format!("Failed to logout from {}", cli.platform()))
}
