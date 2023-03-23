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
    pub(crate) min: Option<usize>,
    pub(crate) max: Option<usize>,
    pub(crate) theme: &'a dyn Theme,
    selected_count: usize,
}

impl<'a, T> MultiSelect<'a, T> {
    pub fn new(message: &'a str, options: Vec<SelectOption<'a, T>>) -> MultiSelect<'a, T> {
        MultiSelect {
            message,
            options,
            focused: 0,
            is_loop: false,
            submit: false,
            min: None,
            max: None,
            selected_count: 0,
            theme: &DefaultTheme,
        }
    }

    pub fn selected(&mut self, selected: &[usize]) -> &mut Self {
        for i in selected {
            if let Some(option) = self.options.get_mut(*i) {
                option.active = true;
                self.selected_count += 1;
            }
        }

        self
    }

    pub fn min(&mut self, min: usize) -> &mut Self {
        self.min = Some(min);
        self
    }

    pub fn max(&mut self, max: usize) -> &mut Self {
        self.max = Some(max);
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
        let focused = &mut self.options[self.focused];

        if focused.disabled {
            return;
        }

        let under_limit = match self.max {
            None => true,
            Some(max) => self.selected_count < max,
        };

        if focused.active {
            focused.active = false;
            self.selected_count -= 1;
        } else if under_limit {
            focused.active = true;
            self.selected_count += 1;
        }
    }

    fn validate_to_submit(&mut self) -> bool {
        if let Some(min) = self.min {
            if self.selected_count < min {
                return false;
            }
        }

        true
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
            KeyCode::Enter | KeyCode::Backspace => submit = self.validate_to_submit(),
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
    fn set_selected_values() {
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
    fn set_min() {
        let mut prompt = MultiSelect::<&str>::new("", vec![]);

        prompt.min(2);

        assert_eq!(prompt.min, Some(2));
    }

    #[test]
    fn set_max() {
        let mut prompt = MultiSelect::<&str>::new("", vec![]);

        prompt.max(2);

        assert_eq!(prompt.max, Some(2));
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
    fn not_submit_without_min() {
        let mut prompt = MultiSelect::new(
            "",
            vec![
                SelectOption::new("a", "a"),
                SelectOption::new("b", "b"),
                SelectOption::new("c", "c"),
            ],
        );

        prompt.min(1);
        prompt.handle_key(KeyEvent::from(KeyCode::Enter));

        assert!(!prompt.submit);

        prompt.handle_key(KeyEvent::from(KeyCode::Char(' ')));
        prompt.handle_key(KeyEvent::from(KeyCode::Enter));

        assert!(prompt.submit);
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

        prompt.max(1);

        assert!(!prompt.options[1].active);
        assert!(!prompt.options[2].active);

        prompt.focused = 1;
        prompt.handle_key(KeyEvent::from(KeyCode::Char(' ')));

        // must not update over limit
        prompt.focused = 2;
        prompt.handle_key(KeyEvent::from(KeyCode::Char(' ')));

        assert!(prompt.options[1].active);
        assert!(!prompt.options[2].active);
    }
}
