use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime, Local};

use super::{error::Result, vault::ProblemVault};
use crate::fsutil::{self, OptCopyContents};

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

    pub fn new(problem_workspace_dir: impl AsRef<Path>) -> Self {
        Self {
            dir: problem_workspace_dir.as_ref().to_owned(),
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

    #[must_use]
    pub fn find_most_recently_modified_file(
        &self,
        filename_pattern: &::glob::Pattern,
    ) -> Result<PathBuf> {
        let mut ans_filepath = None;
        let mut max_modified = SystemTime::UNIX_EPOCH;

        for entry in fsutil::read_dir(&self.dir)?.filter_map(std::result::Result::ok) {
            let file_type = entry.file_type();
            let modified = entry.metadata().and_then(|meta| meta.modified());
            let (Ok(file_type), Ok(modified)) =  (file_type, modified) else {
                continue
            };
            if file_type.is_dir() {
                continue;
            }
            let filename = entry.file_name();
            if filename_pattern.matches(filename.to_string_lossy().as_ref()) {
                if max_modified < modified {
                    max_modified = modified;
                    ans_filepath = Some(entry.path());
                }
            }
        }
        match ans_filepath {
            Some(filepath) => Ok(filepath),
            None => Err(fsutil::Error::NoEntryMatchedGlob(
                filename_pattern.to_owned(),
                self.dir.to_owned(),
            )),
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
