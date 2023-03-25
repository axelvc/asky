use std::{
    io,
    ops::{Deref, DerefMut},
};

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, KeyHandler},
    renderer::Renderer,
    theme::{DefaultTheme, Theme},
};

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct SelectOption<'a, T> {
    pub(crate) value: T,
    pub(crate) data: SelectOptionData<'a>,
}

/// Helper struct to pass SelectOption data to theme trait
pub struct SelectOptionData<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub disabled: bool,
    pub active: bool,
}

impl<'a, T> Deref for SelectOption<'a, T> {
    type Target = SelectOptionData<'a>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<'a, T> DerefMut for SelectOption<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<'a, T> SelectOption<'a, T> {
    pub fn new(value: T, title: &'a str) -> Self {
        Self {
            value,
            data: SelectOptionData {
                title,
                description: None,
                disabled: false,
                active: false,
            },
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
}

pub struct SelectInput<'a, T> {
    pub focused: usize,
    pub items_per_page: usize,
    pub(crate) options: Vec<SelectOption<'a, T>>,
    pub(crate) loop_mode: bool,
}

impl<'a, T> SelectInput<'a, T> {
    pub fn count_pages(&self) -> usize {
        let total = self.options.len();
        let per_page = self.items_per_page;
        let mut pages = total / per_page;

        if total % per_page != 0 {
            pages += 1
        }

        pages
    }

    pub fn page(&self) -> usize {
        self.focused / self.items_per_page
    }

    pub fn get_options_data(&self) -> Vec<&SelectOptionData> {
        self.options.iter().map(|x| &x.data).collect()
    }

    pub(crate) fn new(options: Vec<SelectOption<'a, T>>) -> Self {
        SelectInput {
            options,
            focused: 0,
            items_per_page: 10,
            loop_mode: true,
        }
    }

    pub(crate) fn set_loop_mode(&mut self, loop_mode: bool) {
        self.loop_mode = loop_mode;
    }

    pub(crate) fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.prev_item(),
            Direction::Down => self.next_item(),
            Direction::Left => self.prev_page(),
            Direction::Right => self.next_page(),
        };
    }

    pub(crate) fn set_items_per_page(&mut self, item_per_page: usize) {
        self.items_per_page = item_per_page.min(self.options.len());
    }

    fn prev_item(&mut self) {
        let max = self.options.len() - 1;

        self.focused = match self.loop_mode {
            true => self.focused.checked_sub(1).unwrap_or(max),
            false => self.focused.saturating_sub(1),
        }
    }

    fn next_item(&mut self) {
        let max = self.options.len() - 1;
        let new_value = self.focused + 1;

        self.focused = match (new_value > max, self.loop_mode) {
            (true, true) => 0,
            (true, false) => max,
            (false, _) => new_value,
        }
    }

    fn prev_page(&mut self) {
        self.focused = self.focused.saturating_sub(self.items_per_page)
    }

    fn next_page(&mut self) {
        let max = self.options.len() - 1;
        let new_value = self.focused + self.items_per_page;

        self.focused = new_value.min(max)
    }
}

pub struct Select<'a, T> {
    pub(crate) message: &'a str,
    pub(crate) input: SelectInput<'a, T>,
    pub(crate) submit: bool,
    pub(crate) theme: &'a dyn Theme,
}

impl<'a, T> Select<'a, T> {
    pub fn new(message: &'a str, options: Vec<SelectOption<'a, T>>) -> Select<'a, T> {
        Select {
            message,
            input: SelectInput::new(options),
            submit: false,
            theme: &DefaultTheme,
        }
    }

    pub fn initial(&mut self, value: usize) -> &mut Self {
        self.input.focused = value.min(self.input.options.len() - 1);
        self
    }

    pub fn in_loop(&mut self, loop_mode: bool) -> &mut Self {
        self.input.set_loop_mode(loop_mode);
        self
    }

    pub fn items_per_page(&mut self, item_per_page: usize) -> &mut Self {
        self.input.set_items_per_page(item_per_page);
        self
    }

    pub fn theme(&mut self, theme: &'a dyn Theme) -> &mut Self {
        self.theme = theme;
        self
    }

    pub fn prompt(&mut self) -> io::Result<T> {
        key_listener::listen(self)?;

        let selected = self.input.options.remove(self.input.focused);

        Ok(selected.value)
    }

    /// Only submit if the option isn't disabled
    fn validate_to_submit(&self) -> bool {
        let focused = &self.input.options[self.input.focused];

        !focused.disabled
    }
}

impl<'a, T> KeyHandler for Select<'a, T> {
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
            KeyCode::Up | KeyCode::Char('k' | 'K') => self.input.move_cursor(Direction::Up),
            KeyCode::Down | KeyCode::Char('j' | 'J') => self.input.move_cursor(Direction::Down),
            KeyCode::Left | KeyCode::Char('h' | 'H') => self.input.move_cursor(Direction::Left),
            KeyCode::Right | KeyCode::Char('l' | 'L') => self.input.move_cursor(Direction::Right),
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

        assert_eq!(prompt.input.focused, 0);
        prompt.initial(1);
        assert_eq!(prompt.input.focused, 1);
    }

    #[test]
    fn set_loop_mode() {
        let mut prompt = Select::new(
            "",
            vec![
                SelectOption::new("foo", "foo"),
                SelectOption::new("bar", "bar"),
            ],
        );

        prompt.in_loop(false);
        assert!(!prompt.input.loop_mode);
        prompt.in_loop(true);
        assert!(prompt.input.loop_mode);
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
            assert_eq!(prompt.input.focused, 1);
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
    fn update_focused() {
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
                assert_eq!(prompt.input.focused, expected);
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
                assert_eq!(prompt.input.focused, expected);
            }
        }
    }
}
