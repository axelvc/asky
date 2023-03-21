use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, KeyHandler},
    renderer::Renderer,
    theme::{DefaultTheme, Theme},
};

use super::text::{Direction, TextInput};

pub struct Number<'a> {
    pub(crate) message: &'a str,
    pub(crate) input: TextInput,
    pub(crate) placeholder: Option<&'a str>,
    pub(crate) default_value: Option<&'a str>,
    pub(crate) validator: Option<Box<dyn Fn(&str) -> Result<(), &'a str>>>,
    pub(crate) validator_result: Result<(), &'a str>,
    pub(crate) submit: bool,
    pub(crate) theme: &'a dyn Theme,
}

impl<'a> Number<'a> {
    pub fn new(message: &'a str) -> Number {
        Number {
            message,
            input: TextInput::new(),
            placeholder: None,
            default_value: None,
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

    pub fn initial(&mut self, value: &'a str) -> &mut Self {
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

    fn insert(&mut self, ch: char) {
        let is_sign = ch == '-' || ch == '+';

        if is_sign && self.input.col != 0 {
            return;
        }

        if !ch.is_digit(10) && !is_sign {
            return;
        }

        self.input.insert(ch)
    }

    fn validate_to_submit(&mut self) -> bool {
        if let Some(validator) = &self.validator {
            self.validator_result = validator(&self.get_value());
        }

        self.validator_result.is_ok()
    }
}

impl KeyHandler for Number<'_> {
    fn submit(&self) -> bool {
        self.submit
    }

    fn draw<W: std::io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()> {
        renderer.number(self)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        let mut submit = false;

        match key.code {
            // submit
            KeyCode::Enter => submit = self.validate_to_submit(),
            // type
            KeyCode::Char(c) => self.insert(c),
            // remove delete
            KeyCode::Backspace => self.input.backspace(),
            KeyCode::Delete => self.input.delete(),
            // move cursor
            KeyCode::Left => self.input.move_cursor(Direction::Left),
            KeyCode::Right => self.input.move_cursor(Direction::Right),
            _ => (),
        }

        self.submit = submit;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_placeholder() {
        let mut text = Number::new("");

        assert_eq!(text.placeholder, None);
        text.placeholder("foo");
        assert_eq!(text.placeholder, Some("foo"));
    }

    #[test]
    fn set_default_value() {
        let mut text = Number::new("");

        assert_eq!(text.default_value, None);
        text.default("foo");
        assert_eq!(text.default_value, Some("foo"));
    }

    #[test]
    fn set_initial_value() {
        let mut prompt = Number::new("");

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
    fn allow_sign_at_the_start() {
        let signs = ['-', '+'];

        for c in signs {
            let mut prompt = Number::new("");

            // must accept only one sign, simulate double press
            prompt.handle_key(KeyEvent::from(KeyCode::Char(c)));
            prompt.handle_key(KeyEvent::from(KeyCode::Char(c)));

            assert_eq!(prompt.input.value, c.to_string());
        }
    }

    #[test]
    fn only_accept_digits() {
        let mut prompt = Number::new("");

        // try to type a character
        ('a'..='z').for_each(|c| prompt.handle_key(KeyEvent::from(KeyCode::Char(c))));

        // try to type digits
        ('0'..='9').for_each(|c| prompt.handle_key(KeyEvent::from(KeyCode::Char(c))));

        assert_eq!(prompt.input.value, "0123456789");
    }

    #[test]
    fn update_cursor_position() {
        let mut prompt = Number::new("");
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
        let mut prompt = Number::new("");
        prompt.input.set_value("foo");
        prompt.default("bar");

        assert_eq!(prompt.get_value(), "foo");
    }

    #[test]
    fn submit_default_value() {
        let mut prompt = Number::new("");
        prompt.input.set_value("");
        prompt.default("bar");

        assert_eq!(prompt.get_value(), "bar");
    }
}
