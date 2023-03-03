use std::io;

use crossterm::{
    event::{read, Event, KeyCode, KeyEvent},
    terminal,
};

use crate::{
    renderer::{DrawTime, Renderer},
    utils,
};

enum Position {
    Left,
    Right,
}

enum LinePromptKind {
    Text,
    Password,
    Hidden,
}

pub struct Line<'a, W: io::Write> {
    kind: LinePromptKind,
    renderer: Renderer<'a, W>,
    value: String,
    default_value: String,
    validator: Option<Box<dyn Fn(&str) -> Result<(), &str>>>,
    validator_result: Result<(), String>,
    cursor_col: usize,
    submit: bool,
}

impl<W: io::Write> Line<'_, W> {
    pub fn default(&mut self, value: &str) -> &mut Self {
        self.default_value = String::from(value);
        self
    }

    pub fn initial(&mut self, value: &str) -> &mut Self {
        self.value = String::from(value);
        self.cursor_col = value.len();
        self
    }

    pub fn validate<F>(&mut self, validator: F) -> &mut Self
    where
        F: Fn(&str) -> Result<(), &str> + 'static,
    {
        self.validator = Some(Box::new(validator));
        self
    }

    pub fn prompt(&mut self) -> io::Result<String> {
        terminal::enable_raw_mode()?;
        self.draw()?;

        while !self.submit {
            if let Event::Key(key) = read()? {
                if utils::is_abort(key) {
                    self.last_draw()?;
                    utils::abort()?;
                }

                self.handle_key(key);
                self.draw()?;
            }
        }

        self.last_draw()?;
        terminal::disable_raw_mode()?;

        Ok(self.get_value().to_owned())
    }

    fn last_draw(&mut self) -> io::Result<()> {
        self.renderer.update_draw_time();
        self.draw()
    }

    fn draw(&mut self) -> io::Result<()> {
        match self.kind {
            LinePromptKind::Text => self.renderer.draw_text(
                &self.value,
                &self.default_value,
                &self.validator_result,
                self.cursor_col as u16,
            ),
            LinePromptKind::Password => self.renderer.draw_password(
                &self.value,
                &self.default_value,
                &self.validator_result,
                self.cursor_col as u16,
            ),
            LinePromptKind::Hidden => match self.renderer.draw_time {
                DrawTime::First => self.renderer.draw_password(
                    &self.value,
                    &self.default_value,
                    &self.validator_result,
                    self.cursor_col as u16,
                ),
                _ => Ok(()),
            },
        }
    }

    fn get_value(&self) -> &String {
        if self.value.is_empty() {
            &self.default_value
        } else {
            &self.value
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        let mut submit = false;

        match key.code {
            // submit
            KeyCode::Enter => submit = self.validate_to_submit(),
            // type
            KeyCode::Char(c) => self.update_value(c),
            // remove delete
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),
            // move cursor
            KeyCode::Left => self.move_cursor(Position::Left),
            KeyCode::Right => self.move_cursor(Position::Right),
            _ => (),
        };

        self.submit = submit;
    }

    fn update_value(&mut self, char: char) {
        self.value.insert(self.cursor_col, char);
        self.cursor_col += 1;
    }

    fn validate_to_submit(&mut self) -> bool {
        let validator_result = match &self.validator {
            Some(validator) => validator(self.get_value()).map_err(|e| e.to_string()),
            None => Ok(()),
        };

        self.validator_result = validator_result;
        self.validator_result.is_ok()
    }

    fn backspace(&mut self) {
        if !self.value.is_empty() && self.cursor_col > 0 {
            self.cursor_col -= 1;
            self.value.remove(self.cursor_col);
        }
    }

    fn delete(&mut self) {
        if !self.value.is_empty() && self.cursor_col < self.value.len() {
            self.value.remove(self.cursor_col);
        }
    }

    fn move_cursor(&mut self, position: Position) {
        self.cursor_col = match position {
            Position::Left => self.cursor_col.saturating_sub(1),
            Position::Right => (self.cursor_col + 1).min(self.value.len()),
        }
    }
}

impl Line<'_, io::Stdout> {
    fn new(message: &str, kind: LinePromptKind) -> Line<'_, io::Stdout> {
        Line {
            kind,
            renderer: Renderer::new(message),
            value: String::new(),
            default_value: String::new(),
            cursor_col: 0,
            validator: None,
            validator_result: Ok(()),
            submit: false,
        }
    }

    pub fn new_text(message: &str) -> Line<'_, io::Stdout> {
        Line::new(message, LinePromptKind::Text)
    }

    pub fn new_password(message: &str) -> Line<'_, io::Stdout> {
        Line::new(message, LinePromptKind::Password)
    }

    pub fn new_hidden(message: &str) -> Line<'_, io::Stdout> {
        Line::new(message, LinePromptKind::Hidden)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_default_value() {
        let mut text = Line::new_text("foo");
        text.default("bar");

        assert_eq!(text.default_value, "bar");
    }

    #[test]
    fn set_initial_value() {
        let mut prompt = Line::new_text("");

        prompt.initial("bar");
        assert_eq!(prompt.value, "bar");
        assert_eq!(prompt.cursor_col, 3);
    }

    #[test]
    fn update_value() {
        let mut prompt = Line::new_text("");

        // simulate typing
        let text = "foo";

        for char in text.chars() {
            prompt.handle_key(KeyEvent::from(KeyCode::Char(char)));
        }

        assert_eq!(prompt.value, "foo");
        assert_eq!(prompt.cursor_col, 3);

        // removing
        let keys = [(KeyCode::Backspace, "fo"), (KeyCode::Delete, "f")];
        prompt.cursor_col = 2;

        for (key, expected) in keys {
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.value, expected);
            assert_eq!(prompt.cursor_col, 1);
        }
    }

    #[test]
    fn update_cursor_position() {
        let mut prompt = Line::new_text("");
        prompt.value = "foo".to_string();
        prompt.cursor_col = 2;

        let keys = [(KeyCode::Left, 1), (KeyCode::Right, 2)];

        for (key, expected) in keys {
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.cursor_col, expected);
        }
    }

    #[test]
    fn submit_value() {
        let mut prompt = Line::new_text("");
        let err_str = "Please enter an input";

        prompt.validate(|s| if s.is_empty() { Err(err_str) } else { Ok(()) });

        // invalid value
        prompt.handle_key(KeyEvent::from(KeyCode::Enter));

        assert_eq!(prompt.submit, false);
        assert_eq!(prompt.validator_result, Err(err_str.to_string()));

        // valid value
        prompt.value = "foo".to_string();
        prompt.handle_key(KeyEvent::from(KeyCode::Enter));

        assert_eq!(prompt.submit, true);
        assert_eq!(prompt.validator_result, Ok(()));
    }
}
