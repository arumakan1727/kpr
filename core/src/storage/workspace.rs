use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};

use super::{error::Result, vault::ProblemVault};
use crate::{
    fsutil::{self, OptCopyContents},
    testing::{FsTestcase, FsTestcaseFinder},
};

#[derive(Debug, Clone, Copy)]
pub struct WorkspaceHome<'a> {
    home: &'a Path,
}

#[derive(Debug, Clone)]
pub struct ProblemWorkspace {
    dir: PathBuf,
}

pub struct WorkspaceNameModifier<'categ, 'name> {
    pub today: DateTime<Local>,
    pub category: &'categ str,
    pub name: &'name str,
}

impl ProblemWorkspace {
    const TESTCASE_DIR_NAME: &str = "testcase";
    const PROBLEM_METADATA_FILENAME: &str = ".problem.json";

    pub fn new(problem_workspace_dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: problem_workspace_dir.into(),
        }
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

    pub fn find_most_recently_modified_file(
        &self,
        filename_pattern: &::glob::Pattern,
    ) -> Result<PathBuf> {
        fsutil::find_most_recently_modified_file(&self.dir, filename_pattern)
    }
}

pub struct TestcaseFinder;

impl FsTestcaseFinder for TestcaseFinder {
    /// if and only if `path` matches "in{...}.txt", find "out{...}.txt" and return them as FsTestcase
    fn find_by_input_file_path(&self, path: impl AsRef<Path>) -> Option<FsTestcase> {
        let in_file_path = path.as_ref();
        let in_file_name = in_file_path.file_name()?.to_string_lossy();

        let tail = in_file_name.strip_prefix("in")?;
        let name = tail
            .strip_suffix(".txt")?
            .trim_matches(|c| c == '_' || c == '-');

        let out_file_path = in_file_path.with_file_name(format!("out{}", tail));

        if in_file_path.is_file() && out_file_path.is_file() {
            Some(FsTestcase::new(name, in_file_path, out_file_path))
        } else {
            None
        }
    }
}

impl<'w> WorkspaceHome<'w> {
    #[inline]
    pub fn new(workspace_home_dir: &'w Path) -> Self {
        Self {
            home: workspace_home_dir,
        }
    }

    pub fn resolve_problem_dir(&self, m: WorkspaceNameModifier) -> ProblemWorkspace {
        let yyyy = m.today.format("%Y").to_string();
        let mmdd_a = m.today.format("%m%d-%a").to_string();
        ProblemWorkspace::new(
            self.home
                .join(yyyy)
                .join(mmdd_a)
                .join(m.category)
                .join(m.name),
        )
    }

    #[must_use]
    pub fn create_workspace(
        &self,
        vault: &ProblemVault,
        template_dir: impl AsRef<Path>,
        name_modifier: WorkspaceNameModifier,
    ) -> Result<ProblemWorkspace> {
        let workspace = self.resolve_problem_dir(name_modifier);
        fsutil::symlink_using_relpath_with_mkdir(vault.metadata_file(), workspace.metadata_file())?;
        fsutil::symlink_using_relpath_with_mkdir(vault.testcase_dir(), workspace.testcase_dir())?;
        fsutil::copy_contents_all(
            template_dir,
            workspace.dir(),
            &OptCopyContents {
                overwrite_existing_file: false,
            },
        )?;
        Ok(workspace)
    }
}
