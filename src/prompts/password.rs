use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, Typeable},
    renderer::{DrawTime, Printable, Renderer},
    theme,
};

use super::text::{Direction, InputValidator, TextInput};

type Formatter<'a> = dyn Fn(&Password, DrawTime) -> (String, Option<[u16; 2]>) + 'a;

pub struct Password<'a> {
    pub message: &'a str,
    pub input: TextInput,
    pub placeholder: Option<&'a str>,
    pub default_value: Option<&'a str>,
    pub hidden: bool,
    pub validator_result: Result<(), &'a str>,
    validator: Option<Box<InputValidator<'a>>>,
    formatter: Box<Formatter<'a>>,
}

impl<'a> Password<'a> {
    pub fn new(message: &'a str) -> Self {
        Password {
            message,
            input: TextInput::new(),
            placeholder: None,
            default_value: None,
            hidden: false,
            validator: None,
            validator_result: Ok(()),
            formatter: Box::new(theme::fmt_password),
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
        self.input.set_value(value);
        self
    }

    pub fn hidden(&mut self, hidden: bool) -> &mut Self {
        self.hidden = hidden;
        self
    }

    pub fn validate<F>(&mut self, validator: F) -> &mut Self
    where
        F: Fn(&str) -> Result<(), &'a str> + 'a,
    {
        self.validator = Some(Box::new(validator));
        self
    }

    pub fn format<F>(&mut self, formatter: F) -> &mut Self
    where
        F: Fn(&Password, DrawTime) -> (String, Option<[u16; 2]>) + 'a,
    {
        self.formatter = Box::new(formatter);
        self
    }

    pub fn prompt(&mut self) -> io::Result<String> {
        key_listener::listen(self)?;
        Ok(self.get_value().to_owned())
    }
}

impl Password<'_> {
    fn get_value(&self) -> &str {
        match self.input.value.is_empty() {
            true => self.default_value.unwrap_or_default(),
            false => &self.input.value,
        }
    }

    fn validate_to_submit(&mut self) -> bool {
        if let Some(validator) = &self.validator {
            self.validator_result = validator(self.get_value());
        }

        self.validator_result.is_ok()
    }
}

impl Typeable for Password<'_> {
    fn handle_key(&mut self, key: KeyEvent) -> bool {
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

        submit
    }
}

impl Printable for Password<'_> {
    fn draw(&self, renderer: &mut Renderer) -> io::Result<()> {
        let (text, cursor) = (self.formatter)(self, renderer.draw_time);

        renderer.print(&text)?;
        renderer.set_cursor(cursor)
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
    fn set_custom_formatter() {
        let mut prompt: Password = Password::new("");
        let draw_time = DrawTime::First;
        const EXPECTED_VALUE: &str = "foo";

        prompt.format(|_, _| (String::from(EXPECTED_VALUE), None));

        assert_eq!(
            (prompt.formatter)(&prompt, draw_time),
            (String::from(EXPECTED_VALUE), None)
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
