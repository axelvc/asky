use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    key_listener::{self, KeyHandler},
    renderer::Renderer,
};

pub struct Confirm<'a> {
    message: &'a str,
    submit: bool,
    value: bool,
}

impl Confirm<'_> {
    pub fn new(message: &str) -> Confirm {
        Confirm {
            message,
            value: false,
            submit: false,
        }
    }

    pub fn initial(&mut self, value: bool) -> &mut Self {
        self.value = value;
        self
    }

    pub fn prompt(&mut self) -> io::Result<bool> {
        key_listener::listen(self.message, self)?;
        Ok(self.value)
    }
}

impl KeyHandler for Confirm<'_> {
    fn submit(&self) -> bool {
        self.submit
    }

    fn draw<W: io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()> {
        renderer.draw_toggle(self.value)
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
