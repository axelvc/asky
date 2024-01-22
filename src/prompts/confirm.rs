use std::io::{self, Write};

use std::borrow::Cow;

use crate::utils::renderer::{DrawTime, Printable, Renderer};
use crate::utils::theme;
use crate::Error;
use crate::Valuable;
// use colored::ColoredStrings;
use crate::ColoredStrings;
use crate::style::{DefaultStyle, Style, Section, Region};
use crossterm::{queue, style::{Print}};

type Formatter<'a> = dyn Fn(&Confirm, DrawTime, &mut ColoredStrings) + 'a + Send + Sync;

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
/// use asky::prelude::*;
///
/// # fn main() -> Result<(), Error> {
/// # #[cfg(feature = "terminal")]
/// if Confirm::new("Do you like the pizza?").prompt()? {
///     println!("Great!");
/// } else {
///     println!("Interesting!");
/// }
/// # Ok(())
/// # }
/// ```
// #[derive(Debug)]
pub struct Confirm<'a> {
    /// Message used to display in the prompt.
    pub message: Cow<'a, str>,
    /// Current state of the prompt.
    pub active: bool,
    /// Current formatter
    pub formatter: Box<Formatter<'a>>,
}

impl<'a> Confirm<'a> {
    /// Create a new confirm prompt.
    pub fn new<T: Into<Cow<'a, str>>>(message: T) -> Self {
        Confirm {
            message: message.into(),
            active: false,
            formatter: Box::new(theme::fmt_confirm2),
        }
    }

    /// Set whether the prompt should be active at start.
    pub fn initial(&mut self, active: bool) -> &mut Self {
        self.active = active;
        self
    }

    /// Set custom closure to format the prompt.
    ///
    /// See: [`Customization`](index.html#customization).
    pub fn format<F>(&mut self, formatter: F) -> &mut Self
    where
        F: Fn(&Confirm, DrawTime, &mut ColoredStrings) + Send + Sync + 'a,
    {
        self.formatter = Box::new(formatter);
        self
    }
}

impl Confirm<'_> {
    pub(crate) fn update_and_submit(&mut self, active: bool) -> bool {
        self.active = active;
        true
    }
}

impl Valuable for Confirm<'_> {
    type Output = bool;
    fn value(&self) -> Result<bool, Error> {
        Ok(self.active)
    }
}

// impl Write for Terminal {}

// impl Write for Bevy {}

// trait Formatter {
//     fn message(&mut self, fmt: &mut Formatter) -> Result<(), Error>;
// }
// struct Run(F) where F : FnOnce(;

impl Printable for Confirm<'_> {
    fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
        use Section::*;
        let draw_time = renderer.draw_time();
        let style = DefaultStyle { ascii: true };

        let options = ["No", "Yes"];
        renderer.print2(|writer| {
            if draw_time == DrawTime::Last {

                queue!(writer,
                       style.begin(Query(true)),
                       Print(self.message.to_string()),
                       style.end(Query(true)),

                       style.begin(Answer(true)),
                       Print(options[self.active as usize]),
                       style.end(Answer(true)),
                )?;
                // style.begin(writer, Section::Answered)?;
                // queue!(writer, Print(self.message.to_string()))?;
                // style.end(writer)?;
                // style.begin(writer, Section::Answer)?;
                // queue!(writer, Print(options[self.active as usize]))?;
                // style.end(writer)?;
                Ok(1)
            } else {
                queue!(writer,
                       style.begin(Query(false)),
                       Print(self.message.to_string()),
                       style.end(Query(false)),
                       style.begin(Option(!self.active)),
                       Print(options[0]),
                       style.end(Option(!self.active)),
                       style.begin(Option(self.active)),
                       Print(options[1]),
                       style.end(Option(self.active)),
                )?;
                Ok(2)
            }
        })
        // let mut out = ColoredStrings::default();
        // (self.formatter)(self, renderer.draw_time(), &mut out);
        // // Maybe we can unify these?
        // // renderer.hide_cursor()?;
        // renderer.print(out)
    }
}

#[cfg(feature = "terminal")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::key_listener::Typeable;
    use crossterm::event::{KeyCode, KeyEvent};

    #[test]
    fn set_initial_value() {
        let mut prompt = Confirm::new("");

        prompt.initial(false);
        assert!(!prompt.active);
        prompt.initial(true);
        assert!(prompt.active);
    }

    #[test]
    fn set_custom_formatter() {
        let mut prompt: Confirm = Confirm::new("");
        let draw_time = DrawTime::First;
        const EXPECTED_VALUE: &str = "foo";

        prompt.format(|_, _, out| out.push(EXPECTED_VALUE.into()));
        let mut out = ColoredStrings::new();
        (prompt.formatter)(&prompt, draw_time, &mut out);
        assert_eq!(format!("{}", out), EXPECTED_VALUE);
    }

    #[test]
    fn update_and_submit() {
        let events = [('y', true), ('Y', true), ('n', false), ('N', false)];

        for (char, expected) in events {
            let mut prompt = Confirm::new("");
            let simulated_key = KeyEvent::from(KeyCode::Char(char));

            prompt.initial(!expected);
            let submit = prompt.handle_key(&simulated_key);

            assert_eq!(prompt.active, expected);
            assert!(submit);
        }
    }

    #[test]
    fn submit_focused() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = Confirm::new("");
            let simulated_key = KeyEvent::from(event);

            let submit = prompt.handle_key(&simulated_key);
            assert!(!prompt.active);
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

        for (key, initial, expected) in events {
            let mut prompt = Confirm::new("");
            let simulated_key = KeyEvent::from(key);

            prompt.initial(initial);
            let submit = prompt.handle_key(&simulated_key);

            assert_eq!(prompt.active, expected);
            assert!(!submit);
        }
    }
}
