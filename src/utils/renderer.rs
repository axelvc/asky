use std::io;

use colored::Colorize;
use crossterm::{cursor, queue, style::Print, terminal};

use crate::prompts::{multi_select::MultiSelect, select::Select, text::Text, toggle::Toggle};

#[derive(PartialEq)]
pub enum DrawTime {
    First,
    Update,
    Last,
}

pub struct Renderer<W: io::Write> {
    pub draw_time: DrawTime,
    out: W,
}

impl<W: io::Write> Renderer<W> {
    pub fn text(&mut self, state: &Text) -> io::Result<()> {
        self.draw_line(
            state.message,
            &state.value,
            &state.default_value,
            &state.validator_result,
            state.cursor_col,
        )
    }

    pub fn password(&mut self, state: &Text, is_hidden: bool) -> io::Result<()> {
        let (value, cursor_col) = match is_hidden {
            true => (String::new(), 0),
            false => ("*".repeat(state.value.len()), state.cursor_col),
        };

        self.draw_line(
            state.message,
            &value,
            &state.default_value,
            &state.validator_result,
            cursor_col,
        )
    }

    pub fn toggle(&mut self, state: &Toggle) -> io::Result<()> {
        if let DrawTime::First = self.draw_time {
            queue!(self.out, Print(state.message), cursor::MoveToNextLine(2))?;
            self.update_draw_time();
        }

        queue!(
            self.out,
            cursor::MoveToPreviousLine(1),
            cursor::MoveToColumn(0),
            Print(toggle_string(state.options, state.active)),
            cursor::MoveToNextLine(1),
        )?;

        self.out.flush()
    }

    pub fn select<T: ToString + Copy>(&mut self, state: &Select<T>) -> io::Result<()> {
        if let DrawTime::First = self.draw_time {
            queue!(self.out, Print(state.message), cursor::MoveToNextLine(1))?;
            self.update_draw_time();
        } else {
            queue!(
                self.out,
                cursor::MoveToPreviousLine(state.options.len() as u16)
            )?;
        }

        for (i, option) in state.options.iter().enumerate() {
            let option = option.to_string();
            let (prefix, option) = if i == state.selected {
                ("● ".blue(), option.blue())
            } else {
                ("○ ".bright_black(), option.normal())
            };

            queue!(
                self.out,
                Print(prefix),
                Print(option),
                cursor::MoveToNextLine(1),
            )?;
        }

        self.out.flush()
    }

    pub fn multi_select<T: ToString + Copy>(&mut self, state: &MultiSelect<T>) -> io::Result<()> {
        if let DrawTime::First = self.draw_time {
            queue!(self.out, Print(state.message), cursor::MoveToNextLine(1))?;
            self.update_draw_time();
        } else {
            queue!(
                self.out,
                cursor::MoveToPreviousLine(state.options.len() as u16)
            )?;
        }

        for (i, option) in state.options.iter().enumerate() {
            let str = option.value.to_string();
            let (mut prefix, mut option) = if option.selected {
                ("● ".normal(), str.normal())
            } else {
                ("○ ".bright_black(), str.normal())
            };

            if i == state.focused {
                prefix = prefix.blue();
                option = option.blue();
            }

            queue!(
                self.out,
                Print(prefix),
                Print(option),
                cursor::MoveToNextLine(1),
            )?;
        }

        self.out.flush()
    }

    pub fn update_draw_time(&mut self) {
        self.draw_time = match self.draw_time {
            DrawTime::First => DrawTime::Update,
            _ => DrawTime::Last,
        }
    }

    fn draw_line(
        &mut self,
        message: &str,
        value: &str,
        default_value: &str,
        validator_result: &Result<(), String>,
        cursor_col: usize,
    ) -> io::Result<()> {
        // print message/question
        if let DrawTime::First = self.draw_time {
            queue!(self.out, Print(message), cursor::MoveToNextLine(1))?;
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
}

impl Renderer<io::Stdout> {
    pub fn new() -> Renderer<io::Stdout> {
        Renderer {
            draw_time: DrawTime::First,
            out: io::stdout(),
        }
    }
}

fn toggle_string(options: (&str, &str), active: bool) -> String {
    let (left, right) = (format!(" {} ", options.0), format!(" {} ", options.1));

    let (left, right) = match active {
        false => (toggle_focused(&left), toggle_unfocused(&right)),
        true => (toggle_unfocused(&left), toggle_focused(&right)),
    };

    format!("{}  {}", left, right)
}

fn toggle_focused(s: &str) -> String {
    s.black().on_blue().to_string()
}

fn toggle_unfocused(s: &str) -> String {
    s.white().on_bright_black().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Renderer<Vec<u8>> {
        fn to_test() -> Renderer<Vec<u8>> {
            Renderer {
                draw_time: DrawTime::First,
                out: Vec::new(),
            }
        }
    }

    #[test]
    fn render_text_with_value() {
        let state = Text {
            message: "What's your name?",
            value: "Leopoldo".to_string(),
            default_value: "Goofy".to_string(),
            validator: None,
            validator_result: Ok(()),
            cursor_col: 0,
            submit: false,
        };

        let mut renderer = Renderer::to_test();
        renderer.text(&state).unwrap();

        let out_str = String::from_utf8(renderer.out).unwrap();

        assert!(out_str.contains(state.message));
        assert!(out_str.contains(&state.value));
        assert!(!out_str.contains(&state.default_value));
    }

    #[test]
    fn render_text_with_default_value() {
        let state = Text {
            message: "What's your name?",
            value: String::new(),
            default_value: "Goofy".to_string(),
            validator: None,
            validator_result: Ok(()),
            cursor_col: 0,
            submit: false,
        };

        let mut renderer = Renderer::to_test();

        renderer.text(&state).unwrap();

        let out_str = String::from_utf8(renderer.out).unwrap();

        assert!(out_str.contains(state.message));
        assert!(out_str.contains(&state.default_value));
    }

    #[test]
    fn render_text_with_error() {
        let state = Text {
            message: "What's your name?",
            value: "Leopoldo".to_string(),
            default_value: String::new(),
            validator: None,
            validator_result: Err("Invalid name".to_string()),
            cursor_col: 0,
            submit: false,
        };

        let mut renderer = Renderer::to_test();
        renderer.text(&state).unwrap();

        let out_str = String::from_utf8(renderer.out).unwrap();

        assert!(out_str.contains(state.message));
        assert!(out_str.contains(&state.value));
        assert!(out_str.contains(&state.validator_result.unwrap_err()));
    }

    #[test]
    fn render_toggle() {
        let cases = [false, true];

        for active in cases {
            let state = &Toggle {
                message: "Do you like pizza?",
                options: ("foo", "bar"),
                active,
                submit: false,
            };

            let mut renderer = Renderer::to_test();

            renderer.toggle(&state).unwrap();

            let out_str = String::from_utf8(renderer.out).unwrap();
            let expeted_option_str = toggle_string(state.options, active);
            assert!(out_str.contains(&state.message));
            assert!(out_str.contains(&expeted_option_str));
        }
    }

    #[test]
    fn render_select() {
        let message = "What's your favorite number?";
        let mut renderer = Renderer::to_test();
        let state = Select {
            message,
            options: &["1", "2", "fish"],
            selected: 2,
            is_loop: false,
            submit: false,
        };

        renderer.select(&state).unwrap();

        let out_str = String::from_utf8(renderer.out).unwrap();
        assert!(out_str.contains(&message));
        assert!(out_str.contains("1"));
        assert!(out_str.contains("2"));
        assert!(out_str.contains("fish"));
    }
}
