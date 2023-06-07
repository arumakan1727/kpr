use std::collections::HashMap;

use colored::{Color, ColoredString, Colorize};
use crossterm::terminal;

use crate::testing::{JudgeCode, TestOutcome};

#[macro_export]
macro_rules! print_success {
    ($fmt:literal, $($e:tt)*) => {
        use ::colored::Colorize as _;
        println!("{}", format!($fmt, $($e)*).green())
    }
}

pub fn is_truecolor_supported() -> bool {
    let Ok(v) = std::env::var("COLORTERM") else {
        return false
    };
    match v.as_str() {
        "truecolor" | "24bit" => true,
        _ => false,
    }
}

pub trait ColorTheme {
    fn color(&self) -> Color;
}

impl ColorTheme for log::Level {
    fn color(&self) -> Color {
        use log::Level::*;
        match self {
            Error => Color::BrightRed,
            Warn => Color::BrightYellow,
            Info => Color::Cyan,
            Debug => Color::Magenta,
            Trace => Color::Blue,
        }
    }
}

impl ColorTheme for JudgeCode {
    fn color(&self) -> Color {
        use JudgeCode::*;
        if !self::is_truecolor_supported() {
            return match self {
                AC => Color::Green,
                WA => Color::Yellow,
                TLE => Color::Red,
                RE => Color::Magenta,
            };
        }

        match self {
            AC => Color::TrueColor {
                r: 30,
                g: 180,
                b: 40,
            },
            WA => Color::TrueColor {
                r: 210,
                g: 138,
                b: 4,
            },
            TLE => Color::TrueColor {
                r: 220,
                g: 42,
                b: 42,
            },
            RE => Color::TrueColor {
                r: 171,
                g: 40,
                b: 200,
            },
        }
    }
}

pub fn judge_icon(judge: JudgeCode) -> ColoredString {
    let fg = if is_truecolor_supported() {
        Color::TrueColor {
            r: 255,
            g: 255,
            b: 255,
        }
    } else {
        Color::BrightBlack
    };
    format!(" {} ", judge)
        .on_color(judge.color())
        .bold()
        .color(fg)
}

pub fn print_test_result_summary(results: &[TestOutcome]) {
    let bar = "-".repeat(5);
    print!("{} ", bar);

    let count: HashMap<JudgeCode, usize> = results.iter().fold(HashMap::new(), |mut count, r| {
        *count.entry(r.judge).or_default() += 1;
        count
    });

    let num_total_test = results.len();
    let num_passed = *count.get(&JudgeCode::AC).unwrap_or(&0);
    let num_failed = num_total_test - num_passed;

    if num_passed == num_total_test {
        let msg = format!("All {} tests passed ✨", num_total_test);
        print!("{}", msg.green());
    } else {
        let summary_msg = if num_passed > 0 {
            format!("{}/{} tests failed 💣", num_failed, num_total_test)
        } else {
            format!("All {} tests failed 💀", num_total_test)
        };

        let detail_msg = count
            .iter()
            .filter(|(&judge, _)| judge != JudgeCode::AC)
            .map(|(&judge, &cnt)| {
                format!(
                    "{}{}{}",
                    self::judge_icon(judge),
                    "x".dimmed(),
                    cnt.to_string().bold().bright_white(),
                )
            })
            .collect::<Vec<String>>()
            .join(", ");

        print!("{} ({})", summary_msg.bright_red(), detail_msg);
    }

    println!(" {}", bar);
}

pub fn print_test_result_detail(res: &TestOutcome) {
    let stdout_lines: Vec<_> = res.output.stdout.lines().collect();
    let truth_lines: Vec<_> = res.groundtruth.lines().collect();

    let (cols, _) = terminal::size().unwrap_or((40, 40));

    const BOLD_LINE: &str = "━";
    const THIN_LINE: &str = "─";

    let bold_bar = BOLD_LINE.repeat(cols as usize).blue().bold();

    let title_color = Color::BrightYellow;
    println!(
        "\n{}: {} [{}ms]\n{}",
        res.testcase_name.color(title_color).bold(),
        self::judge_icon(res.judge),
        res.execution_time.as_millis(),
        bold_bar,
    );

    fn print_sub_title(s: &str, cols: usize) {
        println!(
            "{}{}",
            s.cyan().bold(),
            THIN_LINE.repeat(cols - s.len() - 1).bright_black(),
        )
    }

    fn print_lines(lines: &[&str], entire_str: &str) {
        if lines.is_empty() {
            println!("{}", "<EMPTY>".magenta().dimmed());
            return;
        }
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_end();
            print!("{}", trimmed);

            let num_trailling_whitespace = line.len() - trimmed.len();
            if num_trailling_whitespace > 0 {
                print!(
                    "{}{}",
                    " ".repeat(num_trailling_whitespace).on_red(),
                    "(Trailling whitespace)".bright_red().bold()
                );
            }

            let is_last_line = i + 1 == lines.len();
            if is_last_line && !entire_str.ends_with("\n") {
                print!("{}", " Missing new line ".on_yellow().black().bold());
            }

            println!("");
        }
    }

    print_sub_title("[truth-answer]", cols as usize);
    print_lines(&truth_lines, &res.groundtruth);

    print_sub_title("[stdout]", cols as usize);
    print_lines(&stdout_lines, &res.output.stdout);

    if !res.output.stderr.is_empty() {
        print_sub_title("[stderr]", cols as usize);
        print!("{}", res.output.stderr);
    }

    println!("{}", bold_bar);
}
