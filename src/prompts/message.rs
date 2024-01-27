use std::borrow::Cow;
use std::io;

use crate::style::Style;
#[cfg(feature = "terminal")]
use crate::utils::key_listener::listen;
use crate::utils::renderer::{Printable, Renderer};
use crate::Error;
use crate::Valuable;

/// Prompt to ask yes/no questions.
///
/// # Key Events
///
/// | Key                  | Action                       |
/// | -------------------- | ---------------------------- |
/// | `Enter`, `Backspace` | Submit current/initial value |
/// | `y`, `Y`             | Submit `true`                |
/// | `n`, `N`             | Submit `false`               |
/// | `Left`, `h`, `H`     | Focus `false`                |
/// | `Right`, `l`, `L`    | Focus `true`                 |
///
/// # Examples
///
/// ```no_run
/// use asky::Message;
///
/// # fn main() -> std::io::Result<()> {
/// # #[cfg(feature = "terminal")]
/// Message::new("Well, that's great.").prompt()?;
/// # Ok(())
/// # }
/// ```
// #[derive(Debug)]
pub struct Message<'a> {
    /// Message used to display in the prompt.
    // pub message: &'a str,
    pub message: Cow<'a, str>,
    pub action: Option<Cow<'a, str>>,
    // pub wait_for_any_key: bool,
}

impl Valuable for Message<'_> {
    type Output = ();
    fn value(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> Message<'a> {
    /// Create a new message prompt with an call to action, e.g., "Press Any Key".
    pub fn with_option<T: Into<Cow<'a, str>>>(message: T, action: T) -> Self {
        Message {
            message: message.into(),
            action: Some(action.into()),
        }
    }

    /// Create a new confirm prompt.
    pub fn new<T: Into<Cow<'a, str>>>(message: T) -> Self {
        Message {
            message: message.into(),
            action: None,
        }
    }

    #[cfg(feature = "terminal")]
    /// Display the prompt and return the user answer.
    pub fn prompt(&mut self) -> io::Result<()> {
        listen(self, true)
    }
}

impl Printable for Message<'_> {
    fn draw_with_style<R: Renderer, S: Style>(&self, r: &mut R, style: &S) -> io::Result<()> {
        use crate::style::Section::*;
        r.pre_prompt()?;
        style.begin(r, Message)?;
        write!(r, "{}", self.message)?;
        style.end(r, Message)?;
        r.post_prompt(1)
    }
}

#[cfg(feature = "terminal")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DrawTime, utils::key_listener::Typeable};
    use crossterm::event::{KeyCode, KeyEvent};

    // #[test]
    // fn set_custom_formatter() {
    //     let mut prompt: Message = Message::new("");
    //     let draw_time = DrawTime::First;
    //     const EXPECTED_VALUE: &str = "foo";

    //     prompt.format(|_, _, out| out.push(EXPECTED_VALUE.into()));
    //     let mut out = ColoredStrings::new();
    //     (prompt.formatter)(&prompt, draw_time, &mut out);
    //     assert_eq!(format!("{}", out), EXPECTED_VALUE);
    // }

    #[test]
    fn update_and_submit() {
        let events = [('y', true), ('Y', true), ('n', false), ('N', false)];

        for (char, _expected) in events {
            let mut prompt = Message::new("");
            let simulated_key = KeyEvent::from(KeyCode::Char(char));

            // prompt.initial(!expected);
            let submit = prompt.handle_key(&simulated_key);

            assert!(submit);
        }
    }

    #[test]
    fn submit_focused() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = Message::new("");
            let simulated_key = KeyEvent::from(event);

            let submit = prompt.handle_key(&simulated_key);
            // assert!(!prompt.active);
            assert!(submit);
        }
    }

    #[test]
    fn update_focused() {
        let events = [
            (KeyCode::Left, true, false),
            (KeyCode::Char('h'), true, false),
            (KeyCode::Char('H'), true, false),
            (KeyCode::Right, false, true),
            (KeyCode::Char('l'), false, true),
            (KeyCode::Char('L'), false, true),
        ];

        for (key, _initial, _expected) in events {
            let mut prompt = Message::new("");
            let simulated_key = KeyEvent::from(key);

            // prompt.initial(initial);
            let submit = prompt.handle_key(&simulated_key);

            // assert_eq!(prompt.active, expected);
            assert!(submit);
        }
    }
}
