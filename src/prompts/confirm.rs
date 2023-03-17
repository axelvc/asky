use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, KeyHandler},
    renderer::Renderer,
    theme::{DefaultTheme, Theme},
};

pub struct Confirm<'a> {
    pub(crate) message: &'a str,
    pub(crate) active: bool,
    pub(crate) submit: bool,
    pub(crate) theme: &'a dyn Theme,
}

impl<'a> Confirm<'a> {
    pub fn new(message: &str) -> Confirm {
        Confirm {
            message,
            active: false,
            submit: false,
            theme: &DefaultTheme,
        }
    }

    pub fn initial(&mut self, active: bool) -> &mut Self {
        self.active = active;
        self
    }

    pub fn theme(&mut self, theme: &'a dyn Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn prompt(&mut self) -> io::Result<bool> {
        key_listener::listen(self)?;
        Ok(self.active)
    }

    fn update_and_submit(&mut self, active: bool) {
        self.active = active;
        self.submit = true;
    }
}

impl KeyHandler for Confirm<'_> {
    fn submit(&self) -> bool {
        self.submit
    }

    fn draw<W: io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()> {
        renderer.confirm(self)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            // submit focused/initial
            KeyCode::Enter | KeyCode::Backspace => self.submit = true,
            // focus left
            KeyCode::Left | KeyCode::Char('h' | 'H') => self.active = false,
            // focus right
            KeyCode::Right | KeyCode::Char('l' | 'L') => self.active = true,
            // submit yes
            KeyCode::Char('y' | 'Y') => self.update_and_submit(true),
            // submit no
            KeyCode::Char('n' | 'N') => self.update_and_submit(false),
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_initial_value() {
        let mut prompt = Confirm::new("");

        prompt.initial(false);
        assert!(!prompt.active);
        prompt.initial(true);
        assert!(prompt.active);
    }

    #[test]
    fn update_and_submit() {
        let events = [('y', true), ('Y', true), ('n', false), ('N', false)];

        for (char, expected) in events {
            let mut prompt = Confirm::new("");
            let simulated_key = KeyEvent::from(KeyCode::Char(char));

            prompt.initial(!expected);
            prompt.handle_key(simulated_key);

            assert_eq!(prompt.active, expected);
            assert_eq!(prompt.submit, true);
        }
    }

    #[test]
    fn sumit_focused() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = Confirm::new("");
            let simulated_key = KeyEvent::from(event);

            prompt.handle_key(simulated_key);
            assert!(!prompt.active);
            assert_eq!(prompt.submit, true);
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
            prompt.handle_key(simulated_key);

            assert_eq!(prompt.active, expected);
            assert_eq!(prompt.submit, false);
        }
    }
}
