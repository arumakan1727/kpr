use std::path::{Path, PathBuf};

use kpr_webclient::{Platform, ProblemId, ProblemMeta, Testcase};

use super::error::Result;
use crate::fsutil;

#[derive(Debug, Clone, Copy)]
pub struct VaultHome<'a> {
    home: &'a Path,
}

#[derive(Debug, Clone)]
pub struct ProblemVault {
    dir: PathBuf,
}

impl ProblemVault {
    const TESTCASE_DIR_NAME: &str = "testcase";
    const PROBLEM_METADATA_FILENAME: &str = "problem.json";

    pub fn new(problem_vault_dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: problem_vault_dir.into(),
        }
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

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn metadata_file(&self) -> PathBuf {
        self.dir.join(Self::PROBLEM_METADATA_FILENAME)
    }

    pub fn testcase_dir(&self) -> PathBuf {
        self.dir.join(Self::TESTCASE_DIR_NAME)
    }
}

impl<'v> VaultHome<'v> {
    #[inline]
    pub fn new(vault_home_dir: &'v Path) -> Self {
        Self {
            home: vault_home_dir,
        }
    }

    pub fn resolve_problem_dir(&self, p: Platform, problem_id: &ProblemId) -> ProblemVault {
        let dir = self.home.join(p.lowercase()).join(&problem_id);
        ProblemVault::new(dir)
    }

    #[must_use]
    pub fn save_problem_data<'a>(
        &self,
        meta: &ProblemMeta,
        ts: impl IntoIterator<Item = &'a Testcase>,
    ) -> Result<ProblemVault> {
        let loc = self.resolve_problem_dir(meta.platform, &meta.problem_id);

        fsutil::write_json_with_mkdir(loc.metadata_file(), meta)?;

        let testcase_dir = loc.testcase_dir();
        fsutil::mkdir_all(&testcase_dir)?;
        for t in ts {
            let (infile, outfile) = ProblemVault::testcase_filename(t.ord);
            fsutil::write(testcase_dir.join(infile), &t.input)?;
            fsutil::write(testcase_dir.join(outfile), &t.expected)?;
        }
        Ok(loc)
    }

    #[must_use]
    pub fn load_problem_metadata(
        &self,
        plat: Platform,
        problem_id: &ProblemId,
    ) -> Result<(ProblemVault, ProblemMeta)> {
        let loc = self.resolve_problem_dir(plat, problem_id);
        let problem_meta = fsutil::read_json_with_deserialize(loc.metadata_file())?;
        Ok((loc, problem_meta))
    }
}
