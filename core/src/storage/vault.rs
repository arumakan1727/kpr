use std::path::{Path, PathBuf};

use kpr_webclient::{PgLang, Platform, ProblemId, ProblemMeta, SampleTestcase};

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

#[derive(Debug, Clone)]
pub struct PlatformVault {
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
    /// use kpr_core::storage::ProblemVault;
    ///
    /// let (infile, outfile) = ProblemVault::testcase_filename("sample1");
    /// assert_eq!(infile, "in_sample1.txt");
    /// assert_eq!(outfile, "out_sample1.txt");
    /// ```
    pub fn testcase_filename(name: &str) -> (String, String) {
        (format!("in_{}.txt", name), format!("out_{}.txt", name))
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

impl PlatformVault {
    const SUBMITTABLE_LANGS_FILENAME: &str = "submittable-langs.json";

    pub fn new(platform_vault_dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: platform_vault_dir.into(),
        }
    }

    pub fn submittable_langs_file(&self) -> PathBuf {
        self.dir.join(Self::SUBMITTABLE_LANGS_FILENAME)
    }
}

impl<'v> VaultHome<'v> {
    #[inline]
    pub fn new(vault_home_dir: &'v Path) -> Self {
        Self {
            home: vault_home_dir,
        }
    }

    pub fn resolve_platform_dir(&self, p: Platform) -> PlatformVault {
        let dir = self.home.join(p.lowercase());
        PlatformVault::new(dir)
    }

    pub fn resolve_problem_dir(&self, p: Platform, problem_id: &ProblemId) -> ProblemVault {
        let dir = self.home.join(p.lowercase()).join(&problem_id);
        ProblemVault::new(dir)
    }

    #[must_use]
    pub fn save_problem_data<'a>(
        &self,
        meta: &ProblemMeta,
        sample_testcases: impl IntoIterator<Item = &'a SampleTestcase>,
    ) -> Result<ProblemVault> {
        let loc = self.resolve_problem_dir(meta.platform, &meta.problem_id);

        fsutil::write_json_with_mkdir(loc.metadata_file(), meta)?;

        let testcase_dir = loc.testcase_dir();
        fsutil::mkdir_all(&testcase_dir)?;

        for t in sample_testcases {
            let name = format!("sample{}", t.ord);
            self.save_testcase(&loc, &name, &t.input, &t.output)?;
        }
        Ok(loc)
    }

    #[must_use]
    pub fn save_testcase(
        &self,
        location: &ProblemVault,
        name: &str,
        input: impl AsRef<[u8]>,
        output: impl AsRef<[u8]>,
    ) -> Result<()> {
        let dir = location.testcase_dir();
        let (infile, outfile) = ProblemVault::testcase_filename(name);
        fsutil::write(dir.join(infile), input)?;
        fsutil::write(dir.join(outfile), output)?;
        Ok(())
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

    #[must_use]
    pub fn save_submittable_lang_list(
        &self,
        plat: Platform,
        langs: &[PgLang],
    ) -> Result<PlatformVault> {
        let vault = self.resolve_platform_dir(plat);
        fsutil::write_json_with_mkdir(vault.submittable_langs_file(), &langs)?;
        Ok(vault)
    }

    #[must_use]
    pub fn load_submittable_lang_list(
        &self,
        plat: Platform,
    ) -> Result<(PlatformVault, Vec<PgLang>)> {
        let vault = self.resolve_platform_dir(plat);
        let langs: Vec<PgLang> =
            fsutil::read_json_with_deserialize(vault.submittable_langs_file())?;
        Ok((vault, langs))
    }
}
