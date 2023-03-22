use std::{fmt::Display, io, str::FromStr};

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, KeyHandler},
    renderer::Renderer,
    theme::{DefaultTheme, Theme},
};

use super::text::{Direction, TextInput};

pub struct Number<'a, T: Num> {
    pub(crate) message: &'a str,
    pub(crate) input: TextInput,
    pub(crate) placeholder: Option<&'a str>,
    pub(crate) default_value: Option<String>,
    pub(crate) validator: Option<Box<dyn Fn(&str, Result<T, T::Err>) -> Result<(), &'a str>>>,
    pub(crate) validator_result: Result<(), &'a str>,
    pub(crate) submit: bool,
    pub(crate) theme: &'a dyn Theme,
}

impl<'a, T: Num> Number<'a, T> {
    pub fn new(message: &'a str) -> Number<T> {
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

    pub fn default(&mut self, value: T) -> &mut Self {
        self.default_value = Some(value.to_string());
        self
    }

    pub fn initial(&mut self, value: T) -> &mut Self {
        self.input.set_value(&value.to_string());
        self
    }

    pub fn validate<F>(&mut self, validator: F) -> &mut Self
    where
        F: Fn(&str, Result<T, T::Err>) -> Result<(), &'a str> + 'static,
    {
        self.validator = Some(Box::new(validator));
        self
    }

    pub fn theme(&mut self, theme: &'a dyn Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn prompt(&mut self) -> io::Result<Result<T, T::Err>> {
        key_listener::listen(self)?;
        Ok(self.get_value())
    }

    fn get_value(&self) -> Result<T, T::Err> {
        match self.input.value.is_empty() {
            true => self.default_value.clone().unwrap_or_default().parse(),
            false => self.input.value.parse(),
        }
    }

    fn insert(&mut self, ch: char) {
        let is_valid = match ch {
            '-' | '+' => T::is_signed() && self.input.col == 0,
            '.' => T::is_float() && !self.input.value.contains('.'),
            _ => ch.is_digit(10),
        };

        if is_valid {
            self.input.insert(ch)
        }
    }

    fn validate_to_submit(&mut self) -> bool {
        if let Some(validator) = &self.validator {
            self.validator_result = validator(&self.input.value, self.get_value());
        }

        self.validator_result.is_ok()
    }
}

impl<T: Num> KeyHandler for Number<'_, T> {
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

/// A utilitiy trait to allow only number types
pub trait Num: Default + Display + FromStr {
    fn is_float() -> bool {
        false
    }

    fn is_signed() -> bool {
        false
    }
}

impl Num for u8 {}
impl Num for u16 {}
impl Num for u32 {}
impl Num for u64 {}
impl Num for u128 {}
impl Num for usize {}

impl Num for i8 {
    fn is_signed() -> bool {
        true
    }
}

impl Num for i16 {
    fn is_signed() -> bool {
        true
    }
}

impl Num for i32 {
    fn is_signed() -> bool {
        true
    }
}

impl Num for i64 {
    fn is_signed() -> bool {
        true
    }
}

impl Num for i128 {
    fn is_signed() -> bool {
        true
    }
}

impl Num for isize {
    fn is_signed() -> bool {
        true
    }
}

impl Num for f32 {
    fn is_signed() -> bool {
        true
    }

    fn is_float() -> bool {
        true
    }
}

impl Num for f64 {
    fn is_signed() -> bool {
        true
    }

    fn is_float() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_placeholder() {
        let mut text = Number::<i32>::new("");

        assert_eq!(text.placeholder, None);
        text.placeholder("foo");
        assert_eq!(text.placeholder, Some("foo"));
    }

    #[test]
    fn set_default_value() {
        let mut text = Number::<i32>::new("");

        assert_eq!(text.default_value, None);
        text.default(10);
        assert_eq!(text.default_value, Some(String::from("10")));
    }

    #[test]
    fn set_initial_value() {
        let mut prompt = Number::<i32>::new("");

        assert_eq!(prompt.input, TextInput::new());

        prompt.initial(10);

        assert_eq!(
            prompt.input,
            TextInput {
                value: String::from("10"),
                col: 2,
            }
        );
    }

    #[test]
    fn update_cursor_position() {
        let mut prompt = Number::<i32>::new("");
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
        let mut prompt = Number::<i32>::new("");
        prompt.input.set_value(&String::from("10"));
        prompt.default(20);

        assert_eq!(prompt.get_value(), Ok(10));
    }

    #[test]
    fn submit_default_value() {
        let mut prompt = Number::<i32>::new("");
        prompt.input.set_value("");
        prompt.default(20);

        assert_eq!(prompt.get_value(), Ok(20));
    }

    #[test]
    fn allow_sign_at_the_start() {
        let signs = ['-', '+'];

        for c in signs {
            let mut prompt = Number::<i32>::new("");

            // must accept only one sign, simulate double press
            prompt.handle_key(KeyEvent::from(KeyCode::Char(c)));
            prompt.handle_key(KeyEvent::from(KeyCode::Char(c)));

            assert_eq!(prompt.input.value, c.to_string());
        }

        // not allow fo unsigned types
        for c in signs {
            let mut prompt = Number::<u32>::new("");

            prompt.handle_key(KeyEvent::from(KeyCode::Char(c)));

            assert!(prompt.input.value.is_empty());
        }
    }

    #[test]
    fn allow_only_digits() {
        let mut prompt = Number::<i32>::new("");

        // try to type a character
        ('a'..='z').for_each(|c| prompt.handle_key(KeyEvent::from(KeyCode::Char(c))));

        // try to type digits
        ('0'..='9').for_each(|c| prompt.handle_key(KeyEvent::from(KeyCode::Char(c))));

        assert_eq!(prompt.input.value, "0123456789");
    }

    #[test]
    fn allow_decimal_in_floats() {
        let mut prompt = Number::<f32>::new("");

        "1.".chars()
            .for_each(|c| prompt.handle_key(KeyEvent::from(KeyCode::Char(c))));

        assert_eq!(prompt.input.value, "1.");

        // not allow in integers
        let mut prompt = Number::<i32>::new("");

        "2.".chars()
            .for_each(|c| prompt.handle_key(KeyEvent::from(KeyCode::Char(c))));

        assert_eq!(prompt.input.value, "2");
    }
}
