use std::path::{Path, PathBuf};

use crate::fsutil::{self, OptCopyContents};

use super::{error::Result, vault::ProblemVaultLocation};

#[derive(Debug, Clone, Copy)]
pub struct Workspace<'a> {
    home: &'a Path,
}

#[derive(Debug, Clone)]
pub struct ProblemWorkspaceLocation {
    problem_dir: PathBuf,
}

impl ProblemWorkspaceLocation {
    fn new(problem_dir: impl Into<PathBuf>) -> Self {
        Self {
            problem_dir: problem_dir.into(),
        }
    }

    pub fn dirpath(&self) -> &Path {
        &self.problem_dir
    }

    pub fn metadata_filepath(&self) -> PathBuf {
        self.problem_dir.join(Workspace::PROBLEM_METADATA_FILENAME)
    }

    pub fn testcase_dirpath(&self) -> PathBuf {
        self.problem_dir.join(Workspace::TESTCASE_DIR_NAME)
    }
}

impl<'w> Workspace<'w> {
    const TESTCASE_DIR_NAME: &str = "testcase";
    const PROBLEM_METADATA_FILENAME: &str = ".problem.json";

    #[inline]
    pub fn new(workspace_home_dir: &'w Path) -> Self {
        Self {
            home: workspace_home_dir,
        }
    }

    #[must_use]
    pub fn create_workspace(
        &self,
        prefix: impl AsRef<Path>,
        vault: &ProblemVaultLocation,
        template_dir: impl AsRef<Path>,
    ) -> Result<ProblemWorkspaceLocation> {
        let workspace = ProblemWorkspaceLocation::new(self.home.join(prefix));
        fsutil::symlink_using_relpath_with_mkdir(
            vault.metadata_filepath(),
            workspace.metadata_filepath(),
        )?;
        fsutil::symlink_using_relpath_with_mkdir(
            vault.testcase_dirpath(),
            workspace.testcase_dirpath(),
        )?;
        fsutil::copy_contents_all(
            template_dir,
            workspace.dirpath(),
            &OptCopyContents {
                overwrite_existing_file: false,
            },
        )?;
        Ok(workspace)
    }
}
