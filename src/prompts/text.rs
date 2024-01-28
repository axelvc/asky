use crate::Error;
use std::borrow::Cow;

use crate::style::{NoStyle, Style};
use crate::utils::renderer::{DrawTime, Printable, Renderer};
use crate::Valuable;
use std::io;

pub enum Direction {
    Left,
    Right,
}

// region: TextInput

/// State of the user input for read-line text prompts (like [`Text`]).
///
/// **Note**: This structure is not expected to be created, but it can be consumed when using a custom formatter.
#[derive(Debug, PartialEq, Eq, Default)]
pub struct LineInput {
    /// Current value of the input.
    pub value: String,
    /// Current position of the cursor.
    pub col: usize,
}

impl LineInput {
    pub(crate) fn new() -> Self {
        LineInput::default()
    }
}

impl LineInput {
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

// endregion: TextInput

pub type InputValidator<'a> = dyn Fn(&str) -> Result<(), &'a str> + 'a + Send + Sync;

/// Prompt to get one-line user input.
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
///
/// # Examples
///
/// ```no_run
/// use asky::prelude::*;
///
/// # fn main() -> Result<(), Error> {
/// # #[cfg(feature = "terminal")]
/// let name = Text::new("What is your name?").prompt()?;
///
/// # #[cfg(feature = "terminal")]
/// println!("Hello, {}!", name);
///
/// # Ok(())
/// # }
/// ```
pub struct Text<'a> {
    /// Message used to display in the prompt
    pub message: Cow<'a, str>,
    /// Input state for the prompt
    pub input: LineInput,
    /// Placeholder to show when the input is empty
    pub placeholder: Option<&'a str>,
    /// Default value to submit when the input is empty
    pub default_value: Option<&'a str>,
    /// State of the validation of the user input
    pub validator_result: Result<(), &'a str>,
    validator: Option<Box<InputValidator<'a>>>,
}

impl<'a> Valuable for Text<'a> {
    type Output = String;
    fn value(&self) -> Result<String, Error> {
        Ok(self.input.value.to_string())
    }
}
impl<'a> Text<'a> {
    /// Create a new text prompt.
    pub fn new(message: impl Into<Cow<'a, str>>) -> Self {
        Text {
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
    pub fn default(&mut self, value: &'a str) -> &mut Self {
        self.default_value = Some(value);
        self
    }

    /// Set initial value, could be deleted by the user.
    pub fn initial(&mut self, value: &str) -> &mut Self {
        self.input.set_value(value);
        self
    }

    /// Set validator to the user input.
    pub fn validate<F>(&mut self, validator: F) -> &mut Self
    where
        F: Fn(&str) -> Result<(), &'a str> + 'a + Send + Sync,
    {
        self.validator = Some(Box::new(validator));
        self
    }
}

impl Text<'_> {
    fn get_value(&self) -> &str {
        match self.input.value.is_empty() {
            true => self.default_value.unwrap_or_default(),
            false => &self.input.value,
        }
    }

    pub(crate) fn validate_to_submit(&mut self) -> bool {
        if let Some(validator) = &self.validator {
            self.validator_result = validator(self.get_value());
        }

        self.validator_result.is_ok()
    }
}

impl Printable for Text<'_> {
    fn hide_cursor(&self) -> bool {
        false
    }
    fn draw_with_style<R: Renderer, S: Style>(&self, r: &mut R, style: &S) -> io::Result<()> {
        use crate::style::Section::*;
        let draw_time = r.draw_time();

        r.pre_prompt()?;
        let line_count = if draw_time == DrawTime::Last {
            style.begin(r, Query(true))?;
            write!(r, "{}", self.message)?;
            style.end(r, Query(true))?;

            style.begin(r, Answer(true))?;
            write!(r, "{}", &self.input.value)?;
            style.end(r, Answer(true))?;
            1
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
            style.end(r, Input)?;
            if let Err(error) = self.validator_result {
                style.begin(r, Validator(false))?;
                write!(r, "{}", error)?;
                style.end(r, Validator(false))?;
            }
            1
        };

        // assert_eq!(r.newline_count(), &line_count);
        // assert_eq!(r.newline_count(), &0);
        let line_count = *r.newline_count();
        r.post_prompt(line_count)?;
        r.set_cursor([self.input.col, 0])
    }
}

#[cfg(feature = "terminal")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{key_listener::Typeable, renderer::StringRenderer};
    use crossterm::event::{KeyCode, KeyEvent};
    use std::io::Write;

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

        assert_eq!(prompt.input, LineInput::new());

        prompt.initial("foo");

        assert_eq!(
            prompt.input,
            LineInput {
                value: String::from("foo"),
                col: 3,
            }
        );
    }

    #[test]
    fn set_custom_formatter() {
        let mut prompt: Text = Text::new("");
        let draw_time = DrawTime::First;
        const EXPECTED_VALUE: &str = "foo";
        let styled_prompt =
            prompt.with_format(|_, renderer| write!(renderer, "{}", EXPECTED_VALUE));
        let mut out = StringRenderer::default();
        let _ = styled_prompt.draw(&mut out);
        assert_eq!(out.string, EXPECTED_VALUE);
    }

    #[test]
    fn set_custom_style() {
        const EXPECTED_VALUE: &str = "foo";
        let mut prompt: Text = Text::new(EXPECTED_VALUE);
        let draw_time = DrawTime::First;
        let styled_prompt = prompt.with_style(NoStyle);
        let mut out = StringRenderer::default();
        let _ = styled_prompt.draw(&mut out);
        assert_eq!(out.string, EXPECTED_VALUE);
    }

    #[test]
    fn update_value() {
        let mut prompt = Text::new("");

        // simulate typing
        let text = "foo";

        for char in text.chars() {
            prompt.handle_key(&KeyEvent::from(KeyCode::Char(char)));
        }

        assert_eq!(prompt.input.value, "foo");
        assert_eq!(prompt.input.col, 3);

        // removing
        let keys = [(KeyCode::Backspace, "fo"), (KeyCode::Delete, "f")];
        prompt.input.col = 2;

        for (key, expected) in keys {
            prompt.handle_key(&KeyEvent::from(key));

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
            prompt.handle_key(&KeyEvent::from(key));

            assert_eq!(prompt.input.col, expected);
        }
    }

    #[test]
    fn validate_input() {
        let mut prompt = Text::new("");
        let err_str = "Please enter an response";

        prompt.validate(|s| if s.is_empty() { Err(err_str) } else { Ok(()) });

        // invalid value
        let mut submit = prompt.handle_key(&KeyEvent::from(KeyCode::Enter));

        assert!(!submit);
        assert_eq!(prompt.validator_result, Err(err_str));

        // valid value
        prompt.input.set_value("foo");
        submit = prompt.handle_key(&KeyEvent::from(KeyCode::Enter));

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
