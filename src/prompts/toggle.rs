use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, KeyHandler},
    renderer::Renderer,
    theme::{DefaultTheme, Theme},
};

pub struct Toggle<'a> {
    pub(crate) message: &'a str,
    pub(crate) options: (&'a str, &'a str),
    pub(crate) submit: bool,
    pub(crate) active: bool,
    pub(crate) theme: &'a dyn Theme,
}

impl<'a> Toggle<'a> {
    pub fn new(message: &'a str, options: (&'a str, &'a str)) -> Self {
        Toggle {
            message,
            options,
            active: false,
            submit: false,
            theme: &DefaultTheme,
        }
    }

    pub fn initial(&mut self, value: bool) -> &mut Self {
        self.active = value;
        self
    }

    pub fn theme(&mut self, theme: &'a dyn Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn prompt(&mut self) -> io::Result<String> {
        key_listener::listen(self)?;

        Ok(String::from(self.get_value()))
    }

    fn get_value(&self) -> &str {
        if self.active {
            self.options.1
        } else {
            self.options.0
        }
    }
}

impl KeyHandler for Toggle<'_> {
    fn submit(&self) -> bool {
        self.submit
    }

    fn draw<W: io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()> {
        renderer.toggle(self)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        let mut submit = false;

        match key.code {
            // submit focused/initial
            KeyCode::Enter | KeyCode::Backspace => submit = true,
            // focus left
            KeyCode::Left | KeyCode::Char('h' | 'H') => self.active = false,
            // focus right
            KeyCode::Right | KeyCode::Char('l' | 'L') => self.active = true,
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
        let mut prompt = Toggle::new("", ("foo", "bar"));

        prompt.initial(false);
        assert_eq!(prompt.get_value(), "foo");
        prompt.initial(true);
        assert_eq!(prompt.get_value(), "bar");
    }

    #[test]
    fn sumit_focused() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = Toggle::new("", ("foo", "bar"));
            let simulated_key = KeyEvent::from(event);

            prompt.handle_key(simulated_key);
            assert_eq!(prompt.get_value(), "foo");
            assert_eq!(prompt.submit, true);
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
            let mut prompt = Toggle::new("", ("foo", "bar"));
            let simulated_key = KeyEvent::from(key);

            prompt.initial(initial);
            prompt.handle_key(simulated_key);

            assert_eq!(prompt.get_value(), expected);
            assert_eq!(prompt.submit, false);
        }
    }
}
