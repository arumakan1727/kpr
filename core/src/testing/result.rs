use std::{process, time::Duration};

use super::testcase::AsyncTestcase;

pub type ProcessOutput = process::Output;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JudgeCode {
    AC,
    WA,
    TLE,
    RE,
}
