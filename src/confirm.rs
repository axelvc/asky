use std::io;

use crossterm::{
    event::{read, Event, KeyCode, KeyEvent},
    terminal,
};

use crate::{renderer::Renderer, utils};

pub struct Confirm<'a, W: io::Write> {
    submit: bool,
    value: bool,
    renderer: Renderer<'a, W>,
}

impl<W: io::Write> Confirm<'_, W> {
    pub fn initial(&mut self, value: bool) -> &mut Self {
        self.value = value;
        self
    }

    pub fn prompt(&mut self) -> io::Result<bool> {
        terminal::enable_raw_mode()?;
        self.renderer.draw_toggle(self.value)?;

        while !self.submit {
            if let Event::Key(key) = read()? {
                if utils::is_abort(key) {
                    utils::abort()?;
                }

                self.handle_key(key);
                self.renderer.draw_toggle(self.value)?;
            }
        }

        terminal::disable_raw_mode()?;

        Ok(self.value)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        let mut submit = false;

        match key.code {
            // yes
            KeyCode::Char('y' | 'Y') => {
                self.value = true;
                submit = true
            }
            // no
            KeyCode::Char('n' | 'N') => {
                self.value = false;
                submit = true
            }
            // focused/initial
            KeyCode::Enter | KeyCode::Backspace => submit = true,
            // focus yes
            KeyCode::Left | KeyCode::Char('h' | 'H') => self.value = true,
            // focus no
            KeyCode::Right | KeyCode::Char('l' | 'L') => self.value = false,

            _ => (),
        }

        self.submit = submit
    }
}

impl Confirm<'_, io::Stdout> {
    pub fn new(message: &str) -> Confirm<io::Stdout> {
        Confirm {
            value: false,
            submit: false,
            renderer: Renderer::new(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_initial_value() {
        let mut prompt = Confirm::new("");

        assert!(!prompt.value);
        prompt.initial(true);
        assert!(prompt.value);
    }

    #[test]
    fn sumit_focused() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = Confirm::new("");
            let simulated_key = KeyEvent::from(event);

            prompt.initial(true);

            assert_eq!(prompt.submit, false);

            prompt.handle_key(simulated_key);
            assert_eq!(prompt.value, true);
            assert_eq!(prompt.submit, true);
        }
    }

    #[test]
    fn update_and_submit() {
        let events = [('y', true), ('Y', true), ('n', false), ('N', false)];

        for (char, expected) in events {
            let mut prompt = Confirm::new("");
            let simulated_key = KeyEvent::from(KeyCode::Char(char));

            prompt.initial(!expected);

            assert_eq!(prompt.submit, false);

            prompt.handle_key(simulated_key);
            assert_eq!(prompt.value, expected);
            assert_eq!(prompt.submit, true);
        }
    }

    #[test]
    fn update_focused() {
        let events = [
            (KeyCode::Left, true),
            (KeyCode::Char('h'), true),
            (KeyCode::Char('H'), true),
            (KeyCode::Right, false),
            (KeyCode::Char('l'), false),
            (KeyCode::Char('L'), false),
        ];

        for (key, expected) in events {
            let mut prompt = Confirm::new("");
            let simulated_key = KeyEvent::from(key);

            prompt.initial(!expected);
            prompt.handle_key(simulated_key);

            assert_eq!(prompt.value, expected);
            assert_eq!(prompt.submit, false);
        }
    }
}
