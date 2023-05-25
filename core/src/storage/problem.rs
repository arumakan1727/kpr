use std::path::Path;

use kpr_webclient::Testcase;

use crate::config;

use super::{error::*, util};

pub fn save_testcase(t: &Testcase, dir: impl AsRef<Path>) -> Result<()> {
    let dir = dir.as_ref();
    let (infile, outfile) = config::testcase_filename(t.ord);
    util::write_with_mkdir(dir.join(&infile), &t.input)?;
    util::write_with_mkdir(dir.join(&outfile), &t.expected)?;
    Ok(())
}

pub fn save_testcases<'a>(
    ts: impl Iterator<Item = &'a Testcase>,
    dir: impl AsRef<Path>,
) -> Result<()> {
    for t in ts {
        save_testcase(t, &dir)?;
    }
    Ok(())
}

pub fn save_problem_url(url: impl AsRef<str>, dir: impl AsRef<Path>) -> Result<()> {
    let filepath = dir.as_ref().join(config::PROBLEM_URL_FILENAME);
    util::write_with_mkdir(filepath, url.as_ref())
}
