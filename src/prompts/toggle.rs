use std::borrow::Cow;
use std::io;

use crate::Error;
use crate::Valuable;

use crate::style::{Section, Style};
use crate::utils::{
    renderer::{DrawTime, Printable, Renderer},
};

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
}

impl Valuable for Toggle<'_> {
    type Output = bool;
    fn value(&self) -> Result<bool, Error> {
        Ok(self.active)
    }
}

impl<'a> Toggle<'a> {
    /// Create a new toggle prompt.
    pub fn new(
        message: impl Into<Cow<'a, str>>,
        a: impl Into<Cow<'a, str>>,
        b: impl Into<Cow<'a, str>>,
    ) -> Self {
        Toggle {
            message: message.into(),
            options: [a.into(), b.into()],
            active: false,
        }
    }

    /// Set whether the prompt should be active at start.
    pub fn initial(&mut self, value: bool) -> &mut Self {
        self.active = value;
        self
    }

}

impl Printable for Toggle<'_> {
    fn draw_with_style<R: Renderer, S: Style>(&self, r: &mut R, style: &S) -> io::Result<()> {
        use Section::*;
        let draw_time = r.draw_time();
        // let style = DefaultStyle { ascii: true };

        r.print_prompt(|r| {
            if draw_time == DrawTime::Last {
                style.begin(r, Query(true))?;
                write!(r, "{}", self.message)?;
                style.end(r, Query(true))?;

                style.begin(r, Answer(true))?;
                write!(r, "{}", &self.options[self.active as usize])?;
                style.end(r, Answer(true))?;
                Ok(1)
            } else {
                style.begin(r, Query(false))?;
                write!(r, "{}", self.message)?;
                style.end(r, Query(false))?;

                style.begin(r, Toggle(!self.active))?;
                write!(r, "{}", &self.options[0])?;
                style.end(r, Toggle(!self.active))?;
                style.begin(r, Toggle(self.active))?;
                write!(r, "{}", &self.options[1])?;
                style.end(r, Toggle(self.active))?;
                Ok(2)
            }
        })
    }
}

#[cfg(feature = "terminal")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::key_listener::Typeable;
    use crossterm::event::{KeyCode, KeyEvent};

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

    // #[test]
    // fn set_custom_formatter() {
    //     let mut prompt = Toggle::new("", "foo", "bar");
    //     let draw_time = DrawTime::First;
    //     const EXPECTED_VALUE: &str = "foo";

    //     prompt.format(|_, _, out| out.push(EXPECTED_VALUE.into()));
    //     let mut out = ColoredStrings::new();
    //     (prompt.formatter)(&prompt, draw_time, &mut out);
    //     assert_eq!(format!("{}", out), EXPECTED_VALUE);
    // }

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
