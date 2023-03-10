use std::io;

use crate::utils::{
    key_listener::{self, KeyHandler},
    renderer::{DrawTime, Renderer}, theme::Theme,
};

use super::text::Text;

pub struct Password<'a> {
    hidden: bool,
    handler: Text<'a>,
}

impl<'a> Password<'a> {
    pub fn new(message: &str) -> Password {
        Password {
            hidden: false,
            handler: Text::new(message),
        }
    }

    pub fn hidden(&mut self, hidden: bool) -> &mut Self {
        self.hidden = hidden;
        self
    }

    pub fn default(&mut self, value: &str) -> &mut Self {
        self.handler.default(value);
        self
    }

    pub fn initial(&mut self, value: &str) -> &mut Self {
        self.handler.initial(value);
        self
    }

    pub fn validate<F>(&mut self, validator: F) -> &mut Self
    where
        F: Fn(&str) -> Result<(), &str> + 'static,
    {
        self.handler.validate(validator);
        self
    }

    pub fn theme(&mut self, theme: &'a dyn Theme) -> &mut Self {
        self.handler.theme(theme);
        self
    }

    pub fn prompt(&mut self) -> io::Result<String> {
        key_listener::listen(self)?;

        Ok(self.handler.get_value().to_owned())
    }
}

impl KeyHandler for Password<'_> {
    fn submit(&self) -> bool {
        self.handler.submit()
    }

    fn draw<W: io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()> {
        if self.hidden && renderer.draw_time == DrawTime::Update {
            return Ok(());
        }

        renderer.password(&self.handler, self.hidden)
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) {
        self.handler.handle_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn se_hidden_value() {
        let mut prompt = Password::new("");

        prompt.hidden(false);
        assert!(!prompt.hidden);
        prompt.hidden(true);
        assert!(prompt.hidden)
    }
}
