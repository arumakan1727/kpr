use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use anyhow::{bail, Context};
use tokio::process::Command;

use super::{result::*, testcase::*};
use crate::str_interp::{interp, InterpError};

#[derive(Debug, Clone)]
pub struct TestCommand {
    pub compile: Option<String>,
    pub run: String,
}

#[derive(Debug, Clone)]
pub struct TestRunner {
    cmd: TestCommand,
    shell: PathBuf,
    execution_time_limit: Duration,
}

impl TestRunner {
    const DEFAULT_SHELL: &str = "/bin/sh";
    const DEFAULT_EXEC_TIME_LIMIT: Duration = Duration::from_millis(1000);

    pub fn new(cmd: TestCommand) -> Self {
        Self {
            cmd,
            shell: Self::DEFAULT_SHELL.into(),
            execution_time_limit: Self::DEFAULT_EXEC_TIME_LIMIT,
        }
    }

    pub fn shell(mut self, shell: impl Into<PathBuf>) -> Self {
        self.shell = shell.into();
        self
    }

    pub fn execution_time_limit(mut self, limit: Duration) -> Self {
        self.execution_time_limit = limit;
        self
    }

    pub fn program_file(
        mut self,
        filepath: impl AsRef<Path>,
    ) -> std::result::Result<Self, InterpError> {
        self.cmd = Self::interpolate_command_with_program_file(filepath, self.cmd)?;
        Ok(self)
    }

    pub fn interpolate_command_with_program_file(
        filepath: impl AsRef<Path>,
        mut cmd: TestCommand,
    ) -> std::result::Result<TestCommand, InterpError> {
        let vars = Self::make_cmd_interp_vars(filepath.as_ref());
        cmd.compile = cmd.compile.map(|fmt| interp(&fmt, &vars)).transpose()?;
        cmd.run = interp(&cmd.run, &vars)?;
        Ok(cmd)
    }

    fn make_cmd_interp_vars(filepath: &Path) -> HashMap<&'static str, &OsStr> {
        let mut m: HashMap<_, &OsStr> = HashMap::new();
        m.insert("filePath", filepath.as_ref());
        m.insert("fileName", filepath.file_name().unwrap());
        m.insert(
            "fileDir",
            filepath.parent().unwrap_or(Path::new(".")).as_os_str(),
        );
        m.insert(
            "fileStem",
            filepath
                .file_stem()
                .unwrap_or(OsStr::new("UNDEFINED_FILE_STEM")),
        );
        m.insert(
            "fileExt",
            filepath
                .extension()
                .unwrap_or(OsStr::new("UNDEFINED_FILE_EXTENSION")),
        );
        m
    }

    pub fn get_shell(&self) -> &Path {
        &self.shell
    }

    pub fn get_command(&self) -> &TestCommand {
        &self.cmd
    }

    pub fn get_exec_time_limit(&self) -> Duration {
        self.execution_time_limit
    }

    pub async fn compile(&self) -> anyhow::Result<()> {
        let Some(cmd) = &self.cmd.compile else {
            bail!("Undefined compile command")
        };
        println!("{}", cmd);

        let status = Command::new(&self.shell)
            .args(["-c", &cmd])
            .status()
            .await
            .with_context(|| {
                format!(
                    "Failed to spawn '{} -c {}'",
                    self.shell.to_string_lossy(),
                    cmd
                )
            })?;

        match status.code() {
            Some(0) => Ok(()),
            Some(code) => bail!("Compile error: exitcode={}", code),
            None => bail!("Failed to compile: process terminated by signal"),
        }
    }

    pub async fn run<'t, T>(&self, testcase: &'t T) -> anyhow::Result<TestOutcome<'t, T>>
    where
        T: AsyncTestcase<'t>,
    {
        let (mut input_reader, mut groundtruth_reader) = tokio::try_join!(
            testcase.new_input_reader(),
            testcase.new_groundtruth_reader()
        )?;

        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();

        let cmd = &self.cmd.run;
        let mut proc = Command::new(&self.shell)
            .args(["-c", &cmd])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| {
                format!(
                    "Failed to spawn '{} -c {}'",
                    self.shell.to_string_lossy(),
                    &cmd
                )
            })?;
        let mut stdin = proc.stdin.take().context("Failed to open stdin")?;
        let mut stdout = proc.stdout.take().context("Failed to open stdout")?;
        let mut stderr = proc.stderr.take().context("Failed to open stderr")?;

        let start_at = tokio::time::Instant::now();

        let res = {
            let fut_stdin = tokio::io::copy(&mut input_reader, &mut stdin);
            let fut_stdout = tokio::io::copy(&mut stdout, &mut stdout_buf);
            let fut_stderr = tokio::io::copy(&mut stderr, &mut stderr_buf);
            let fut_exit_status = proc.wait();

            tokio::time::timeout(self.execution_time_limit, async {
                tokio::try_join!(fut_stdin, fut_stdout, fut_stderr, fut_exit_status)
                    .context("Failed to communicate with subprocess")
            })
            .await
        };

        let execution_time = tokio::time::Instant::now().duration_since(start_at);

        let (judge, output) = match res {
            Err(_) => {
                proc.kill()
                    .await
                    .unwrap_or_else(|e| eprintln!("[Warn] Failed to kill TLE process: {:#}", e));
                (JudgeCode::TLE, None)
            }

            Ok(Err(e)) => bail!(e), // error on communicating with subprocess (io::copy)

            Ok(Ok((_, _, _, exit_status))) => {
                let judge = if !exit_status.success() {
                    JudgeCode::RE
                } else {
                    let mut groundtruth = Vec::new();
                    tokio::io::copy(&mut groundtruth_reader, &mut groundtruth)
                        .await
                        .context("Failed to read groundtruth data")?;
                    if stdout_buf == groundtruth {
                        JudgeCode::AC
                    } else {
                        JudgeCode::WA
                    }
                };
                let output = ProcessOutput {
                    status: exit_status,
                    stdout: stdout_buf,
                    stderr: stderr_buf,
                };
                (judge, Some(output))
            }
        };

        Ok(TestOutcome {
            judge,
            testcase,
            execution_time,
            output,
        })
    }
}

#[cfg(test)]
mod test {
    use std::process::Output;

    use super::*;

    async fn run_or_fail<'t, T: AsyncTestcase<'t>>(r: &TestRunner, t: &'t T) -> TestOutcome<'t, T> {
        r.run(t).await.unwrap_or_else(|e| {
            panic!("{:?}", e);
        })
    }

    #[tokio::test]
    async fn should_be_ac() {
        let cmd = TestCommand {
            compile: None,
            run: r#"python3 -c 'print("hello_" + input())'"#.to_owned(),
        };
        let t = OnMemoryTestcase::<&'static str>::new("sample", "123\n", "hello_123\n");
        let r = TestRunner::new(cmd);

        let res = run_or_fail(&r, &t).await;
        dbg!(&res);
        assert_eq!(res.judge, JudgeCode::AC);

        let Output {
            status,
            stdout,
            stderr,
        } = res.output.unwrap();
        assert_eq!(stdout, t.groundtruth.as_bytes());
        assert_eq!(status.code(), Some(0));
        assert!(stderr.is_empty());
    }
}
