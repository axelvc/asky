use std::io;
use crate::Error;
#[cfg(feature = "bevy")]
use crate::bevy as cbevy;
#[cfg(feature = "bevy")]
use bevy::input::keyboard::KeyCode as BKeyCode;
#[cfg(feature = "terminal")]
use crossterm::event::{KeyCode, KeyEvent};

#[cfg(feature = "terminal")]
use crate::utils::key_listener;
use crate::utils::key_listener::Typeable;
use crate::utils::{
    num_like::NumLike,
    renderer::{DrawTime, Printable, Renderer},
    theme,
};
use crate::Valuable;

use super::text::{Direction, LineInput};
use crate::ColoredStrings;

type InputValidator<'a, T> =
    dyn Fn(&str, Result<T, Error>) -> Result<(), &'a str> + 'a + Send + Sync;
type Formatter<'a, T> =
    dyn Fn(&Number<T>, DrawTime, &mut ColoredStrings) -> [usize; 2] + 'a + Send + Sync;

/// Prompt to get one-line user input of numbers.
///
/// Similar to [`Text`] prompt, but only accept numbers, decimal point [^decimal], and sign symbol [^sign].
///
/// # Key Events
///
/// | Key         | Action                       |
/// | ----------- | ---------------------------- |
/// | `Enter`     | Submit current/initial value |
/// | `Backspace` | Delete previous character    |
/// | `Delete`    | Delete current character     |
/// | `Left`      | Move cursor left             |
/// | `Right`     | Move cursor right            |
/// | `Backspace` | Delete previous character    |
/// | `.`         | Add decimal point [^decimal]  |
/// | `-`, `+`    | Add sign to the input [^sign] |
///
/// [^decimal]: Only for floating values.
///
/// [^sign]:  Only for signed values and when cursor is at start of the input.
///
/// # Examples
///
/// ```no_run
/// use asky::Number;
///
/// # fn main() -> std::io::Result<()> {
/// let number = Number::<u8>::new("How many pets do you have?").prompt()?;
/// # Ok(())
/// # }
/// ```
/// [`Text`]: crate::Text
pub struct Number<'a, T: NumLike> {
    /// Message used to display in the prompt.
    pub message: &'a str,
    /// Input state for the prompt.
    pub input: LineInput,
    /// Placeholder to show when the input is empty.
    pub placeholder: Option<&'a str>,
    /// Default value to submit when the input is empty.
    pub default_value: Option<T>,
    /// State of the validation of the user input.
    pub validator_result: Result<(), &'a str>,
    validator: Option<Box<InputValidator<'a, T>>>,
    formatter: Box<Formatter<'a, T>>,
}

impl<T: NumLike + Send> Valuable for Number<'_, T> {
    type Output = T;
    fn value(&self) -> Result<T, Error> {
        // XXX: How do I convert T::Err into a string?
        self.get_value()
    }
}

impl<'a, T: NumLike + 'a> Number<'a, T> {
    /// Create a new number prompt.
    pub fn new(message: &'a str) -> Self {
        Number {
            message,
            input: LineInput::new(),
            placeholder: None,
            default_value: None,
            validator: None,
            validator_result: Ok(()),
            formatter: Box::new(theme::fmt_number2),
        }
    }

    /// Set text to show when the input is empty.
    ///
    /// This not will not be submitted when the input is empty.
    pub fn placeholder(&mut self, value: &'a str) -> &mut Self {
        self.placeholder = Some(value);
        self
    }

    /// Set default value to submit when the input is empty.
    pub fn default(&mut self, value: T) -> &mut Self {
        self.default_value = Some(value);
        self
    }

    /// Set initial value, could be deleted by the user.
    pub fn initial(&mut self, value: T) -> &mut Self {
        self.input.set_value(&value.to_string());
        self
    }

    /// Set validator to the user input.
    pub fn validate<F>(&mut self, validator: F) -> &mut Self
    where
        F: Fn(&str, Result<T, Error>) -> Result<(), &'a str> + 'static + Send + Sync,
    {
        self.validator = Some(Box::new(validator));
        self
    }

    /// Set custom closure to format the prompt.
    ///
    /// See: [`Customization`](index.html#customization).
    pub fn format<F>(&mut self, formatter: F) -> &mut Self
    where
        F: Fn(&Number<T>, DrawTime, &mut ColoredStrings) -> [usize; 2] + 'a + Send + Sync,
    {
        self.formatter = Box::new(formatter);
        self
    }

    #[cfg(feature = "terminal")]
    /// Display the prompt and return the user answer.
    pub fn prompt(&mut self) -> io::Result<Result<T, Error>> {
        key_listener::listen(self, false)?;
        Ok(self.get_value())
    }
}

impl<T: NumLike> Number<'_, T> {
    fn get_value(&self) -> Result<T, Error> {
        match self.input.value.is_empty() {
            // FIXME: This is not good behavior, right?
            true => match self.default_value {
                Some(v) => Ok(v),
                None => Err(Error::InvalidInput)
            }
            false => self.input.value.parse().map_err(|_| Error::InvalidInput),
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

#[cfg(feature = "terminal")]
impl<T: NumLike> Typeable<KeyEvent> for Number<'_, T> {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
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

#[cfg(feature = "bevy")]
impl<T: NumLike> Typeable<cbevy::KeyEvent> for Number<'_, T> {
    fn will_handle_key(&self, key: &cbevy::KeyEvent) -> bool {
        for c in key.chars.iter() {
            if !c.is_control() {
                return true;
            }
        }

        for code in &key.codes {
            if match code {
                // submit
                BKeyCode::Return => true,
                // type
                // BKeyCode::Char(c) => self.input.insert(c),
                // remove delete
                BKeyCode::Back => true,
                BKeyCode::Delete => true,
                // move cursor
                BKeyCode::Left => true,
                BKeyCode::Right => true,
                _ => false,
            } {
                return true;
            }
        }

        false
    }
    fn handle_key(&mut self, key: &cbevy::KeyEvent) -> bool {
        let mut submit = false;

        for c in key.chars.iter() {
            if !c.is_control() {
                self.insert(*c);
            }
        }

        for code in &key.codes {
            match code {
                // submit
                BKeyCode::Return => submit = self.validate_to_submit(),
                // type
                // BKeyCode::Char(c) => self.input.insert(c),
                // remove delete
                BKeyCode::Back => self.input.backspace(),
                BKeyCode::Delete => self.input.delete(),
                // move cursor
                BKeyCode::Left => self.input.move_cursor(Direction::Left),
                BKeyCode::Right => self.input.move_cursor(Direction::Right),
                _ => (),
            };
        }

        submit
    }
}

#[cfg(feature = "terminal")]
impl<T: NumLike> Printable for Number<'_, T> {
    fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
        let mut out = ColoredStrings::default();
        let cursor = (self.formatter)(self, renderer.draw_time(), &mut out);
        renderer.print(out)?;
        renderer.set_cursor(cursor)
    }
}

#[cfg(feature = "bevy")]
impl<T: NumLike> Printable for cbevy::AskyNode<Number<'_, T>> {
    fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
        let mut out = ColoredStrings::default();
        let cursor = (self.formatter)(self, renderer.draw_time(), &mut out);
        renderer.show_cursor()?;
        renderer.set_cursor(cursor)?;
        renderer.print(out)
    }
}

impl<'a, T: NumLike + 'a> Default for Number<'a, T> {
    fn default() -> Self {
        Self::new("")
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
        assert_eq!(text.default_value, Some(10));
    }

    #[test]
    fn set_initial_value() {
        let mut prompt = Number::<i32>::new("");

        assert_eq!(prompt.input, LineInput::new());

        prompt.initial(10);

        assert_eq!(
            prompt.input,
            LineInput {
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

        prompt.format(|_, _, out| {
            out.push(EXPECTED_VALUE.into());
            [0, 0]
        });
        let mut out = ColoredStrings::new();
        assert_eq!((prompt.formatter)(&prompt, draw_time, &mut out), [0, 0]);
        assert_eq!(format!("{}", out), EXPECTED_VALUE);
    }

    #[test]
    fn update_cursor_position() {
        let mut prompt = Number::<i32>::new("");
        prompt.input.set_value("foo");
        prompt.input.col = 2;

        let keys = [(KeyCode::Left, 1), (KeyCode::Right, 2)];

        for (key, expected) in keys {
            prompt.handle_key(&KeyEvent::from(key));

            assert_eq!(prompt.input.col, expected);
        }
    }

    #[test]
    fn submit_input_value() {
        let mut prompt = Number::<i32>::new("");
        prompt.input.set_value(&String::from("10"));
        prompt.default(20);

        assert_eq!(prompt.get_value().unwrap(), 10);
    }

    #[test]
    fn submit_default_value() {
        let mut prompt = Number::<i32>::new("");
        prompt.input.set_value("");
        prompt.default(20);

        assert_eq!(prompt.get_value().unwrap(), 20);
    }

    #[test]
    fn allow_sign_at_the_start() {
        let signs = ['-', '+'];

        for c in signs {
            let mut prompt = Number::<i32>::new("");

            // must accept only one sign, simulate double press
            prompt.handle_key(&KeyEvent::from(KeyCode::Char(c)));
            prompt.handle_key(&KeyEvent::from(KeyCode::Char(c)));

            assert_eq!(prompt.input.value, c.to_string());
        }

        // not allow fo unsigned types
        for c in signs {
            let mut prompt = Number::<u32>::new("");

            prompt.handle_key(&KeyEvent::from(KeyCode::Char(c)));

            assert!(prompt.input.value.is_empty());
        }
    }

    #[test]
    fn allow_only_digits() {
        let mut prompt = Number::<i32>::new("");

        // try to type a character
        ('a'..='z').for_each(|c| {
            prompt.handle_key(&KeyEvent::from(KeyCode::Char(c)));
        });

        // try to type digits
        ('0'..='9').for_each(|c| {
            prompt.handle_key(&KeyEvent::from(KeyCode::Char(c)));
        });

        assert_eq!(prompt.input.value, "0123456789");
    }

    #[test]
    fn allow_decimal_in_floats() {
        let mut prompt = Number::<f32>::new("");

        "1.".chars().for_each(|c| {
            prompt.handle_key(&KeyEvent::from(KeyCode::Char(c)));
        });

        assert_eq!(prompt.input.value, "1.");

        // not allow in integers
        let mut prompt = Number::<i32>::new("");

        "2.".chars().for_each(|c| {
            prompt.handle_key(&KeyEvent::from(KeyCode::Char(c)));
        });

        assert_eq!(prompt.input.value, "2");
    }
}
