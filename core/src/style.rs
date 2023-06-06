use colored::{Color, ColoredString, Colorize};

use crate::testing::JudgeCode;

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
