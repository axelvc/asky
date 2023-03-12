use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, KeyHandler},
    renderer::Renderer,
};

use super::toggle::Toggle;

pub struct Confirm<'a> {
    handler: Toggle<'a>,
}

impl Confirm<'_> {
    pub fn new(message: &str) -> Confirm {
        Confirm {
            handler: Toggle::new(message, ("No", "Yes")),
        }
    }

    pub fn initial(&mut self, value: bool) -> &mut Self {
        self.handler.initial(value);
        self
    }

    pub fn prompt(&mut self) -> io::Result<bool> {
        key_listener::listen(self)?;
        Ok(self.handler.active)
    }

    fn update_and_submit(&mut self, active: bool) {
        self.handler.active = active;
        self.handler.submit = true;
    }
}

impl KeyHandler for Confirm<'_> {
    fn submit(&self) -> bool {
        self.handler.submit()
    }

    fn draw<W: io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()> {
        self.handler.draw(renderer)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            // submit yes
            KeyCode::Char('y' | 'Y') => self.update_and_submit(true),
            // submit no
            KeyCode::Char('n' | 'N') => self.update_and_submit(false),
            _ => self.handler.handle_key(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_and_submit() {
        let events = [('y', true), ('Y', true), ('n', false), ('N', false)];

        for (char, expected) in events {
            let mut prompt = Confirm::new("");
            let simulated_key = KeyEvent::from(KeyCode::Char(char));

            prompt.initial(!expected);
            prompt.handle_key(simulated_key);

            assert_eq!(prompt.handler.active, expected);
            assert_eq!(prompt.handler.submit, true);
        }
    }
}
