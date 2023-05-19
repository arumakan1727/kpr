use std::path::Path;

use kpr_webclient::{Client, Platform};

use super::{error::*, util};
use crate::config;

#[must_use]
pub fn save_authtoken(cli: &Box<dyn Client>, dir: impl AsRef<Path>) -> Result<()> {
    let dir = dir.as_ref();
    let filepath = dir.join(config::authtoken_filename(cli.platform()));
    let contents = cli.export_authtoken_as_json();
    util::write_with_mkdir(filepath, contents)
}

#[must_use]
pub fn load_authtoken(cli: &mut Box<dyn Client>, dir: impl AsRef<Path>) -> Result<()> {
    let dir = dir.as_ref();
    let filepath = dir.join(config::authtoken_filename(cli.platform()));
    let contents = util::read_to_string(&filepath)?;
    cli.load_authtoken_json(&contents).map_err(|e| Error {
        action: ActionKind::DeserializeAuthtokenJson,
        path: filepath,
        source: Box::new(e),
    })
}

#[must_use]
pub fn erase_authtoken(platform: Platform, dir: impl AsRef<Path>) -> Result<()> {
    let dir = dir.as_ref();
    let filepath = dir.join(config::authtoken_filename(platform));
    util::remove_file(filepath)
}
