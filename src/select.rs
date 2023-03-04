use std::{fmt::Display, io};

use crossterm::{
    event::{read, Event, KeyCode, KeyEvent},
    terminal,
};

use crate::{renderer::Renderer, utils};

enum Direction {
    Up,
    Down,
}

pub struct Select<'a, T, W>
where
    T: Display + Copy,
    W: io::Write,
{
    options: &'a [T],
    selected: usize,
    renderer: Renderer<'a, W>,
    is_loop: bool,
    submit: bool,
}

impl<'a, T, W> Select<'a, T, W>
where
    T: Display + Copy,
    W: io::Write,
{
    pub fn initial(&mut self, selected: usize) -> &mut Self {
        self.selected = selected;
        self
    }

    pub fn in_loop(&mut self, is_loop: bool) -> &mut Self {
        self.is_loop = is_loop;
        self
    }

    pub fn prompt(&mut self) -> io::Result<T> {
        terminal::enable_raw_mode()?;
        self.renderer.draw_select(&self.options, self.selected)?;

        while !self.submit {
            if let Event::Key(key) = read()? {
                if utils::is_abort(key) {
                    utils::abort()?;
                }

                self.handle_key(key);
                self.renderer.draw_select(&self.options, self.selected)?;
            }
        }

        terminal::disable_raw_mode()?;

        Ok(self.options[self.selected])
    }

    fn handle_key(&mut self, key: KeyEvent) {
        let mut submit = false;

        match key.code {
            // submit
            KeyCode::Enter | KeyCode::Backspace => submit = true,
            KeyCode::Up | KeyCode::Char('k' | 'K') => self.update_value(Direction::Up),
            KeyCode::Down | KeyCode::Char('j' | 'J') => self.update_value(Direction::Down),
            _ => (),
        }

        self.submit = submit
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
}

impl<'a, T> Select<'a, T, io::Stdout>
where
    T: Display + Copy,
{
    pub fn new(message: &'a str, options: &'a [T]) -> Select<'a, T, io::Stdout> {
        Select {
            options,
            selected: 0,
            renderer: Renderer::new(message),
            submit: false,
            is_loop: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_initial_value() {
        let mut prompt = Select::new("", &["foo", "bar"]);

        assert_eq!(prompt.selected, 0);
        prompt.initial(1);
        assert_eq!(prompt.selected, 1);
    }

    #[test]
    fn set_in_loop() {
        let mut prompt = Select::new("", &["foo", "bar"]);

        assert!(!prompt.is_loop);
        prompt.in_loop(true);
        assert!(prompt.is_loop);
    }

    #[test]
    fn submit_selected_value() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = Select::new("", &["foo", "bar"]);
            let simulated_key = KeyEvent::from(event);

            prompt.initial(1);

            prompt.handle_key(simulated_key);
            assert_eq!(prompt.selected, 1);
            assert_eq!(prompt.submit, true);
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
                let mut prompt = Select::new("", &["foo", "bar"]);
                let simulated_key = KeyEvent::from(key);

                prompt.initial(initial);
                prompt.in_loop(in_loop);
                prompt.handle_key(simulated_key);
                assert_eq!(prompt.selected, expected);
            }
        }

        for key in down_keys {
            for (in_loop, initial, expected) in down_cases {
                let mut prompt = Select::new("", &["foo", "bar"]);
                let simulated_key = KeyEvent::from(key);

                prompt.initial(initial);
                prompt.in_loop(in_loop);
                prompt.handle_key(simulated_key);
                assert_eq!(prompt.selected, expected);
            }
        }
    }
}
