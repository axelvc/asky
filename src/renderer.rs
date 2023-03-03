use std::io;

use colored::Colorize;
use crossterm::{cursor, queue, style::Print, terminal};

pub enum DrawTime {
    First,
    Update,
    Last,
}

pub enum ConfirmOption {
    Yes,
    No,
}

impl ConfirmOption {
    fn to_string(&self) -> String {
        const YES: &str = " Yes ";
        const NO: &str = " No ";

        let (yes, no) = match self {
            ConfirmOption::Yes => (
                ConfirmOption::focused_colors(YES),
                ConfirmOption::unfocused_colors(NO),
            ),
            ConfirmOption::No => (
                ConfirmOption::unfocused_colors(YES),
                ConfirmOption::focused_colors(NO),
            ),
        };

        format!("{yes}  {no}")
    }

    fn focused_colors(s: &str) -> String {
        s.black().on_blue().to_string()
    }

    fn unfocused_colors(s: &str) -> String {
        s.white().on_bright_black().to_string()
    }
}

impl From<bool> for ConfirmOption {
    fn from(value: bool) -> Self {
        match value {
            true => ConfirmOption::Yes,
            false => ConfirmOption::No,
        }
    }
}

pub struct Renderer<'a, W: io::Write> {
    pub draw_time: DrawTime,
    message: &'a str,
    out: W,
}

impl<W: io::Write> Renderer<'_, W> {
    pub fn draw_text(
        &mut self,
        value: &str,
        default_value: &str,
        validator_result: &Result<(), String>,
        cursor_col: u16,
    ) -> io::Result<()> {
        // print message/question
        if let DrawTime::First = self.draw_time {
            queue!(self.out, Print(&self.message), cursor::MoveToNextLine(1))?;
            self.update_draw_time()
        }

        // print response prefix
        let prefix = if validator_result.is_ok() {
            "› ".blue()
        } else {
            "› ".red()
        };

        queue!(
            self.out,
            cursor::MoveToColumn(0),
            Print(prefix),
            terminal::Clear(terminal::ClearType::UntilNewLine),
        )?;

        // print response or default value
        let value = if value.is_empty() {
            default_value.bright_black().to_string()
        } else {
            value.to_owned()
        };

        queue!(self.out, Print(value))?;

        // print/clean validator error
        queue!(self.out, cursor::MoveToNextLine(1))?;

        if let Err(validator_error) = validator_result {
            queue!(self.out, Print(validator_error.bright_red()))?;
        } else {
            queue!(self.out, terminal::Clear(terminal::ClearType::CurrentLine))?;
        }

        queue!(self.out, cursor::MoveToPreviousLine(1))?;

        // set cursor position
        if let DrawTime::Last = self.draw_time {
            queue!(self.out, cursor::MoveToNextLine(1))?;
        } else {
            queue!(self.out, cursor::MoveToColumn(2 + cursor_col as u16))?;
        }

        self.out.flush()
    }

    pub fn draw_password(
        &mut self,
        value: &str,
        placeholder: &str,
        validator_result: &Result<(), String>,
        cursor_col: u16,
    ) -> io::Result<()> {
        let value = "*".repeat(value.len());
        let placeholder = "*".repeat(placeholder.len());

        self.draw_text(&value, &placeholder, validator_result, cursor_col)
    }

    pub fn draw_toggle(&mut self, value: bool) -> io::Result<()> {
        if let DrawTime::First = self.draw_time {
            queue!(self.out, Print(self.message), cursor::MoveToNextLine(2))?;
            self.update_draw_time();
        }

        let options = ConfirmOption::from(value);

        queue!(
            self.out,
            cursor::MoveToPreviousLine(1),
            cursor::MoveToColumn(0),
            Print(options.to_string()),
            cursor::MoveToNextLine(1),
        )?;

        self.out.flush()
    }

    pub fn update_draw_time(&mut self) {
        self.draw_time = match self.draw_time {
            DrawTime::First => DrawTime::Update,
            _ => DrawTime::Last,
        }
    }
}

impl Renderer<'_, io::Stdout> {
    pub fn new(message: &str) -> Renderer<'_, io::Stdout> {
        Renderer {
            draw_time: DrawTime::First,
            message,
            out: io::stdout(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Renderer<'_, Vec<u8>> {
        fn to_test(message: &str) -> Renderer<'_, Vec<u8>> {
            Renderer {
                draw_time: DrawTime::First,
                message,
                out: Vec::new(),
            }
        }
    }

    #[test]
    fn render_text_with_value() {
        let message = "What's your name?";
        let mut renderer = Renderer::to_test(message);
        let value = "Leopoldo";
        let default_value = "Goofy";
        let validator_result = Ok(());
        let cursor_col = 0;

        renderer
            .draw_text(value, default_value, &validator_result, cursor_col)
            .unwrap();

        let out_str = String::from_utf8(renderer.out).unwrap();

        assert!(out_str.contains(message));
        assert!(out_str.contains(value));
        assert!(!out_str.contains(default_value));
    }

    #[test]
    fn render_text_with_default_value() {
        let message = "What's your name?";
        let mut renderer = Renderer::to_test(message);
        let value = "";
        let default_value = "Goofy";
        let validator_result = Ok(());
        let cursor_col = 0;

        renderer
            .draw_text(value, default_value, &validator_result, cursor_col)
            .unwrap();

        let out_str = String::from_utf8(renderer.out).unwrap();

        assert!(out_str.contains(message));
        assert!(out_str.contains(default_value));
    }

    #[test]
    fn render_text_with_error() {
        let message = "What's your name?";
        let mut renderer = Renderer::to_test(message);
        let value = "Leopoldo";
        let default_value = "";
        let validator_result = Err("Invalid name".to_string());
        let cursor_col = 0;

        renderer
            .draw_text(value, default_value, &validator_result, cursor_col)
            .unwrap();

        let out_str = String::from_utf8(renderer.out).unwrap();

        assert!(out_str.contains(message));
        assert!(out_str.contains(value));
        assert!(out_str.contains(&validator_result.unwrap_err()));
    }

    #[test]
    fn render_toggle() {
        let message = "Do you like pizza?";
        let cases = [true, false];

        for case in cases {
            let mut renderer = Renderer::to_test(message);
            let expeted_option_str = ConfirmOption::from(case).to_string();

            renderer.draw_toggle(case).unwrap();

            let out_str = String::from_utf8(renderer.out).unwrap();
            assert!(out_str.contains(&message));
            assert!(out_str.contains(&expeted_option_str));
        }
    }
}
