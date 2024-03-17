use std::borrow::Cow;
use std::io;

use crate::style::Style;
#[cfg(feature = "terminal")]
use crate::utils::key_listener::listen;
use crate::utils::renderer::{Printable, Renderer};
use crate::{DrawTime, Error, Valuable};

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
    pub wait_for_key: bool,
}

impl Valuable for Message<'_> {
    type Output = ();
    fn value(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl<'a> Message<'a> {
    /// Create a new message prompt with an call to action, e.g., "Press Any Key".
    pub fn wait<T: Into<Cow<'a, str>>>(message: T) -> Self {
        Message {
            message: message.into(),
            action: None,
            wait_for_key: true,
        }
    }

    pub fn call_to_action<T: Into<Cow<'a, str>>>(message: T, action: T) -> Self {
        Message {
            message: message.into(),
            action: Some(action.into()),
            wait_for_key: true,
        }
    }

    /// Create a new confirm prompt.
    pub fn new<T: Into<Cow<'a, str>>>(message: T) -> Self {
        Message {
            message: message.into(),
            action: None,
            wait_for_key: false,
        }
    }

    #[cfg(feature = "terminal")]
    /// Display the prompt and return the user answer.
    pub fn prompt(&mut self) -> io::Result<()> {
        listen(self)
    }
}

impl Printable for Message<'_> {
    fn draw_with_style<R: Renderer, S: Style>(&self, r: &mut R, style: &S) -> io::Result<()> {
        use crate::style::Section::*;
        let draw_time = r.draw_time();
        r.pre_prompt()?;
        if ! self.wait_for_key {
            // XXX: This is a little funky. It'd be better to pass done some other way.
            r.update_draw_time();
            r.update_draw_time();
        }
        if draw_time == DrawTime::Last {
            style.begin(r, Message)?;
            write!(r, "{}", self.message)?;
            style.end(r, Message)?;
        } else {
            style.begin(r, Message)?;
            write!(r, "{}", self.message)?;
            if let Some(action) = &self.action {
                style.begin(r, Action)?;
                write!(r, "{}", action)?;
                style.end(r, Action)?;
            }
            style.end(r, Message)?;
        }
        r.post_prompt()
    }
}

#[cfg(feature = "terminal")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{utils::key_listener::Typeable};
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
