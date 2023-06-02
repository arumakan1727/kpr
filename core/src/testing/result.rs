use std::time::Duration;

use super::testcase::AsyncTestcase;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    pub status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone)]
pub struct TestOutcome<'t, T>
where
    T: AsyncTestcase<'t>,
{
    pub judge: JudgeCode,
    pub testcase: &'t T,
    pub execution_time: Duration,
    pub output: Option<ProcessOutput>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::Display)]
pub enum JudgeCode {
    AC,
    WA,
    TLE,
    RE,
}
