use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, Typeable},
    renderer::{Printable, Renderer},
    theme,
};

pub enum Direction {
    Left,
    Right,
}

// region: TextPrompt

#[derive(Debug, PartialEq, Default)]
pub struct TextInput {
    pub value: String,
    pub col: usize,
}

impl TextInput {
    pub fn new() -> Self {
        TextInput::default()
    }
}

impl TextInput {
    pub(crate) fn set_value(&mut self, value: &str) {
        self.value = String::from(value);
        self.col = value.len();
    }

    pub(crate) fn insert(&mut self, ch: char) {
        self.value.insert(self.col, ch);
        self.col += 1;
    }

    pub(crate) fn backspace(&mut self) {
        if !self.value.is_empty() && self.col > 0 {
            self.col -= 1;
            self.value.remove(self.col);
        }
    }

    pub(crate) fn delete(&mut self) {
        if !self.value.is_empty() && self.col < self.value.len() {
            self.value.remove(self.col);
        }
    }

    pub(crate) fn move_cursor(&mut self, position: Direction) {
        self.col = match position {
            Direction::Left => self.col.saturating_sub(1),
            Direction::Right => (self.col + 1).min(self.value.len()),
        }
    }
}

// endregion

pub type InputValidator<'a> = dyn Fn(&str) -> Result<(), &'a str> + 'a;
type Formatter<'a> = dyn Fn(&Text, &Renderer) -> (String, Option<(u16, u16)>) + 'a;

pub struct Text<'a> {
    pub input: TextInput,
    pub message: &'a str,
    pub placeholder: Option<&'a str>,
    pub default_value: Option<&'a str>,
    pub validator_result: Result<(), &'a str>,
    validator: Option<Box<InputValidator<'a>>>,
    formatter: Box<Formatter<'a>>,
}

impl<'a> Text<'a> {
    pub fn new(message: &'a str) -> Self {
        Text {
            message,
            input: TextInput::new(),
            placeholder: None,
            default_value: None,
            validator: None,
            validator_result: Ok(()),
            formatter: Box::new(theme::fmt_text),
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

    pub fn validate<F>(&mut self, validator: F) -> &mut Self
    where
        F: Fn(&str) -> Result<(), &'a str> + 'a,
    {
        self.validator = Some(Box::new(validator));
        self
    }

    pub fn format<F>(&mut self, formatter: F) -> &mut Self
    where
        F: Fn(&Text, &Renderer) -> (String, Option<(u16, u16)>) + 'a,
    {
        self.formatter = Box::new(formatter);
        self
    }

    pub fn prompt(&mut self) -> io::Result<String> {
        key_listener::listen(self)?;
        Ok(self.get_value().to_owned())
    }
}

impl Text<'_> {
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

impl Typeable for Text<'_> {
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

impl Printable for Text<'_> {
    fn draw(&self, renderer: &mut crate::utils::renderer::Renderer) -> io::Result<()> {
        let (text, cursor) = (self.formatter)(self, renderer);
        renderer.print(&text)?;
        renderer.update_cursor(self.input.col, cursor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_placeholder() {
        let mut text = Text::new("");
        text.placeholder("foo");

        assert_eq!(text.placeholder, Some("foo"));
    }

    #[test]
    fn set_default_value() {
        let mut text = Text::new("");
        text.default("foo");

        assert_eq!(text.default_value, Some("foo"));
    }

    #[test]
    fn set_initial_value() {
        let mut prompt = Text::new("");

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
        let mut prompt: Text = Text::new("");
        let renderer = Renderer::new();

        const EXPECTED_VALUE: &str = "foo";
        let formatter = |_: &Text, _: &Renderer| (String::from(EXPECTED_VALUE), None);

        prompt.format(formatter);

        assert_eq!(
            (prompt.formatter)(&prompt, &renderer),
            (String::from(EXPECTED_VALUE), None)
        );
    }

    #[test]
    fn update_value() {
        let mut prompt = Text::new("");

        // simulate typing
        let text = "foo";

        for char in text.chars() {
            prompt.handle_key(KeyEvent::from(KeyCode::Char(char)));
        }

        assert_eq!(prompt.input.value, "foo");
        assert_eq!(prompt.input.col, 3);

        // removing
        let keys = [(KeyCode::Backspace, "fo"), (KeyCode::Delete, "f")];
        prompt.input.col = 2;

        for (key, expected) in keys {
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.input.value, expected);
            assert_eq!(prompt.input.col, 1);
        }
    }

    #[test]
    fn update_cursor_position() {
        let mut prompt = Text::new("");
        prompt.input.set_value("foo");
        prompt.input.col = 2;

        let keys = [(KeyCode::Left, 1), (KeyCode::Right, 2)];

        for (key, expected) in keys {
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.input.col, expected);
        }
    }

    #[test]
    fn validate_input() {
        let mut prompt = Text::new("");
        let err_str = "Please enter an response";

        prompt.validate(|s| if s.is_empty() { Err(err_str) } else { Ok(()) });

        // invalid value
        let mut submit = prompt.handle_key(KeyEvent::from(KeyCode::Enter));

        assert!(!submit);
        assert_eq!(prompt.validator_result, Err(err_str));

        // valid value
        prompt.input.set_value("foo");
        submit = prompt.handle_key(KeyEvent::from(KeyCode::Enter));

        assert!(submit);
        assert_eq!(prompt.validator_result, Ok(()));
    }

    #[test]
    fn submit_input_value() {
        let mut prompt = Text::new("");
        prompt.input.set_value("foo");
        prompt.default("bar");

        assert_eq!(prompt.get_value(), "foo");
    }

    #[test]
    fn submit_default_value() {
        let mut prompt = Text::new("");
        prompt.input.set_value("");
        prompt.default("bar");

        assert_eq!(prompt.get_value(), "bar");
    }
}
