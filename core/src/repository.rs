pub mod error {
    pub use crate::fsutil::error::*;
}

use std::path::{Path, PathBuf};

use kpr_webclient::{GlobalId, Platform, ProblemMeta, Testcase};

use self::error::Result;
use crate::{config, fsutil};

pub struct Vault<'a> {
    pub home: &'a Path,
}

impl<'v> Vault<'v> {
    pub fn new(home_dir: &'v Path) -> Self {
        Self { home: home_dir }
    }

    /// Returns tuple (input_filename, output_filename).
    ///
    /// ```
    /// use kpr_core::repository::Vault;
    ///
    /// let (infile, outfile) = Vault::testcase_filename(1);
    /// assert_eq!(infile, "in1.txt");
    /// assert_eq!(outfile, "out1.txt");
    /// ```
    pub fn testcase_filename(ord: u32) -> (String, String) {
        (format!("in{}.txt", ord), format!("out{}.txt", ord))
    }

    pub fn resolve_problem_dir(&self, p: Platform, problem_id: &GlobalId) -> PathBuf {
        self.home.join(p.lowercase()).join(problem_id.as_ref())
    }

    pub fn save_testcase(&self, t: &Testcase, p: Platform, problem_id: &GlobalId) -> Result<()> {
        let dir = self.resolve_problem_dir(p, problem_id);
        let (infile, outfile) = Self::testcase_filename(t.ord);
        fsutil::write_with_mkdir(dir.join(&infile), &t.input)?;
        fsutil::write_with_mkdir(dir.join(&outfile), &t.expected)?;
        Ok(())
    }

    pub fn save_testcases<'a>(
        &self,
        ts: impl IntoIterator<Item = &'a Testcase>,
        plat: Platform,
        problem_id: &GlobalId,
    ) -> Result<()> {
        for t in ts {
            self.save_testcase(t, plat, problem_id)?;
        }
        Ok(())
    }

    pub fn save_problem_metadata(&self, data: &ProblemMeta) -> Result<()> {
        let dir = self.resolve_problem_dir(data.platform, &data.global_id);
        let filepath = dir.join(config::PROBLEM_METADATA_FILENAME);
        fsutil::write_json_with_mkdir(filepath, data)
    }

    pub fn load_problem_metadata(&self, plat: Platform, id: &GlobalId) -> Result<ProblemMeta> {
        let dir = self.resolve_problem_dir(plat, id);
        let filepath = dir.join(config::PROBLEM_METADATA_FILENAME);
        fsutil::read_json_with_deserialize(filepath)
    }
}
