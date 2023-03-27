use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, Typeable},
    renderer::{Printable, Renderer},
    theme,
};

use super::select::{Direction, SelectCursor, SelectOption};

type Formatter<'a, T> = dyn Fn(&MultiSelect<T>, &Renderer) -> String + 'a;

pub struct MultiSelect<'a, T> {
    pub message: &'a str,
    pub options: Vec<SelectOption<'a, T>>,
    pub min: Option<usize>,
    pub max: Option<usize>,
    pub selected_count: usize,
    pub cursor: SelectCursor,
    formatter: Box<Formatter<'a, T>>,
}

impl<'a, T: 'a> MultiSelect<'a, T> {
    pub fn new(message: &'a str, options: Vec<SelectOption<'a, T>>) -> Self {
        let options_len = options.len();

        MultiSelect {
            message,
            options,
            min: None,
            max: None,
            selected_count: 0,
            cursor: SelectCursor::new(options_len),
            formatter: Box::new(theme::fmt_multi_select),
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

    pub fn in_loop(&mut self, is_loop: bool) -> &mut Self {
        self.cursor.set_loop_mode(is_loop);
        self
    }

    pub fn items_per_page(&mut self, items_per_page: usize) -> &mut Self {
        self.cursor.set_items_per_page(items_per_page);
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

    pub fn format<F>(&mut self, formatter: F) -> &mut Self
    where
        F: Fn(&MultiSelect<T>, &Renderer) -> String + 'a,
    {
        self.formatter = Box::new(formatter);
        self
    }

    pub fn prompt(&mut self) -> io::Result<Vec<T>> {
        key_listener::listen(self)?;

        let (selected, _): (Vec<_>, Vec<_>) = self.options.drain(..).partition(|x| x.active);
        let selected = selected.into_iter().map(|x| x.value).collect();

        Ok(selected)
    }
}

impl<T> MultiSelect<'_, T> {
    fn toggle_focused(&mut self) {
        let selected = self.cursor.focused;
        let focused = &self.options[selected];

        if focused.disabled {
            return;
        }

        let under_limit = match self.max {
            None => true,
            Some(max) => self.selected_count < max,
        };

        let focused = &mut self.options[selected];

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

impl<T> Typeable for MultiSelect<'_, T> {
    fn handle_key(&mut self, key: KeyEvent) -> bool {
        let mut submit = false;

        match key.code {
            // submit
            KeyCode::Enter | KeyCode::Backspace => submit = self.validate_to_submit(),
            // select/unselect
            KeyCode::Char(' ') => self.toggle_focused(),
            // update focus
            KeyCode::Up | KeyCode::Char('k' | 'K') => self.cursor.move_cursor(Direction::Up),
            KeyCode::Down | KeyCode::Char('j' | 'J') => self.cursor.move_cursor(Direction::Down),
            KeyCode::Left | KeyCode::Char('h' | 'H') => self.cursor.move_cursor(Direction::Left),
            KeyCode::Right | KeyCode::Char('l' | 'L') => self.cursor.move_cursor(Direction::Right),
            _ => (),
        }

        submit
    }
}

impl<T> Printable for MultiSelect<'_, T> {
    fn draw(&self, renderer: &mut crate::utils::renderer::Renderer) -> io::Result<()> {
        let text = (self.formatter)(self, renderer);
        renderer.print(&text)
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

        prompt.in_loop(false);
        assert!(!prompt.cursor.loop_mode);
        prompt.in_loop(true);
        assert!(prompt.cursor.loop_mode);
    }

    #[test]
    fn set_custom_formatter() {
        let mut prompt: MultiSelect<u8> = MultiSelect::new("", vec![]);
        let renderer = Renderer::new();

        const EXPECTED_VALUE: &str = "foo";
        let formatter = |_: &MultiSelect<u8>, _: &Renderer| String::from(EXPECTED_VALUE);

        prompt.format(formatter);

        assert_eq!((prompt.formatter)(&prompt, &renderer), EXPECTED_VALUE);
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
            prompt.cursor.focused = 0;
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.cursor.focused, 1);
        }

        // move next in loop
        prompt.in_loop(true);

        for key in next_keys {
            prompt.cursor.focused = 2;
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.cursor.focused, 0);
        }

        // move next
        prompt.in_loop(false);

        for key in prev_keys {
            prompt.cursor.focused = 2;
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.cursor.focused, 1);
        }

        // move next in loop
        prompt.in_loop(true);

        for key in prev_keys {
            prompt.cursor.focused = 0;
            prompt.handle_key(KeyEvent::from(key));

            assert_eq!(prompt.cursor.focused, 2);
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

        prompt.cursor.focused = 1;
        prompt.handle_key(KeyEvent::from(KeyCode::Char(' ')));

        // must not update over limit
        prompt.cursor.focused = 2;
        prompt.handle_key(KeyEvent::from(KeyCode::Char(' ')));

        assert!(prompt.options[1].active);
        assert!(!prompt.options[2].active);
    }

}
