use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, Typeable},
    renderer::{DrawTime, Printable, Renderer},
    theme,
};

type Formatter<'a> = dyn Fn(&Toggle, DrawTime) -> String + 'a;

/// Prompt to choose between two options.
///
/// # Key Events
///
/// | Key                  | Action                       |
/// | -------------------- | ---------------------------- |
/// | `Enter`, `Backspace` | Submit current/initial value |
/// | `Left`, `h`, `H`     | Focus `false`                |
/// | `Right`, `l`, `L`    | Focus `true`                 |
///
/// # Examples
///
/// ```no_run
/// use asky::Toggle;
///
/// # fn main() -> std::io::Result<()> {
/// let os = Toggle::new("What is your favorite OS?", ["Android", "IOS"]).prompt()?;
///
/// println!("{os} is the best!");
/// # Ok(())
/// # }
/// ```
pub struct Toggle<'a> {
    /// Message used to display in the prompt.
    pub message: &'a str,
    /// Options to display in the prompt.
    pub options: [&'a str; 2],
    /// Current state of the prompt.
    pub active: bool,
    formatter: Box<Formatter<'a>>,
}

impl<'a> Toggle<'a> {
    /// Create a new toggle prompt.
    pub fn new(message: &'a str, options: [&'a str; 2]) -> Self {
        Toggle {
            message,
            options,
            active: false,
            formatter: Box::new(theme::fmt_toggle),
        }
    }

    /// Set whether the prompt should be active at start.
    pub fn initial(&mut self, value: bool) -> &mut Self {
        self.active = value;
        self
    }

    /// Set custom closure to format the prompt.
    ///
    /// See: [`Customization`](index.html#customization).
    pub fn format<F>(&mut self, formatter: F) -> &mut Self
    where
        F: Fn(&Toggle, DrawTime) -> String + 'a,
    {
        self.formatter = Box::new(formatter);
        self
    }

    /// Display the prompt and return the user answer.
    pub fn prompt(&mut self) -> io::Result<String> {
        key_listener::listen(self, true)?;
        Ok(String::from(self.get_value()))
    }
}

impl Toggle<'_> {
    fn get_value(&self) -> &str {
        self.options[self.active as usize]
    }
}

impl Typeable for Toggle<'_> {
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        let mut submit = false;

        match key.code {
            // submit focused/initial option
            KeyCode::Enter | KeyCode::Backspace => submit = true,
            // update focus option
            KeyCode::Left | KeyCode::Char('h' | 'H') => self.active = false,
            KeyCode::Right | KeyCode::Char('l' | 'L') => self.active = true,
            _ => (),
        }

        submit
    }
}

impl Printable for Toggle<'_> {
    fn draw(&self, renderer: &mut Renderer) -> io::Result<()> {
        let text = (self.formatter)(self, renderer.draw_time);
        renderer.print(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_initial_value() {
        let mut prompt = Toggle::new("", ["foo", "bar"]);

        prompt.initial(false);
        assert_eq!(prompt.get_value(), "foo");
        prompt.initial(true);
        assert_eq!(prompt.get_value(), "bar");
    }

    #[test]
    fn set_custom_formatter() {
        let mut prompt = Toggle::new("", ["foo", "bar"]);
        let draw_time = DrawTime::First;
        const EXPECTED_VALUE: &str = "foo";

        prompt.format(|_, _| String::from(EXPECTED_VALUE));

        assert_eq!((prompt.formatter)(&prompt, draw_time), EXPECTED_VALUE);
    }

    #[test]
    fn sumit_focused() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = Toggle::new("", ["foo", "bar"]);
            let simulated_key = KeyEvent::from(event);

            let submit = prompt.handle_key(simulated_key);
            assert_eq!(prompt.get_value(), "foo");
            assert!(submit);
        }
    }

    #[test]
    fn update_focused() {
        let events = [
            (KeyCode::Left, true, "foo"),
            (KeyCode::Char('h'), true, "foo"),
            (KeyCode::Char('H'), true, "foo"),
            (KeyCode::Right, false, "bar"),
            (KeyCode::Char('l'), false, "bar"),
            (KeyCode::Char('L'), false, "bar"),
        ];

        for (key, initial, expected) in events {
            let mut prompt = Toggle::new("", ["foo", "bar"]);
            let simulated_key = KeyEvent::from(key);

            prompt.initial(initial);
            let submit = prompt.handle_key(simulated_key);

            assert_eq!(prompt.get_value(), expected);
            assert!(!submit);
        }
    }
}
