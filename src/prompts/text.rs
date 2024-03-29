use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, Typeable},
    renderer::{DrawTime, Printable, Renderer},
    theme,
};

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

pub type InputValidator<'a> = dyn Fn(&str) -> Result<(), &'a str> + 'a;
type Formatter<'a> = dyn Fn(&Text, DrawTime) -> (String, [usize; 2]) + 'a;

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
/// use asky::Text;
///
/// # fn main() -> std::io::Result<()> {
/// let name = Text::new("What is your name?").prompt()?;
///
/// println!("Hello, {}!", name);
///
/// # Ok(())
/// # }
/// ```
pub struct Text<'a> {
    /// Message used to display in the prompt
    pub message: &'a str,
    /// Input state for the prompt
    pub input: LineInput,
    /// Placeholder to show when the input is empty
    pub placeholder: Option<&'a str>,
    /// Default value to submit when the input is empty
    pub default_value: Option<&'a str>,
    /// State of the validation of the user input
    pub validator_result: Result<(), &'a str>,
    validator: Option<Box<InputValidator<'a>>>,
    formatter: Box<Formatter<'a>>,
}

impl<'a> Text<'a> {
    /// Create a new text prompt.
    pub fn new(message: &'a str) -> Self {
        Text {
            message,
            input: LineInput::new(),
            placeholder: None,
            default_value: None,
            validator: None,
            validator_result: Ok(()),
            formatter: Box::new(theme::fmt_text),
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
        F: Fn(&str) -> Result<(), &'a str> + 'a,
    {
        self.validator = Some(Box::new(validator));
        self
    }

    /// Set custom closure to format the prompt.
    ///
    /// See: [`Customization`](index.html#customization).
    pub fn format<F>(&mut self, formatter: F) -> &mut Self
    where
        F: Fn(&Text, DrawTime) -> (String, [usize; 2]) + 'a,
    {
        self.formatter = Box::new(formatter);
        self
    }

    /// Display the prompt and return the user answer.
    pub fn prompt(&mut self) -> io::Result<String> {
        key_listener::listen(self, false)?;
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
    fn draw(&self, renderer: &mut Renderer) -> io::Result<()> {
        let (text, cursor) = (self.formatter)(self, renderer.draw_time);
        renderer.print(text)?;
        renderer.set_cursor(cursor)
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

        prompt.format(|_, _| (String::from(EXPECTED_VALUE), [0, 0]));

        assert_eq!(
            (prompt.formatter)(&prompt, draw_time),
            (String::from(EXPECTED_VALUE), [0, 0])
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
