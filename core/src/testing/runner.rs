use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use anyhow::{bail, Context};
use tokio::{io::AsyncReadExt, process::Command};

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

    #[must_use]
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

    pub fn is_compile_cmd_defined(&self) -> bool {
        self.cmd.compile.is_some()
    }

    pub async fn compile(&self) -> anyhow::Result<()> {
        let Some(cmd) = &self.cmd.compile else {
            bail!("Undefined compile command")
        };

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

    pub async fn run<'t, T>(
        &self,
        testcase: &'t T,
        stdout_capture_max_bytes: usize,
        stderr_capture_max_bytes: usize,
    ) -> anyhow::Result<TestOutcome>
    where
        T: AsyncTestcase<'t>,
    {
        let (mut input_reader, mut groundtruth_reader) = tokio::try_join!(
            testcase.new_input_reader(),
            testcase.new_groundtruth_reader()
        )?;

        let mut stdout_buf = Vec::with_capacity(stdout_capture_max_bytes);
        let mut stderr_buf = Vec::with_capacity(stderr_capture_max_bytes);

        let mut groundtrugh_buf = Vec::new();
        let fut_groundtruth_read = tokio::io::copy(&mut groundtruth_reader, &mut groundtrugh_buf);

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
        let mut stdout = proc.stdout.take().expect("Failed to open stdout");
        let mut stderr = proc.stderr.take().expect("Failed to open stderr");
        let mut stdin = proc.stdin.take().expect("Failed to open stdin");

        tokio::io::copy(&mut input_reader, &mut stdin)
            .await
            .context("Failed to pass input-data to stdin")?;
        drop(input_reader);
        drop(stdin); // NOTE: this line is essential

        let start_at = tokio::time::Instant::now();
        let wait_result = tokio::time::timeout(self.execution_time_limit, proc.wait()).await;
        let execution_time = tokio::time::Instant::now().duration_since(start_at);

        let (is_timeout, exit_code) = match wait_result {
            Ok(Ok(status)) => (false, status.code()),
            Ok(Err(e)) => panic!("Failed to wait for child process to exit: {}", e),
            Err(_) => {
                proc.kill()
                    .await
                    .unwrap_or_else(|e| log::warn!("Failed to kill TLE process: {:#}", e));
                (true, None)
            }
        };

        stdout
            .read_buf(&mut stdout_buf)
            .await
            .expect("Failed to capture stdout");
        stderr
            .read_buf(&mut stderr_buf)
            .await
            .expect("Failed to capture stderr");

        fut_groundtruth_read.await?;

        let groundtruth = String::from_utf8_lossy(&groundtrugh_buf).to_string();
        let stdout = String::from_utf8_lossy(&stdout_buf).to_string();
        let stderr = String::from_utf8_lossy(&stderr_buf).to_string();

        let judge = if is_timeout {
            JudgeCode::TLE
        } else if exit_code != Some(0) {
            JudgeCode::RE
        } else if stdout != groundtruth {
            JudgeCode::WA
        } else {
            JudgeCode::AC
        };

        Ok(TestOutcome {
            testcase_name: testcase.name().to_owned(),
            judge,
            execution_time,
            groundtruth,
            output: ProcessOutput {
                status: exit_code,
                stdout,
                stderr,
            },
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct X {
        input: &'static str,
        groundtruth: &'static str,
        pyscript: &'static str,
        want_judge: JudgeCode,
        want_output: ProcessOutput,
    }
    const STDERR_CAPTURE_MAX_BYTES: usize = 64;
    const STDOUT_CAPTURE_MAX_BYTES: usize = 256;

    async fn run_test(x: X) -> () {
        let cmd = TestCommand {
            compile: None,
            // terminate '  ->  enclose ' with "  ->  restart '
            run: format!("python3 -c '{}'", x.pyscript.replace("'", r#"'"'"'"#)),
        };
        let t = OnMemoryTestcase::<&'static str>::new("sample testcase", x.input, x.groundtruth);
        let r = TestRunner::new(cmd).execution_time_limit(Duration::from_millis(300));

        let res = dbg!(
            r.run(&t, STDOUT_CAPTURE_MAX_BYTES, STDERR_CAPTURE_MAX_BYTES)
                .await
        )
        .unwrap();
        assert_eq!(res.judge, x.want_judge);
        assert_eq!(res.output, x.want_output);
    }

    #[tokio::test]
    async fn should_be_ac() {
        run_test(X {
            input: "123\n",
            groundtruth: "hello_123\n",
            pyscript: r#"print("hello_" + input())"#,
            want_judge: JudgeCode::AC,
            want_output: ProcessOutput {
                status: Some(0),
                stdout: "hello_123\n".into(),
                stderr: "".into(),
            },
        })
        .await;
    }

    #[tokio::test]
    async fn should_be_ac_even_if_stdin_is_not_read() {
        run_test(X {
            input: "123\n",
            groundtruth: "hello_123\n",
            pyscript: r#"print("hello_123")"#,
            want_judge: JudgeCode::AC,
            want_output: ProcessOutput {
                status: Some(0),
                stdout: "hello_123\n".into(),
                stderr: "".into(),
            },
        })
        .await;
    }

    #[tokio::test]
    async fn should_be_wa() {
        run_test(X {
            input: "123\n",
            groundtruth: "hello_123\n",
            pyscript: r#"import sys; print("hello_123", file=sys.stderr)"#,
            want_judge: JudgeCode::WA,
            want_output: ProcessOutput {
                status: Some(0),
                stdout: "".into(),
                stderr: "hello_123\n".into(),
            },
        })
        .await;
    }

    #[tokio::test]
    async fn should_be_wa_if_just_missing_newline() {
        run_test(X {
            input: "123\n",
            groundtruth: "hello_123\n",
            pyscript: r#"print("hello_123", end='')"#,
            want_judge: JudgeCode::WA,
            want_output: ProcessOutput {
                status: Some(0),
                stdout: "hello_123".into(),
                stderr: "".into(),
            },
        })
        .await;
    }

    #[tokio::test]
    async fn should_be_re_even_if_stdout_is_correct() {
        run_test(X {
            input: "123\n",
            groundtruth: "hello_123\n",
            pyscript: r#"print("hello_123"); exit(42)"#,
            want_judge: JudgeCode::RE,
            want_output: ProcessOutput {
                status: Some(42),
                stdout: "hello_123\n".into(),
                stderr: "".into(),
            },
        })
        .await;
    }

    #[tokio::test]
    async fn should_be_tle() {
        run_test(X {
            input: "123\n",
            groundtruth: "hello_123\n",
            pyscript: "\
import sys, time
print('hello', flush=True)
print('world', file=sys.stderr)
time.sleep(0.5)",
            want_judge: JudgeCode::TLE,
            want_output: ProcessOutput {
                status: None,
                stdout: "hello\n".into(),
                stderr: "world\n".into(),
            },
        })
        .await;
    }

    #[tokio::test]
    async fn should_be_tle_and_massive_output_should_not_exceed_capture_limit() {
        run_test(X {
            input: "123\n",
            groundtruth: "hello_123\n",
            pyscript: "\
import sys
while True:
    print('hello')
    print('world', file=sys.stderr)
",
            want_judge: JudgeCode::TLE,
            want_output: ProcessOutput {
                status: None,
                stdout: "hello\n".repeat(100)[..STDOUT_CAPTURE_MAX_BYTES].into(),
                stderr: "world\n".repeat(100)[..STDERR_CAPTURE_MAX_BYTES].into(),
            },
        })
        .await;
    }
}
