use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, KeyHandler},
    renderer::Renderer,
    theme::{DefaultTheme, Theme},
};

enum Direction {
    Up,
    Down,
}

pub struct SelectOption<'a, T: Copy> {
    pub(crate) value: T,
    pub(crate) title: &'a str,
    pub(crate) description: Option<&'a str>,
    pub(crate) disabled: bool,
    pub(crate) active: bool,
}

impl<'a, T: Copy> SelectOption<'a, T> {
    pub fn new(value: T, title: &'a str) -> Self {
        Self {
            value,
            title,
            description: None,
            disabled: false,
            active: false,
        }
    }

    /// Description text to show in the prompt when focus the option
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// If it is disabled, the option cannot be selected by the user
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub(crate) fn toggle_selected(&mut self) {
        if !self.disabled {
            self.active = !self.active
        }
    }
}

pub struct Select<'a, T: Copy> {
    pub(crate) message: &'a str,
    pub(crate) options: Vec<SelectOption<'a, T>>,
    pub(crate) selected: usize,
    pub(crate) is_loop: bool,
    pub(crate) submit: bool,
    pub(crate) theme: &'a dyn Theme,
}

impl<'a, T: Copy> Select<'a, T> {
    pub fn new(message: &'a str, options: Vec<SelectOption<'a, T>>) -> Select<'a, T> {
        Select {
            message,
            options,
            selected: 0,
            submit: false,
            is_loop: false,
            theme: &DefaultTheme,
        }
    }

    pub fn initial(&mut self, selected: usize) -> &mut Self {
        self.selected = selected;
        self
    }

    pub fn in_loop(&mut self, is_loop: bool) -> &mut Self {
        self.is_loop = is_loop;
        self
    }

    pub fn theme(&mut self, theme: &'a dyn Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn prompt(&mut self) -> io::Result<T> {
        key_listener::listen(self)?;
        Ok(self.options[self.selected].value)
    }

    fn update_value(&mut self, direction: Direction) {
        let max = self.options.len() - 1;

        self.selected = match direction {
            Direction::Up => {
                if self.is_loop && self.selected == 0 {
                    max
                } else {
                    self.selected.saturating_sub(1)
                }
            }
            Direction::Down => {
                if self.is_loop && self.selected == max {
                    0
                } else {
                    (self.selected + 1).min(self.options.len() - 1)
                }
            }
        }
    }

    /// Only submit if the option isn't disabled
    fn validate_to_submit(&self) -> bool {
        !self.options[self.selected].disabled
    }
}

impl<'a, T: Copy> KeyHandler for Select<'a, T> {
    fn submit(&self) -> bool {
        self.submit
    }

    fn draw<W: io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()> {
        renderer.select(self)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        let mut submit = false;

        match key.code {
            // submit
            KeyCode::Enter | KeyCode::Backspace => submit = self.validate_to_submit(),
            // update value
            KeyCode::Up | KeyCode::Char('k' | 'K') => self.update_value(Direction::Up),
            KeyCode::Down | KeyCode::Char('j' | 'J') => self.update_value(Direction::Down),
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
        let mut prompt = Select::new(
            "",
            vec![
                SelectOption::new("foo", "foo"),
                SelectOption::new("bar", "bar"),
            ],
        );

        assert_eq!(prompt.selected, 0);
        prompt.initial(1);
        assert_eq!(prompt.selected, 1);
    }

    #[test]
    fn set_in_loop() {
        let mut prompt = Select::new(
            "",
            vec![
                SelectOption::new("foo", "foo"),
                SelectOption::new("bar", "bar"),
            ],
        );

        assert!(!prompt.is_loop);
        prompt.in_loop(true);
        assert!(prompt.is_loop);
    }

    #[test]
    fn submit_selected_value() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = Select::new(
                "",
                vec![
                    SelectOption::new("foo", "foo"),
                    SelectOption::new("bar", "bar"),
                ],
            );
            let simulated_key = KeyEvent::from(event);

            prompt.initial(1);

            prompt.handle_key(simulated_key);
            assert_eq!(prompt.selected, 1);
            assert_eq!(prompt.submit, true);
        }
    }

    #[test]
    fn not_sumit_disabled() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = Select::new("", vec![SelectOption::new("foo", "foo").disabled(true)]);

            prompt.handle_key(KeyEvent::from(event));
            assert_eq!(prompt.submit, false);
        }
    }

    #[test]
    fn update_selected() {
        let up_keys = [KeyCode::Up, KeyCode::Char('k'), KeyCode::Char('K')];
        let down_keys = [KeyCode::Down, KeyCode::Char('j'), KeyCode::Char('j')];

        let up_cases = [
            //in_loop, initial, expected
            (false, 0, 0),
            (false, 1, 0),
            (true, 0, 1),
        ];
        let down_cases = [
            //in_loop, initial, expected
            (false, 1, 1),
            (false, 0, 1),
            (true, 1, 0),
        ];

        for key in up_keys {
            for (in_loop, initial, expected) in up_cases {
                let mut prompt = Select::new(
                    "",
                    vec![
                        SelectOption::new("foo", "foo"),
                        SelectOption::new("bar", "bar"),
                    ],
                );
                let simulated_key = KeyEvent::from(key);

                prompt.initial(initial);
                prompt.in_loop(in_loop);
                prompt.handle_key(simulated_key);
                assert_eq!(prompt.selected, expected);
            }
        }

        for key in down_keys {
            for (in_loop, initial, expected) in down_cases {
                let mut prompt = Select::new(
                    "",
                    vec![
                        SelectOption::new("foo", "foo"),
                        SelectOption::new("bar", "bar"),
                    ],
                );
                let simulated_key = KeyEvent::from(key);

                prompt.initial(initial);
                prompt.in_loop(in_loop);
                prompt.handle_key(simulated_key);
                assert_eq!(prompt.selected, expected);
            }
        }
    }
}
