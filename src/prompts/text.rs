use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, KeyHandler},
    renderer::Renderer,
    theme::{DefaultTheme, Theme},
};

enum Position {
    Left,
    Right,
}

pub struct Text<'a> {
    pub(crate) message: &'a str,
    pub(crate) value: String,
    pub(crate) placeholder: Option<&'a str>,
    pub(crate) default_value: Option<&'a str>,
    pub(crate) validator: Option<Box<dyn Fn(&str) -> Result<(), &str>>>,
    pub(crate) validator_result: Result<(), String>,
    pub(crate) cursor_col: usize,
    pub(crate) submit: bool,
    pub(crate) theme: &'a dyn Theme,
}

impl<'a> Text<'a> {
    pub fn new(message: &str) -> Text<'_> {
        Text {
            message,
            value: String::new(),
            placeholder: None,
            default_value: None,
            cursor_col: 0,
            validator: None,
            validator_result: Ok(()),
            submit: false,
            theme: &DefaultTheme,
        }
    }

    pub fn placeholder(&mut self, value: &'a str) -> &mut Self {
        self.placeholder = Some(value);
        self
    }

    pub fn default(&mut self, value: &'a str) -> &mut Self {
        self.default_value = Some(value);
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

    pub fn theme(&mut self, theme: &'a dyn Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn prompt(&mut self) -> io::Result<String> {
        key_listener::listen(self)?;
        Ok(self.get_value().to_owned())
    }

    pub(super) fn get_value(&self) -> &str {
        self.default_value.unwrap_or(&self.value)
    }

    pub(super) fn update_value(&mut self, char: char) {
        self.value.insert(self.cursor_col, char);
        self.cursor_col += 1;
    }

    fn validate_to_submit(&mut self) -> bool {
        let validator_result = match &self.validator {
            Some(validator) => validator(&self.get_value()).map_err(|e| e.to_string()),
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

impl KeyHandler for Text<'_> {
    fn submit(&self) -> bool {
        self.submit
    }

    fn draw<W: io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()> {
        renderer.text(self)
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_default_value() {
        let mut text = Text::new("foo");
        text.default("bar");

        assert_eq!(text.default_value, Some("bar"));
    }

    #[test]
    fn set_initial_value() {
        let mut prompt = Text::new("");

        prompt.initial("bar");
        assert_eq!(prompt.value, "bar");
        assert_eq!(prompt.cursor_col, 3);
    }

    #[test]
    fn update_value() {
        let mut prompt = Text::new("");

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
        let mut prompt = Text::new("");
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
        let mut prompt = Text::new("");
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
