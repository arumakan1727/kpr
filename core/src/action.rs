pub mod error {
    #[allow(unused_imports)]
    pub(crate) use anyhow::{anyhow, bail, ensure, Context as _};
    pub use anyhow::{Error, Result};
}
use std::path::Path;

use error::*;
use kpr_webclient::Url;

use crate::client::SessionPersistentClient;
use crate::interactive::ask_credential;
use crate::storage;

pub async fn login(cli: &mut SessionPersistentClient) -> Result<()> {
    ensure!(
        !cli.is_logged_in(),
        "Already logged in to {}",
        cli.platform()
    );

    let cred = ask_credential(cli.credential_fields());

    cli.login(cred)
        .await
        .with_context(|| format!("Failed to login to {}", cli.platform()))?;

    cli.save_authtoken_to_storage()
}

pub async fn logout(cli: &mut SessionPersistentClient) -> Result<()> {
    ensure!(
        cli.is_logged_in(),
        "Already logged out from {}",
        cli.platform()
    );

    let _ = cli.remove_authtoken_from_storagr();

    cli.logout()
        .await
        .with_context(|| format!("Failed to logout from {}", cli.platform()))
}

pub async fn save_problem_data(
    cli: &SessionPersistentClient,
    url: &Url,
    dir: impl AsRef<Path>,
    testcase_dir_name: &str,
) -> Result<()> {
    ensure!(cli.is_problem_url(url), "{} is not a problem url", url);

    let problem_dir = dir
        .as_ref()
        .join(cli.platform().lowercase())
        .join(cli.get_problem_id(url.path()).unwrap());

    let testcase_dir = problem_dir.join(testcase_dir_name);

    let testcases = cli
        .fetch_testcases(url)
        .await
        .context("Failed to fetch testcase")?;

    storage::save_testcases(testcases.iter(), &testcase_dir).context("Failed to save testcase")?;
    storage::save_problem_url(url, &problem_dir).context("Failed to save problem url")?;

    Ok(())
}
