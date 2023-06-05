use colored::Color;

pub trait SemanticColor {
    fn level(&self, level: log::Level) -> Color {
        use log::Level::*;
        match level {
            Error => self.error(),
            Warn => self.warn(),
            Info => self.info(),
            Debug => self.debug(),
            Trace => self.trace(),
        }
    }

    fn error(&self) -> Color;
    fn warn(&self) -> Color;
    fn info(&self) -> Color;
    fn debug(&self) -> Color;
    fn trace(&self) -> Color;
    fn success(&self) -> Color;
}

pub struct DefaultPalette;

impl SemanticColor for DefaultPalette {
    fn error(&self) -> Color {
        Color::BrightRed
    }

    fn warn(&self) -> Color {
        Color::BrightYellow
    }

    fn info(&self) -> Color {
        Color::Cyan
    }

    fn debug(&self) -> Color {
        Color::Magenta
    }

    fn trace(&self) -> Color {
        Color::Blue
    }

    fn success(&self) -> Color {
        Color::Green
    }
}
