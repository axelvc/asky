use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, KeyHandler},
    renderer::{DrawTime, Renderer},
    theme::{DefaultTheme, Theme},
};

use super::text::{Direction, TextInput};

pub struct Password<'a> {
    pub(crate) message: &'a str,
    pub(crate) input: TextInput,
    pub(crate) placeholder: Option<&'a str>,
    pub(crate) default_value: Option<&'a str>,
    pub(crate) validator: Option<Box<dyn Fn(&str) -> Result<(), &'a str>>>,
    pub(crate) validator_result: Result<(), &'a str>,
    pub(crate) hidden: bool,
    pub(crate) submit: bool,
    pub(crate) theme: &'a dyn Theme,
}

impl<'a> Password<'a> {
    pub fn new(message: &str) -> Password {
        Password {
            message,
            input: TextInput::new(),
            placeholder: None,
            default_value: None,
            validator: None,
            validator_result: Ok(()),
            hidden: false,
            submit: false,
            theme: &DefaultTheme,
        }
    }

    pub fn hidden(&mut self, hidden: bool) -> &mut Self {
        self.hidden = hidden;
        self
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
        self.input.set_value(value);
        self
    }

    pub fn validate<F>(&mut self, validator: F) -> &mut Self
    where
        F: Fn(&str) -> Result<(), &'a str> + 'static,
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

    fn get_value(&self) -> &str {
        match self.input.value.is_empty() {
            true => self.default_value.unwrap_or_default(),
            false => &self.input.value,
        }
    }

    fn validate_to_submit(&mut self) -> bool {
        if let Some(validator) = &self.validator {
            self.validator_result = validator(&self.get_value());
        }

        self.validator_result.is_ok()
    }
}

impl KeyHandler for Password<'_> {
    fn submit(&self) -> bool {
        self.submit
    }

    fn draw<W: io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()> {
        if self.hidden && renderer.draw_time == DrawTime::Update {
            return Ok(());
        }

        renderer.password(&self)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        let mut submit = false;

        match key.code {
            // submit
            KeyCode::Enter => submit = self.validate_to_submit(),
            // type
            KeyCode::Char(c) => self.input.insert(c),
            // remove delete
            KeyCode::Backspace => self.input.backspace(),
            KeyCode::Delete => self.input.delete(),
            // move cursor
            KeyCode::Left => self.input.move_cursor(Direction::Left),
            KeyCode::Right => self.input.move_cursor(Direction::Right),
            _ => (),
        };

        self.submit = submit;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_placeholder() {
        let mut text = Password::new("");

        assert_eq!(text.placeholder, None);
        text.placeholder("foo");
        assert_eq!(text.placeholder, Some("foo"));
    }

    #[test]
    fn set_default_value() {
        let mut text = Password::new("");

        assert_eq!(text.default_value, None);
        text.default("foo");
        assert_eq!(text.default_value, Some("foo"));
    }

    #[test]
    fn set_initial_value() {
        let mut prompt = Password::new("");

        assert_eq!(prompt.input, TextInput::new());

        prompt.initial("foo");

        assert_eq!(
            prompt.input,
            TextInput {
                value: String::from("foo"),
                col: 3,
            }
        );
    }

    #[test]
    fn set_hidden_value() {
        let mut prompt = Password::new("");

        assert!(!prompt.hidden);
        prompt.hidden(true);
        assert!(prompt.hidden)
    }

    #[test]
    fn update_cursor_position() {
        let mut prompt = Password::new("");
        prompt.input.set_value("foo");
        prompt.input.col = 2;

        let keys = [(KeyCode::Left, 1), (KeyCode::Right, 2)];

        for (key, expected) in keys {
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.input.col, expected);
        }
    }

    #[test]
    fn submit_input_value() {
        let mut prompt = Password::new("");
        prompt.input.set_value("foo");
        prompt.default("bar");

        assert_eq!(prompt.get_value(), "foo");
    }

    #[test]
    fn submit_default_value() {
        let mut prompt = Password::new("");
        prompt.input.set_value("");
        prompt.default("bar");

        assert_eq!(prompt.get_value(), "bar");
    }
}
