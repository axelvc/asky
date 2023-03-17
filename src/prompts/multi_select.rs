use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, KeyHandler},
    renderer::Renderer,
    theme::{DefaultTheme, Theme},
};

use super::select::SelectOption;

enum Direction {
    Up,
    Down,
}

pub struct MultiSelect<'a, T> {
    pub(crate) message: &'a str,
    pub(crate) options: Vec<SelectOption<'a, T>>,
    pub(crate) focused: usize,
    pub(crate) is_loop: bool,
    pub(crate) submit: bool,
    pub(crate) theme: &'a dyn Theme,
}

impl<'a, T> MultiSelect<'a, T> {
    pub fn new(message: &'a str, options: Vec<SelectOption<'a, T>>) -> MultiSelect<'a, T> {
        MultiSelect {
            message,
            options,
            focused: 0,
            is_loop: false,
            submit: false,
            theme: &DefaultTheme,
        }
    }

    pub fn selected(&mut self, selected: &[usize]) -> &mut Self {
        for i in selected {
            if let Some(option) = self.options.get_mut(*i) {
                option.active = true
            }
        }

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

    pub fn prompt(&mut self) -> io::Result<Vec<T>> {
        key_listener::listen(self)?;

        let (selected, _): (Vec<_>, Vec<_>) = self.options.drain(..).partition(|x| x.active);

        Ok(selected.into_iter().map(|x| x.value).collect())
    }

    fn move_cursor(&mut self, direction: Direction) {
        let max = self.options.len() - 1;

        self.focused = match direction {
            Direction::Up => {
                if self.is_loop && self.focused == 0 {
                    max
                } else {
                    self.focused.saturating_sub(1)
                }
            }
            Direction::Down => {
                if self.is_loop && self.focused == max {
                    0
                } else {
                    (self.focused + 1).min(self.options.len() - 1)
                }
            }
        }
    }

    fn toggle_focused(&mut self) {
        self.options[self.focused].toggle_selected()
    }
}

impl<'a, T> KeyHandler for MultiSelect<'a, T> {
    fn submit(&self) -> bool {
        self.submit
    }

    fn draw<W: io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()> {
        renderer.multi_select(self)
    }

    fn handle_key(&mut self, key: KeyEvent) {
        let mut submit = false;

        match key.code {
            // submit
            KeyCode::Enter | KeyCode::Backspace => submit = true,
            // select/unselect
            KeyCode::Char(' ') => self.toggle_focused(),
            // update value
            KeyCode::Up | KeyCode::Char('k' | 'K') => self.move_cursor(Direction::Up),
            KeyCode::Down | KeyCode::Char('j' | 'J') => self.move_cursor(Direction::Down),
            _ => (),
        }

        self.submit = submit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_selected_value() {
        let mut prompt = MultiSelect::new(
            "",
            vec![
                SelectOption::new("a", "a"),
                SelectOption::new("b", "b"),
                SelectOption::new("c", "c"),
            ],
        );

        prompt.selected(&[0, 2]);
        assert!(prompt.options[0].active);
        assert!(prompt.options[2].active);
    }

    #[test]
    fn set_in_loop() {
        let mut prompt = MultiSelect::new(
            "",
            vec![
                SelectOption::new("a", "a"),
                SelectOption::new("b", "b"),
                SelectOption::new("c", "c"),
            ],
        );

        assert!(!prompt.is_loop);
        prompt.in_loop(true);
        assert!(prompt.is_loop);
    }

    #[test]
    fn submit_keys() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = MultiSelect::new(
                "",
                vec![
                    SelectOption::new("a", "a"),
                    SelectOption::new("b", "b"),
                    SelectOption::new("c", "c"),
                ],
            );
            let simulated_key = KeyEvent::from(event);

            prompt.handle_key(simulated_key);
            assert_eq!(prompt.submit, true);
        }
    }

    #[test]
    fn move_cursor() {
        let mut prompt = MultiSelect::new(
            "",
            vec![
                SelectOption::new("a", "a"),
                SelectOption::new("b", "b"),
                SelectOption::new("c", "c"),
            ],
        );
        let prev_keys = [KeyCode::Up, KeyCode::Char('k'), KeyCode::Char('K')];
        let next_keys = [KeyCode::Down, KeyCode::Char('j'), KeyCode::Char('j')];

        // move next
        prompt.in_loop(false);

        for key in next_keys {
            prompt.focused = 0;
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.focused, 1);
        }

        // move next in loop
        prompt.in_loop(true);

        for key in next_keys {
            prompt.focused = 2;
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.focused, 0);
        }

        // move next
        prompt.in_loop(false);

        for key in prev_keys {
            prompt.focused = 2;
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.focused, 1);
        }

        // move next in loop
        prompt.in_loop(true);

        for key in prev_keys {
            prompt.focused = 0;
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.focused, 2);
        }
    }

    #[test]
    fn update_focused_selected() {
        let mut prompt = MultiSelect::new(
            "",
            vec![
                SelectOption::new("a", "a"),
                SelectOption::new("b", "b"),
                SelectOption::new("c", "c"),
            ],
        );

        assert!(!prompt.options[1].active);

        prompt.focused = 1;
        prompt.handle_key(KeyEvent::from(KeyCode::Char(' ')));

        assert!(prompt.options[1].active);
    }
}
