use std::path::Path;

use kpr_webclient::{ProblemMeta, Testcase};

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
    util::write_json_with_mkdir(filepath, data)
}

pub fn load_problem_metadata(dir: impl AsRef<Path>) -> Result<ProblemMeta> {
    let filepath = dir.as_ref().join(config::PROBLEM_METADATA_FILENAME);
    util::read_json_with_deserialize(filepath)
}

pub fn exists_problem_data(dir: impl AsRef<Path>, testcase_dir_name: &str) -> bool {
    let dir = dir.as_ref();
    let metadata_filepath = dir.join(config::PROBLEM_METADATA_FILENAME);
    let testcase_dirpath = dir.join(testcase_dir_name);
    metadata_filepath.is_file() && testcase_dirpath.is_dir()
}
