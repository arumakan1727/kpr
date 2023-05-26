pub mod error {
    pub use crate::fsutil::error::*;
}

use std::path::Path;

use error::Result;
use kpr_webclient::{ProblemMeta, Testcase};

use crate::{config, fsutil};

pub fn save_testcase(t: &Testcase, dir: impl AsRef<Path>) -> Result<()> {
    let dir = dir.as_ref();
    let (infile, outfile) = config::testcase_filename(t.ord);
    fsutil::write_with_mkdir(dir.join(&infile), &t.input)?;
    fsutil::write_with_mkdir(dir.join(&outfile), &t.expected)?;
    Ok(())
}

pub fn save_testcases<'a>(
    ts: impl IntoIterator<Item = &'a Testcase>,
    dir: impl AsRef<Path>,
) -> Result<()> {
    for t in ts {
        save_testcase(t, &dir)?;
    }
    Ok(())
}

pub fn save_problem_metadata(data: &ProblemMeta, dir: impl AsRef<Path>) -> Result<()> {
    let filepath = dir.as_ref().join(config::PROBLEM_METADATA_FILENAME);
    fsutil::write_json_with_mkdir(filepath, data)
}

pub fn load_problem_metadata(dir: impl AsRef<Path>) -> Result<ProblemMeta> {
    let filepath = dir.as_ref().join(config::PROBLEM_METADATA_FILENAME);
    fsutil::read_json_with_deserialize(filepath)
}

pub fn exists_problem_data(dir: impl AsRef<Path>, testcase_dir_name: &str) -> bool {
    let dir = dir.as_ref();
    let metadata_filepath = dir.join(config::PROBLEM_METADATA_FILENAME);
    let testcase_dirpath = dir.join(testcase_dir_name);
    metadata_filepath.is_file() && testcase_dirpath.is_dir()
}
