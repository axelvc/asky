use std::io::{self, Write};

use colored::Colorize;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::Print,
    terminal,
};

enum Position {
    Left,
    Right,
}

pub struct Text<'a, W: Write> {
    message: &'a str,
    out: W,
    value: String,
    default_value: String,
    validator: Option<fn(&str) -> Result<(), &str>>,
    validator_result: Result<(), String>,
    cursor_col: usize,
    first_draw: bool,
    last_render: bool,
}

impl<'a, W: Write> Text<'a, W> {
    pub fn default(&mut self, value: &str) -> &mut Self {
        self.default_value = String::from(value);
        self
    }

    pub fn initial(&mut self, value: &str) -> &mut Self {
        self.value = String::from(value);
        self.cursor_col = value.len();
        self
    }

    pub fn validate(&mut self, validator: fn(&str) -> Result<(), &str>) -> &mut Self {
        self.validator = Some(validator);
        self
    }

    pub fn prompt(&mut self) -> io::Result<String> {
        self.draw()?;

        match self.listen() {
            Ok(_) => {
                let value = if self.value.is_empty() {
                    &self.default_value
                } else {
                    &self.value
                };

                Ok(value.to_string())
            }
            Err(e) => {
                terminal::disable_raw_mode()?;
                Err(e)
            }
        }
    }

    fn get_value(&self) -> &String {
        if self.value.is_empty() {
            &self.default_value
        } else {
            &self.value
        }
    }

    fn draw(&mut self) -> io::Result<()> {
        // print message/question
        if self.first_draw {
            queue!(self.out, Print(&self.message), cursor::MoveToNextLine(1))?;
            self.first_draw = false;
        }

        // print/clean validator error
        if let Err(validator_error) = &self.validator_result {
            queue!(
                self.out,
                cursor::MoveToNextLine(1),
                Print(validator_error.bright_red()),
                cursor::MoveToPreviousLine(1),
            )?;
        } else {
            queue!(
                self.out,
                cursor::MoveToNextLine(1),
                terminal::Clear(terminal::ClearType::CurrentLine),
                cursor::MoveToPreviousLine(1),
            )?;
        }

        // print response prefix
        let prefix = if self.validator_result.is_ok() {
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
        let mut value = self.get_value().normal();

        if self.value.is_empty() {
            value = value.bright_black();
        }

        queue!(self.out, Print(value))?;

        // set cursor position
        if self.last_render {
            queue!(self.out, cursor::MoveToNextLine(1))?;
        } else {
            queue!(self.out, cursor::MoveToColumn(2 + self.cursor_col as u16))?;
        }

        self.out.flush()
    }

    fn listen(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;

        loop {
            if let Event::Key(key) = read()? {
                match key {
                    // submit
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    } => {
                        let mut validator_result = Ok(());

                        if let Some(validator) = &self.validator {
                            if let Err(e) = validator(self.get_value()) {
                                validator_result = Err(e.to_string());
                            }
                        }

                        if let Err(e) = validator_result {
                            self.validator_result = Err(e.to_string());
                            self.draw()?;
                            self.validator_result = Ok(());
                        } else {
                            self.last_render = true;
                            self.draw()?;

                            break;
                        }
                    }
                    // delete
                    KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    } => self.backspace()?,
                    KeyEvent {
                        code: KeyCode::Delete,
                        ..
                    } => self.delete()?,
                    // move cursor
                    KeyEvent {
                        code: KeyCode::Left,
                        ..
                    } => self.move_cursor(Position::Left)?,
                    KeyEvent {
                        code: KeyCode::Right,
                        ..
                    } => self.move_cursor(Position::Right)?,
                    // abort
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    }
                    | KeyEvent {
                        code: KeyCode::Char('c' | 'd'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => self.abort()?,
                    // type
                    KeyEvent {
                        code: KeyCode::Char(c),
                        ..
                    } => self.update_value(c)?,
                    _ => (),
                }
            }
        }

        terminal::disable_raw_mode()
    }

    fn update_value(&mut self, char: char) -> io::Result<()> {
        self.value.insert(self.cursor_col, char);
        self.cursor_col += 1;

        self.draw()
    }

    fn backspace(&mut self) -> io::Result<()> {
        if !self.value.is_empty() && self.cursor_col > 0 {
            self.cursor_col -= 1;
            self.value.remove(self.cursor_col);
        }

        self.draw()
    }

    fn delete(&mut self) -> io::Result<()> {
        if !self.value.is_empty() {
            self.value.remove(self.cursor_col);
        }

        self.draw()
    }

    fn move_cursor(&mut self, position: Position) -> io::Result<()> {
        self.cursor_col = match position {
            Position::Left => self.cursor_col.saturating_sub(1),
            Position::Right => (self.cursor_col + 1).min(self.value.len()),
        };

        self.draw()
    }

    fn abort(&mut self) -> io::Result<()> {
        queue!(self.out, cursor::MoveToNextLine(1))?;
        self.out.flush()?;

        terminal::disable_raw_mode()?;
        std::process::exit(0)
    }
}

impl<'a> Text<'a, io::Stdout> {
    pub fn new(message: &str) -> Text<'_, io::Stdout> {
        Text {
            message,
            out: io::stdout(),
            value: String::new(),
            default_value: String::new(),
            cursor_col: 0,
            validator: None,
            validator_result: Ok(()),
            first_draw: true,
            last_render: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_default_value() {
        let mut text = Text::new("foo");
        text.default("bar");

        assert_eq!(text.default_value, "bar");
    }

    #[test]
    fn set_initial_value() {
        let mut text = Text::new("foo");
        text.initial("bar");

        assert_eq!(text.value, "bar");
        assert_eq!(text.cursor_col, 3);
    }
}
