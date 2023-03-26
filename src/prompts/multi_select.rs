use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, KeyHandler},
    renderer::Renderer,
    theme::{DefaultTheme, Theme},
};

use super::select::{Direction, SelectInput, SelectOption};

pub struct MultiSelect<'a, T> {
    pub(crate) message: &'a str,
    pub(crate) input: SelectInput<'a, T>,
    pub(crate) theme: &'a dyn Theme,
    pub(crate) min: Option<usize>,
    pub(crate) max: Option<usize>,
    selected_count: usize,
}

impl<'a, T> MultiSelect<'a, T> {
    pub fn new(message: &'a str, options: Vec<SelectOption<'a, T>>) -> Self {
        MultiSelect {
            message,
            input: SelectInput::new(options),
            theme: &DefaultTheme,
            min: None,
            max: None,
            selected_count: 0,
        }
    }

    pub fn selected(&mut self, selected: &[usize]) -> &mut Self {
        for i in selected {
            if let Some(option) = self.input.options.get_mut(*i) {
                option.active = true;
                self.selected_count += 1;
            }
        }

        self
    }

    pub fn in_loop(&mut self, is_loop: bool) -> &mut Self {
        self.input.set_loop_mode(is_loop);
        self
    }

    pub fn items_per_page(&mut self, items_per_page: usize) -> &mut Self {
        self.input.set_items_per_page(items_per_page);
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

    pub fn theme(&mut self, theme: &'a dyn Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn prompt(&mut self) -> io::Result<Vec<T>> {
        key_listener::listen(self)?;

        let (selected, _): (Vec<_>, Vec<_>) = self.input.options.drain(..).partition(|x| x.active);
        let selected = selected.into_iter().map(|x| x.value).collect();

        Ok(selected)
    }
}

impl<T> MultiSelect<'_, T> {
    fn toggle_focused(&mut self) {
        let selected = self.input.focused;
        let focused = &self.input.options[selected];

        if focused.disabled {
            return;
        }

        let under_limit = match self.max {
            None => true,
            Some(max) => self.selected_count < max,
        };

        let focused = &mut self.input.options[selected];

        if focused.active {
            focused.active = false;
            self.selected_count -= 1;
        } else if under_limit {
            focused.active = true;
            self.selected_count += 1;
        }
    }

    /// Only submit if the minimun are selected
    fn validate_to_submit(&self) -> bool {
        match self.min {
            None => true,
            Some(min) => self.selected_count >= min,
        }
    }
}

impl<T> KeyHandler for MultiSelect<'_, T> {
    fn draw<W: io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()> {
        renderer.multi_select(self)
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        let mut submit = false;

        match key.code {
            // submit
            KeyCode::Enter | KeyCode::Backspace => submit = self.validate_to_submit(),
            // select/unselect
            KeyCode::Char(' ') => self.toggle_focused(),
            // update focus
            KeyCode::Up | KeyCode::Char('k' | 'K') => self.input.move_cursor(Direction::Up),
            KeyCode::Down | KeyCode::Char('j' | 'J') => self.input.move_cursor(Direction::Down),
            KeyCode::Left | KeyCode::Char('h' | 'H') => self.input.move_cursor(Direction::Left),
            KeyCode::Right | KeyCode::Char('l' | 'L') => self.input.move_cursor(Direction::Right),
            _ => (),
        }

        submit
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
        assert!(prompt.input.options[0].active);
        assert!(prompt.input.options[2].active);
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

        prompt.in_loop(false);
        assert!(!prompt.input.loop_mode);
        prompt.in_loop(true);
        assert!(prompt.input.loop_mode);
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

            let submit = prompt.handle_key(simulated_key);
            assert!(submit);
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
        let mut submit = prompt.handle_key(KeyEvent::from(KeyCode::Enter));

        assert!(!submit);

        prompt.handle_key(KeyEvent::from(KeyCode::Char(' ')));
        submit = prompt.handle_key(KeyEvent::from(KeyCode::Enter));

        assert!(submit);
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
            prompt.input.focused = 0;
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.input.focused, 1);
        }

        // move next in loop
        prompt.in_loop(true);

        for key in next_keys {
            prompt.input.focused = 2;
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.input.focused, 0);
        }

        // move next
        prompt.in_loop(false);

        for key in prev_keys {
            prompt.input.focused = 2;
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.input.focused, 1);
        }

        // move next in loop
        prompt.in_loop(true);

        for key in prev_keys {
            prompt.input.focused = 0;
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.input.focused, 2);
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

        assert!(!prompt.input.options[1].active);
        assert!(!prompt.input.options[2].active);

        prompt.input.focused = 1;
        prompt.handle_key(KeyEvent::from(KeyCode::Char(' ')));

        // must not update over limit
        prompt.input.focused = 2;
        prompt.handle_key(KeyEvent::from(KeyCode::Char(' ')));

        assert!(prompt.input.options[1].active);
        assert!(!prompt.input.options[2].active);
    }
}
