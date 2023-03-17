use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, KeyHandler},
    renderer::Renderer,
};

use super::text::Text;

pub struct Number<'a> {
    pub(crate) handler: Text<'a>,
}

impl Number<'_> {
    pub fn new(message: &str) -> Number {
        Number {
            handler: Text::new(message),
        }
    }

    pub fn default<T: ToString>(&mut self, value: T) -> &mut Self {
        self.handler.default(&value.to_string());
        self
    }

    pub fn initial<T: ToString>(&mut self, value: T) -> &mut Self {
        self.handler.initial(&value.to_string());
        self
    }

    pub fn validate<F>(&mut self, validator: F) -> &mut Self
    where
        F: Fn(&str) -> Result<(), &str> + 'static,
    {
        self.handler.validate(validator);
        self
    }

    pub fn prompt(&mut self) -> io::Result<String> {
        key_listener::listen(self)?;

        Ok(self.handler.value.to_owned())
    }

    fn update_value(&mut self, ch: char) {
        let is_sign = ch == '-' || ch == '+';

        if is_sign && self.handler.cursor_col != 0 {
            return;
        }

        if !ch.is_digit(10) && !is_sign {
            return;
        }

        self.handler.update_value(ch);
    }
}

impl KeyHandler for Number<'_> {
    fn submit(&self) -> bool {
        self.handler.submit()
    }

    fn draw<W: std::io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()> {
        renderer.number(self)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => self.update_value(c),
            _ => self.handler.handle_key(key),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allow_sign_at_the_start() {
        let signs = ['-', '+'];

        for c in signs {
            let mut prompt = Number::new("");

            // must accept only one sign, simulate double press
            prompt.handle_key(KeyEvent::from(KeyCode::Char(c)));
            prompt.handle_key(KeyEvent::from(KeyCode::Char(c)));

            assert_eq!(prompt.handler.value, c.to_string());
        }
    }

    #[test]
    fn only_accept_digits() {
        let mut prompt = Number::new("");

        // try to type a character
        ('a'..='z').for_each(|c| prompt.handle_key(KeyEvent::from(KeyCode::Char(c))));

        // try to type digits
        ('0'..='9').for_each(|c| prompt.handle_key(KeyEvent::from(KeyCode::Char(c))));

        assert_eq!(prompt.handler.value, "0123456789");
    }
}
