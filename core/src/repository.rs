pub mod error {
    pub use crate::fsutil::error::*;
}

use std::path::{Path, PathBuf};

use kpr_webclient::{GlobalId, Platform, ProblemMeta, Testcase};

use self::error::Result;
use crate::fsutil;

#[derive(Debug, Clone, Copy)]
pub struct Vault<'a> {
    home: &'a Path,
}

#[derive(Debug, Clone, Copy)]
pub struct Workspace<'a> {
    pub home: &'a Path,
}

#[derive(Debug, Clone)]
pub struct ProblemVaultLocation {
    problem_dir: PathBuf,
}

impl ProblemVaultLocation {
    fn new(problem_dir: impl Into<PathBuf>) -> Self {
        Self {
            problem_dir: problem_dir.into(),
        }
    }

    pub fn dirpath(&self) -> &Path {
        &self.problem_dir
    }

    pub fn metadata_filepath(&self) -> PathBuf {
        self.problem_dir.join(Vault::PROBLEM_METADATA_FILENAME)
    }

    pub fn testcase_dirpath(&self) -> PathBuf {
        self.problem_dir.join(Vault::TESTCASE_DIR_NAME)
    }
}

impl<'v> Vault<'v> {
    pub const TESTCASE_DIR_NAME: &str = "testcase";
    pub const PROBLEM_METADATA_FILENAME: &str = ".problem.json";

    pub fn new(vault_home_dir: &'v Path) -> Self {
        assert!(vault_home_dir.is_absolute());
        Self {
            home: vault_home_dir,
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

    pub fn resolve_problem_dir(&self, p: Platform, problem_id: &GlobalId) -> ProblemVaultLocation {
        let dir = self.home.join(p.lowercase()).join(problem_id.as_ref());
        ProblemVaultLocation::new(dir)
    }

    pub fn save_problem_data<'a>(
        &self,
        meta: &ProblemMeta,
        ts: impl IntoIterator<Item = &'a Testcase>,
    ) -> Result<ProblemVaultLocation> {
        let loc = self.resolve_problem_dir(meta.platform, &meta.global_id);

        fsutil::write_json_with_mkdir(loc.metadata_filepath(), meta)?;

        let testcase_dir = loc.testcase_dirpath();
        for t in ts {
            let (infile, outfile) = Self::testcase_filename(t.ord);
            fsutil::write_with_mkdir(testcase_dir.join(infile), &t.input)?;
            fsutil::write_with_mkdir(testcase_dir.join(outfile), &t.expected)?;
        }
        Ok(loc)
    }

    pub fn load_problem_metadata(
        &self,
        plat: Platform,
        problem_id: &GlobalId,
    ) -> Result<(ProblemVaultLocation, ProblemMeta)> {
        let loc = self.resolve_problem_dir(plat, problem_id);
        let problem_meta = fsutil::read_json_with_deserialize(loc.metadata_filepath())?;
        Ok((loc, problem_meta))
    }
}

impl<'w> Workspace<'w> {
    pub fn new(workspace_home_dir: &'w Path) -> Self {
        Self {
            home: workspace_home_dir,
        }
    }

    pub fn load_problem_metadata(&self, plat: Platform, id: &GlobalId) -> Result<ProblemMeta> {
        let dir = self.resolve_problem_dir(plat, id);
        let filepath = dir.join(config::PROBLEM_METADATA_FILENAME);
        fsutil::read_json_with_deserialize(filepath)
    }
}
