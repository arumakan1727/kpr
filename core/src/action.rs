pub mod error {
    #[allow(unused_imports)]
    pub(crate) use anyhow::{anyhow, bail, ensure, Context as _};
    pub use anyhow::{Error, Result};
}
use error::*;

use crate::client::SessionPersistentClient as Client;
use crate::interactive::ask_credential;

pub async fn login(cli: &mut Client) -> Result<()> {
    ensure!(
        !cli.is_logged_in(),
        anyhow!("Already logged in to {}", cli.platform())
    );

    let cred = ask_credential(cli.credential_fields());

    cli.login(cred)
        .await
        .with_context(|| format!("Failed to login to {}", cli.platform()))?;

    cli.save_authtoken_to_storage()
}

pub async fn logout(cli: &mut Client) -> Result<()> {
    ensure!(
        cli.is_logged_in(),
        anyhow!("Already logged out from {}", cli.platform())
    );

    let _ = cli.remove_authtoken_from_storagr();

    cli.logout()
        .await
        .with_context(|| format!("Failed to logout from {}", cli.platform()))
}
