use std::{io, str::FromStr};

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, Typeable},
    num::Num,
    renderer::{DrawTime, Printable, Renderer},
    theme,
};

use super::text::{Direction, TextInput};

type InputValidator<'a, T> =
    dyn Fn(&str, Result<T, <T as FromStr>::Err>) -> Result<(), &'a str> + 'a;
type Formatter<'a, T> = dyn Fn(&Number<T>, DrawTime) -> (String, Option<[u16; 2]>) + 'a;

pub struct Number<'a, T: Num> {
    pub message: &'a str,
    pub input: TextInput,
    pub placeholder: Option<&'a str>,
    pub default_value: Option<String>,
    pub validator_result: Result<(), &'a str>,
    validator: Option<Box<InputValidator<'a, T>>>,
    formatter: Box<Formatter<'a, T>>,
}

impl<'a, T: Num + 'a> Number<'a, T> {
    pub fn new(message: &'a str) -> Self {
        Number {
            message,
            input: TextInput::new(),
            placeholder: None,
            default_value: None,
            validator: None,
            validator_result: Ok(()),
            formatter: Box::new(theme::fmt_number),
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

    pub fn format<F>(&mut self, formatter: F) -> &mut Self
    where
        F: Fn(&Number<T>, DrawTime) -> (String, Option<[u16; 2]>) + 'a,
    {
        self.formatter = Box::new(formatter);
        self
    }

    pub fn prompt(&mut self) -> io::Result<Result<T, T::Err>> {
        key_listener::listen(self)?;
        Ok(self.get_value())
    }
}

impl<T: Num> Number<'_, T> {
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
            _ => ch.is_ascii_digit(),
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

impl<T: Num> Typeable for Number<'_, T> {
    fn handle_key(&mut self, key: KeyEvent) -> bool {
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

        submit
    }
}

impl<T: Num> Printable for Number<'_, T> {
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
    fn set_custom_formatter() {
        let mut prompt: Number<u8> = Number::new("");
        let draw_time = DrawTime::First;
        const EXPECTED_VALUE: &str = "foo";

        prompt.format(|_, _| (String::from(EXPECTED_VALUE), None));

        assert_eq!(
            (prompt.formatter)(&prompt, draw_time),
            (String::from(EXPECTED_VALUE), None)
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
        ('a'..='z').for_each(|c| {
            prompt.handle_key(KeyEvent::from(KeyCode::Char(c)));
        });

        // try to type digits
        ('0'..='9').for_each(|c| {
            prompt.handle_key(KeyEvent::from(KeyCode::Char(c)));
        });

        assert_eq!(prompt.input.value, "0123456789");
    }

    #[test]
    fn allow_decimal_in_floats() {
        let mut prompt = Number::<f32>::new("");

        "1.".chars().for_each(|c| {
            prompt.handle_key(KeyEvent::from(KeyCode::Char(c)));
        });

        assert_eq!(prompt.input.value, "1.");

        // not allow in integers
        let mut prompt = Number::<i32>::new("");

        "2.".chars().for_each(|c| {
            prompt.handle_key(KeyEvent::from(KeyCode::Char(c)));
        });

        assert_eq!(prompt.input.value, "2");
    }
}
