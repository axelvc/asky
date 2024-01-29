use crate::Error;

use super::text::LineInput;
use crate::Valuable;
use std::borrow::Cow;

use crate::style::Style;
use crate::utils::{
    num_like::NumLike,
    renderer::{DrawTime, Printable, Renderer},
};
use std::io;

type InputValidator<'a, T> =
    dyn Fn(&str, Result<T, Error>) -> Result<(), &'a str> + 'a + Send + Sync;

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
/// use asky::prelude::*;
///
/// # fn main() -> Result<(), Error> {
/// # #[cfg(feature = "terminal")]
/// let number = Number::<u8>::new("How many pets do you have?").prompt()?;
/// # Ok(())
/// # }
/// ```
/// [`Text`]: crate::Text
pub struct Number<'a, T: NumLike> {
    /// Message used to display in the prompt.
    pub message: Cow<'a, str>,
    /// Input state for the prompt.
    pub input: LineInput,
    /// Placeholder to show when the input is empty.
    pub placeholder: Option<&'a str>,
    /// Default value to submit when the input is empty.
    pub default_value: Option<T>,
    /// State of the validation of the user input.
    pub validator_result: Result<(), &'a str>,
    validator: Option<Box<InputValidator<'a, T>>>,
}

impl<T: NumLike + Send> Valuable for Number<'_, T> {
    type Output = T;
    fn value(&self) -> Result<T, Error> {
        // XXX: How do I convert T::Err into a string?
        match self.input.value.is_empty() {
            // FIXME: This is not good behavior, right?
            true => match self.default_value {
                Some(v) => Ok(v),
                None => Err(Error::InvalidInput),
            },
            false => self.input.value.parse().map_err(|_| Error::InvalidInput),
        }
    }
}

impl<'a, T: NumLike + 'a> Number<'a, T> {
    /// Create a new number prompt.
    pub fn new(message: impl Into<Cow<'a, str>>) -> Self {
        Number {
            message: message.into(),
            input: LineInput::new(),
            placeholder: None,
            default_value: None,
            validator: None,
            validator_result: Ok(()),
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
}

impl<T: NumLike> Number<'_, T> {
    pub(crate) fn insert(&mut self, ch: char) {
        let is_valid = match ch {
            '-' | '+' => T::is_signed() && self.input.col == 0,
            '.' => T::is_float() && !self.input.value.contains('.'),
            _ => ch.is_ascii_digit(),
        };

        if is_valid {
            self.input.insert(ch)
        }
    }

    pub(crate) fn validate_to_submit(&mut self) -> bool {
        if let Some(validator) = &self.validator {
            self.validator_result = validator(&self.input.value, self.value());
        }

        self.validator_result.is_ok()
    }
}

impl<T: NumLike> Printable for Number<'_, T> {
    fn hide_cursor(&self) -> bool {
        false
    }
    fn draw_with_style<R: Renderer, S: Style>(&self, r: &mut R, style: &S) -> io::Result<()> {
        use crate::style::Section::*;
        let draw_time = r.draw_time();

        r.pre_prompt()?;

        if draw_time == DrawTime::Last {
            style.begin(r, Query(true))?;
            write!(r, "{}", self.message)?;
            style.end(r, Query(true))?;

            style.begin(r, Answer(true))?;
            write!(r, "{}", &self.input.value)?;
            style.end(r, Answer(true))?;
        } else {
            style.begin(r, Query(false))?;
            write!(r, "{}", self.message)?;
            style.end(r, Query(false))?;

            if let Some(x) = self.default_value {
                style.begin(r, DefaultAnswer)?;
                write!(r, "{}", x)?;
                style.end(r, DefaultAnswer)?;
            }
            style.begin(r, Input)?;
            r.save_cursor()?;
            write!(r, "{}", &self.input.value)?;
            if self.input.value.is_empty() {
                if let Some(placeholder) = self.placeholder {
                    style.begin(r, Placeholder)?;
                    write!(r, "{}", placeholder)?;
                    style.end(r, Placeholder)?;
                }
            }
            let is_valid = self.validator_result.is_ok();
            if let Err(_error) = self.validator_result {
                style.begin(r, Validator(is_valid))?;
                style.begin(r, Input)?;
                write!(r, "{}", &self.input.value)?;
                style.end(r, Input)?;
                style.end(r, Validator(is_valid))?;
            }
        };
        r.restore_cursor()?;
        r.move_cursor([self.input.col, 0])
    }
}

#[cfg(feature = "terminal")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::key_listener::Typeable;
    use crate::Valuable;
    use crossterm::event::{KeyCode, KeyEvent};

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

    // #[test]
    // fn set_custom_formatter() {
    //     let mut prompt: Number<u8> = Number::new("");
    //     let draw_time = DrawTime::First;
    //     const EXPECTED_VALUE: &str = "foo";

    //     prompt.format(|_, _, out| {
    //         out.push(EXPECTED_VALUE.into());
    //         [0, 0]
    //     });
    //     let mut out = ColoredStrings::new();
    //     assert_eq!((prompt.formatter)(&prompt, draw_time, &mut out), [0, 0]);
    //     assert_eq!(format!("{}", out), EXPECTED_VALUE);
    // }

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

        assert_eq!(prompt.value().unwrap(), 10);
    }

    #[test]
    fn submit_default_value() {
        let mut prompt = Number::<i32>::new("");
        prompt.input.set_value("");
        prompt.default(20);

        assert_eq!(prompt.value().unwrap(), 20);
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
