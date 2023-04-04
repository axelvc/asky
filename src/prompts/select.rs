use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::{
    key_listener::{self, Typeable},
    renderer::{DrawTime, Printable, Renderer},
    theme,
};

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// region: SelectOption

/// Utility struct to create items for select-like prompts (like [`Select`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectOption<'a, T> {
    /// Value that will be returned by the prompt when the user selects the option.
    pub value: T,
    /// String that will be displayed in the prompt.
    pub title: &'a str,
    /// Description text to show in the prompt when focus the option.
    pub description: Option<&'a str>,
    /// Indicate if the option is disabled.
    pub disabled: bool,
    /// Indicate if the option is active..
    ///
    /// **Note**: This field is only used for [`MultiSelect`] prompt, not for [`Select`] prompt.
    ///
    /// [`MultiSelect`]: crate::MultiSelect
    pub active: bool,
}

impl<'a, T> SelectOption<'a, T> {
    /// Create a new option.
    ///
    /// * `value`: value that will be returned by the prompt when the user selects the option.
    /// * `title`: string that will be displayed in the prompt.
    pub fn new(value: T, title: &'a str) -> Self {
        SelectOption {
            value,
            title,
            description: None,
            disabled: false,
            active: false,
        }
    }

    /// Description text to show in the prompt when focus the option.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Set whether the user can choose this option
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

// endregion: SelectOption

// region: SelectCursor

/// State of the input for select-like prompts (like [`Select`]).
///
/// **Note**: This structure is not expected to be created, but it can be consumed when using a custom formatter.
pub struct SelectInput {
    /// Focused index of the list.
    pub focused: usize,
    /// Number of items that must be displayed per page.
    pub items_per_page: usize,
    /// Indicate if the loop mode is enabled in the prompt.
    pub loop_mode: bool,
    /// Number of total items in the prompt.
    pub total_items: usize,
}

impl SelectInput {
    /// Returns the number of pages in the list.
    pub fn count_pages(&self) -> usize {
        let total = self.total_items;
        let per_page = self.items_per_page;
        let rem = total % per_page;

        total / per_page + (rem != 0) as usize
    }

    /// Returns the index of the current page.
    pub fn get_page(&self) -> usize {
        self.focused / self.items_per_page
    }
}

impl SelectInput {
    pub(crate) fn new(total_items: usize) -> Self {
        SelectInput {
            total_items,
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
        self.items_per_page = item_per_page.min(self.total_items);
    }

    fn prev_item(&mut self) {
        let max = self.total_items - 1;

        self.focused = match self.loop_mode {
            true => self.focused.checked_sub(1).unwrap_or(max),
            false => self.focused.saturating_sub(1),
        }
    }

    fn next_item(&mut self) {
        let max = self.total_items - 1;
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
        let max = self.total_items - 1;
        let new_value = self.focused + self.items_per_page;

        self.focused = new_value.min(max)
    }
}

// endregion: SelectCursor

type Formatter<'a, T> = dyn Fn(&Select<T>, DrawTime) -> String + 'a;

/// Prompt to select an item from a list.
///
/// To allow choosing multiple items, use the [`MultiSelect`] struct instead.
/// # Key Events
///
/// | Key                  | Action                       |
/// | -------------------- | ---------------------------- |
/// | `Enter`, `Backspace` | Submit current/initial value |
/// | `Up`, `k`, `K`       | Focus next item              |
/// | `Down`, `j`, `J`     | Focus previous item          |
/// | `Left`, `h`, `H`     | Focus next page              |
/// | `Right`, `l`, `L`    | Focus previous page          |
///
/// # Examples
///
/// ```no_run
/// use asky::{Select, SelectOption};
///
/// # fn main() -> std::io::Result<()> {
/// let options = vec![
///     SelectOption::new(1, "Horror"),
///     SelectOption::new(2, "Romance"),
///     SelectOption::new(3, "Action"),
///     SelectOption::new(4, "Comedy"),
/// ];
///
/// let answer = Select::new("What genre do you like?", options).prompt()?;
///
/// # Ok(())
/// # }
/// ```
/// [`MultiSelect`]: crate::MultiSelect
pub struct Select<'a, T> {
    /// Message used to display in the prompt.
    pub message: &'a str,
    /// List of options.
    pub options: Vec<SelectOption<'a, T>>,
    /// Input state.
    pub input: SelectInput,
    formatter: Box<Formatter<'a, T>>,
}

impl<'a, T: 'a> Select<'a, T> {
    /// Create a new select prompt.
    pub fn new(message: &'a str, options: Vec<SelectOption<'a, T>>) -> Self {
        let options_len = options.len();

        Select {
            message,
            options,
            input: SelectInput::new(options_len),
            formatter: Box::new(theme::fmt_select),
        }
    }

    /// Set initial selected index.
    pub fn selected(&mut self, index: usize) -> &mut Self {
        self.input.focused = index.min(self.options.len() - 1);
        self
    }

    /// Set whether the cursor should go to the first option when it reaches the last option and vice-versa.
    pub fn in_loop(&mut self, loop_mode: bool) -> &mut Self {
        self.input.set_loop_mode(loop_mode);
        self
    }

    /// Set number of items per page to display.
    pub fn items_per_page(&mut self, item_per_page: usize) -> &mut Self {
        self.input.set_items_per_page(item_per_page);
        self
    }

    /// Set custom closure to format the prompt.
    ///
    /// See: [`Customization`](index.html#customization).
    pub fn format<F>(&mut self, formatter: F) -> &mut Self
    where
        F: Fn(&Select<T>, DrawTime) -> String + 'a,
    {
        self.formatter = Box::new(formatter);
        self
    }

    /// Display the prompt and return the user answer.
    pub fn prompt(&mut self) -> io::Result<T> {
        key_listener::listen(self)?;

        let selected = self.options.remove(self.input.focused);

        Ok(selected.value)
    }
}

impl<T> Select<'_, T> {
    /// Only submit if the option isn't disabled.
    fn validate_to_submit(&self) -> bool {
        let focused = &self.options[self.input.focused];

        !focused.disabled
    }
}

impl<T> Typeable for Select<'_, T> {
    fn handle_key(&mut self, key: KeyEvent) -> bool {
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

        submit
    }
}

impl<T> Printable for Select<'_, T> {
    fn draw(&self, renderer: &mut Renderer) -> io::Result<()> {
        let text = (self.formatter)(self, renderer.draw_time);
        renderer.print(text)
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
        prompt.selected(1);
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
    fn set_custom_formatter() {
        let mut prompt: Select<u8> = Select::new("", vec![]);
        let draw_time = DrawTime::First;
        const EXPECTED_VALUE: &str = "foo";

        prompt.format(|_, _| String::from(EXPECTED_VALUE));

        assert_eq!((prompt.formatter)(&prompt, draw_time), EXPECTED_VALUE);
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

            prompt.selected(1);

            let submit = prompt.handle_key(simulated_key);
            assert_eq!(prompt.input.focused, 1);
            assert!(submit);
        }
    }

    #[test]
    fn not_submit_disabled() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = Select::new("", vec![SelectOption::new("foo", "foo").disabled(true)]);

            let submit = prompt.handle_key(KeyEvent::from(event));
            assert!(!submit);
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

                prompt.selected(initial);
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

                prompt.selected(initial);
                prompt.in_loop(in_loop);
                prompt.handle_key(simulated_key);
                assert_eq!(prompt.input.focused, expected);
            }
        }
    }
}
