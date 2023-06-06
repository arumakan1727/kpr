use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    pub status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct TestOutcome {
    pub testcase_name: String,
    pub judge: JudgeCode,
    pub execution_time: Duration,
    pub groundtruth: String,
    pub output: ProcessOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display)]
pub enum JudgeCode {
    AC,
    WA,
    TLE,
    RE,
}
