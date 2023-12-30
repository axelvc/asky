use std::borrow::Cow;
use crate::ColoredStrings;
use std::io;

use crate::Error;
use crate::Valuable;

use crate::utils::{
    renderer::{DrawTime, Printable, Renderer},
    theme,
};


type Formatter<'a> = dyn Fn(&Toggle, DrawTime, &mut ColoredStrings) + 'a + Send + Sync;

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
/// use asky::prelude::*;
///
/// # fn main() -> Result<(), Error> {
/// # #[cfg(feature = "terminal")]
/// let os = Toggle::new("What is your favorite OS?", "Android", "iOS").prompt()?;
/// # #[cfg(not(feature = "terminal"))]
/// # let os = "iOS";
///
/// println!("{os} is the best!");
/// # Ok(())
/// # }
/// ```
pub struct Toggle<'a> {
    /// Message used to display in the prompt.
    pub message: Cow<'a, str>,
    /// Options to display in the prompt.
    pub options: [Cow<'a, str>; 2],
    /// Current state of the prompt.
    pub active: bool,
    pub(crate) formatter: Box<Formatter<'a>>,
}

impl Valuable for Toggle<'_> {
    type Output = bool;
    fn value(&self) -> Result<bool, Error> {
        Ok(self.active)
    }
}

impl<'a> Toggle<'a> {
    /// Create a new toggle prompt.
    pub fn new(message: impl Into<Cow<'a, str>>, a: impl Into<Cow<'a, str>>, b: impl Into<Cow<'a, str>>) -> Self {
        Toggle {
            message: message.into(),
            options: [a.into(), b.into()],
            active: false,
            formatter: Box::new(theme::fmt_toggle2),
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
        F: Fn(&Toggle, DrawTime, &mut ColoredStrings) + 'a + Send + Sync,
    {
        self.formatter = Box::new(formatter);
        self
    }

}

impl Printable for Toggle<'_> {
    fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
        let mut out = ColoredStrings::default();
        (self.formatter)(self, renderer.draw_time(), &mut out);
        renderer.print(out)
    }
}

#[cfg(feature = "terminal")]
#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent};
    use crate::utils::key_listener::Typeable;

    impl Toggle<'_> {
        fn get_value(&self) -> &str {
            self.options[self.active as usize].as_ref()
        }
    }

    #[test]
    fn set_initial_value() {
        let mut prompt = Toggle::new("", "foo", "bar");

        prompt.initial(false);
        assert_eq!(prompt.get_value(), "foo");
        prompt.initial(true);
        assert_eq!(prompt.get_value(), "bar");
    }

    #[test]
    fn set_custom_formatter() {
        let mut prompt = Toggle::new("", "foo", "bar");
        let draw_time = DrawTime::First;
        const EXPECTED_VALUE: &str = "foo";

        prompt.format(|_, _, out| out.push(EXPECTED_VALUE.into()));
        let mut out = ColoredStrings::new();
        (prompt.formatter)(&prompt, draw_time, &mut out);
        assert_eq!(format!("{}", out), EXPECTED_VALUE);
    }

    #[test]
    fn sumit_focused() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = Toggle::new("", "foo", "bar");
            let simulated_key = KeyEvent::from(event);

            let submit = prompt.handle_key(&simulated_key);
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
            let mut prompt = Toggle::new("", "foo", "bar");
            let simulated_key = KeyEvent::from(key);

            prompt.initial(initial);
            let submit = prompt.handle_key(&simulated_key);

            assert_eq!(prompt.get_value(), expected);
            assert!(!submit);
        }
    }
}
