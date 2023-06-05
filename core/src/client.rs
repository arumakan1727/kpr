use std::{
    io,
    ops::{Deref, DerefMut},
    path::Path,
};

use anyhow::{anyhow, Context};
use kpr_webclient::{Platform, Url};

use crate::{config, fsutil::SingleFileDriver};

pub struct SessionPersistentClient {
    cli: Box<dyn kpr_webclient::Client>,
    authtoken_file: SingleFileDriver,
}

impl Deref for SessionPersistentClient {
    type Target = Box<dyn kpr_webclient::Client>;

    fn deref(&self) -> &Self::Target {
        &self.cli
    }
}

impl DerefMut for SessionPersistentClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cli
    }
}

impl SessionPersistentClient {
    pub fn new(p: Platform, save_dir: impl AsRef<Path>) -> Self {
        let savepath = save_dir.as_ref().join(config::authtoken_filename(p));

        let mut x = Self {
            cli: kpr_webclient::new_client(p),
            authtoken_file: SingleFileDriver::new(savepath),
        };

        x.load_authtoken_if_file_exists().unwrap_or_else(|e| {
            eprintln!("[Warn] Initializing SessionPersistenceClient: {}", e);
        });
        x
    }

    pub fn new_with_parse_url(
        url: &str,
        save_dir: impl AsRef<Path>,
    ) -> anyhow::Result<(Self, Url)> {
        let url =
            Url::parse(url).map_err(|e| anyhow!("Failed to parse as URL '{}': {}", url, e))?;
        let platform = kpr_webclient::detect_platform_from_url(&url).with_context(|| {
            format!(
                "Cannot detect platform from URL '{}'\n  Example of supported domain: atcoder.jp",
                url
            )
        })?;
        let cli = Self::new(platform, save_dir);
        Ok((cli, url))
    }

    pub fn load_authtoken_if_file_exists(&mut self) -> anyhow::Result<()> {
        use crate::fsutil::error::Error;

        match self.authtoken_file.read() {
            Err(Error::SingleIO(_msg, _path, err)) if err.kind() == io::ErrorKind::NotFound => {
                Ok(())
            }

            Ok(json) => self.cli.load_authtoken_json(&json).with_context(|| {
                format!(
                    "Invalid JSON '{}'",
                    self.authtoken_file.filepath.to_string_lossy(),
                )
            }),

            Err(err) => Err(anyhow!(err)),
        }
    }

    #[must_use]
    pub fn load_authtoken_from_storage(&mut self) -> anyhow::Result<()> {
        let json = self.authtoken_file.read()?;
        self.cli.load_authtoken_json(&json).with_context(|| {
            format!(
                "Invalid JSON '{}'",
                self.authtoken_file.filepath.to_string_lossy()
            )
        })
    }

    #[must_use]
    pub fn save_authtoken_to_storage(&self) -> anyhow::Result<()> {
        let json = self.cli.export_authtoken_as_json();
        self.authtoken_file.write(&json).map_err(|e| anyhow!(e))
    }

    #[must_use]
    pub fn remove_authtoken_from_storage(&self) -> anyhow::Result<()> {
        self.authtoken_file.remove().map_err(|e| anyhow!(e))
    }
}
