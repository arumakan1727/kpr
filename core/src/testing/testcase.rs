use std::{
    fs::File,
    io::{Cursor, Read},
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use async_trait::async_trait;
use tokio::{fs::File as TokioFile, io::AsyncRead};

pub trait Testcase<'a> {
    type Reader: Read;
    fn name(&self) -> &str;
    fn new_input_reader(&'a self) -> anyhow::Result<Self::Reader>;
    fn new_groundtruth_reader(&'a self) -> anyhow::Result<Self::Reader>;
}

#[async_trait]
pub trait AsyncTestcase<'a> {
    type Reader: AsyncRead + Unpin;
    fn name(&self) -> &str;
    async fn new_input_reader(&'a self) -> anyhow::Result<Self::Reader>;
    async fn new_groundtruth_reader(&'a self) -> anyhow::Result<Self::Reader>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FsTestcase {
    name: String,
    input_data_path: PathBuf,
    groundtruth_data_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnMemoryTestcase<B: AsRef<[u8]>> {
    pub name: String,
    pub input: B,
    pub groundtruth: B,
}

impl<'a> Testcase<'a> for FsTestcase {
    type Reader = File;

    fn name(&self) -> &str {
        &self.name
    }

    fn new_input_reader(&self) -> anyhow::Result<Self::Reader> {
        File::open(&self.input_data_path).with_context(|| {
            format!(
                "Failed to read testcase {}",
                self.input_data_path.to_string_lossy(),
            )
        })
    }

    fn new_groundtruth_reader(&self) -> anyhow::Result<Self::Reader> {
        File::open(&self.groundtruth_data_path).with_context(|| {
            format!(
                "Failed to read testcase {}",
                self.input_data_path.to_string_lossy(),
            )
        })
    }
}

#[async_trait]
impl<'a> AsyncTestcase<'a> for FsTestcase {
    type Reader = TokioFile;

    fn name(&self) -> &str {
        &self.name
    }

    async fn new_input_reader(&'a self) -> anyhow::Result<TokioFile> {
        TokioFile::open(&self.input_data_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to read testcase {}",
                    self.input_data_path.to_string_lossy(),
                )
            })
    }

    async fn new_groundtruth_reader(&'a self) -> anyhow::Result<TokioFile> {
        TokioFile::open(&self.groundtruth_data_path)
            .await
            .with_context(|| {
                format!(
                    "Failed to read testcase {}",
                    self.groundtruth_data_path.to_string_lossy(),
                )
            })
    }
}

pub trait FsTestcaseFinder {
    fn find_by_input_file_path(&self, path: impl AsRef<Path>) -> Option<FsTestcase>;
}

impl FsTestcase {
    pub fn new(
        name: impl Into<String>,
        input: impl Into<PathBuf>,
        output: impl Into<PathBuf>,
    ) -> Self {
        Self {
            name: name.into(),
            input_data_path: input.into(),
            groundtruth_data_path: output.into(),
        }
    }

    pub fn enumerate(
        dir: impl AsRef<Path>,
        finder: &impl FsTestcaseFinder,
    ) -> fsutil::Result<Vec<Self>> {
        let mut res = Vec::new();
        for entry in fsutil::read_dir(&dir)?.filter_map(Result::ok) {
            let Ok(ft) =  entry.file_type() else {
                    continue
                };
            if ft.is_dir() {
                continue;
            }
            if let Some(t) = finder.find_by_input_file_path(entry.path()) {
                res.push(t)
            }
        }
        res.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(res)
    }
}

impl<B> OnMemoryTestcase<B>
where
    B: AsRef<[u8]>,
{
    pub fn new(name: impl Into<String>, input: impl Into<B>, groundtruth: impl Into<B>) -> Self {
        Self {
            name: name.into(),
            input: input.into(),
            groundtruth: groundtruth.into(),
        }
    }
}

impl<'a, B> Testcase<'a> for OnMemoryTestcase<B>
where
    B: AsRef<[u8]>,
{
    type Reader = Cursor<&'a [u8]>;

    fn name(&self) -> &str {
        &self.name
    }

    fn new_input_reader(&'a self) -> anyhow::Result<Self::Reader> {
        Ok(Cursor::new(self.input.as_ref()))
    }

    fn new_groundtruth_reader(&'a self) -> anyhow::Result<Self::Reader> {
        Ok(Cursor::new(self.groundtruth.as_ref()))
    }
}

#[async_trait]
impl<'a, B> AsyncTestcase<'a> for OnMemoryTestcase<B>
where
    B: AsRef<[u8]> + Sync,
{
    type Reader = Cursor<&'a [u8]>;

    fn name(&self) -> &str {
        &self.name
    }

    async fn new_input_reader(&'a self) -> anyhow::Result<Self::Reader> {
        Ok(Cursor::new(self.input.as_ref()))
    }

    async fn new_groundtruth_reader(&'a self) -> anyhow::Result<Self::Reader> {
        Ok(Cursor::new(self.groundtruth.as_ref()))
    }
}
