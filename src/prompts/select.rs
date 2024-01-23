use std::borrow::Cow;
use std::io;

use crate::utils::{
    renderer::{DrawTime, Printable, Renderer},
    theme,
};

use crate::{ColoredStrings, Error, Valuable};

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
use crate::style::{DefaultStyle, Flags, Region, Section, Style};
use crossterm::{queue, style::Print};

// region: SelectOption

/// Utility struct to create items for select-like prompts (like [`Select`]).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SelectOption<'a, T> {
    /// Value that will be returned by the prompt when the user selects the option.
    pub value: T,
    /// String that will be displayed in the prompt.
    pub title: String,
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

impl<'a, T: ToString> SelectOption<'a, T> {
    /// Create a new option.
    ///
    /// * `value`: value that will be returned by the prompt when the user selects the option.
    pub fn new(value: T) -> Self {
        let title = value.to_string();

        SelectOption {
            value,
            title,
            description: None,
            disabled: false,
            active: false,
        }
    }

    /// Create a new option with a custom title.
    ///
    /// * `value`: value that will be returned by the prompt when the user selects the option.
    /// * `title`: string that will be displayed in the prompt.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title.to_string();
        self
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

type Formatter<'a, T> = dyn Fn(&Select<T>, DrawTime, &mut ColoredStrings) + 'a + Send + Sync;

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
/// use asky::prelude::*;
///
/// # fn main() -> Result<(), Error> {
/// let languages = ["Rust", "Go", "Python", "Javascript", "Brainfuck", "Other"];
/// # #[cfg(feature = "terminal")]
/// let answer = Select::new("What is your favorite language?", languages).prompt()?;
/// # Ok(())
/// # }
/// ```
/// [`MultiSelect`]: crate::MultiSelect
pub struct Select<'a, T> {
    /// Message used to display in the prompt.
    pub message: Cow<'a, str>,
    /// List of options.
    pub options: Vec<SelectOption<'a, T>>,
    /// Input state.
    pub input: SelectInput,
    pub(crate) formatter: Box<Formatter<'a, T>>,
}

impl<'a, T: 'a> Select<'a, T> {
    /// Create a new select prompt.
    pub fn new<I>(message: impl Into<Cow<'a, str>>, iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: ToString,
    {
        let options = iter.into_iter().map(|o| SelectOption::new(o)).collect();
        Self::new_complex(message, options)
    }

    /// Create a new select prompt with custom [`SelectOption`] items.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use asky::prelude::*;
    ///
    /// # fn main() -> Result<(), Error> {
    /// let options = vec![
    ///     SelectOption::new(1),
    ///     SelectOption::new(2),
    ///     SelectOption::new(3),
    ///     SelectOption::new(4).title("Fish"),
    /// ];
    ///
    /// # #[cfg(feature = "terminal")]
    /// Select::new_complex("Choose a number", options).prompt()?;
    /// # Ok(())
    /// # }
    pub fn new_complex(
        message: impl Into<Cow<'a, str>>,
        options: Vec<SelectOption<'a, T>>,
    ) -> Self {
        let options_len = options.len();

        Select {
            message: message.into(),
            options,
            input: SelectInput::new(options_len),
            formatter: Box::new(theme::fmt_select2),
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
        F: Fn(&Select<T>, DrawTime, &mut ColoredStrings) + 'a + Send + Sync,
    {
        self.formatter = Box::new(formatter);
        self
    }
}

impl<T> Select<'_, T> {
    /// Only submit if the option isn't disabled.
    pub(crate) fn validate_to_submit(&self) -> bool {
        let focused = &self.options[self.input.focused];

        !focused.disabled
    }
}

impl<T> Valuable for Select<'_, T> {
    type Output = usize;
    fn value(&self) -> Result<usize, Error> {
        let focused = &self.options[self.input.focused];

        if !focused.disabled {
            Ok(self.input.focused)
        } else {
            Err(Error::InvalidCount {
                expected: 1,
                actual: 0,
            })
        }
    }
}

impl<T> Printable for Select<'_, T> {
    fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
        use Section::*;
        let draw_time = renderer.draw_time();
        let style = DefaultStyle { ascii: true };

        renderer.print2(|writer| {
            if draw_time == DrawTime::Last {
                queue!(
                    writer,
                    style.begin(Query(true)),
                    Print(self.message.to_string()),
                    style.end(Query(true)),
                    style.begin(Answer(true)),
                    Print(&self.options[self.input.focused].title),
                    style.end(Answer(true)),
                )?;
                Ok(1)
            } else {
                queue!(
                    writer,
                    style.begin(Query(false)),
                    Print(self.message.to_string()),
                    style.end(Query(false)),
                )?;

                let items_per_page = self.input.items_per_page;
                let total = self.input.total_items;

                let page_len = items_per_page.min(total);
                let page_start = self.input.get_page() * items_per_page;
                let page_end = (page_start + page_len).min(total);
                let page_focused = self.input.focused % items_per_page;

                for (n, option) in self.options[page_start..page_end].iter().enumerate() {
                    let mut flags = Flags::empty();
                    if (n == page_focused) {
                        flags |= Flags::Focused;
                    }
                    if (option.disabled) {
                        flags |= Flags::Disabled;
                    }
                    queue!(
                        writer,
                        style.begin(OptionExclusive(flags)),
                        Print(&option.title),
                        style.end(OptionExclusive(flags)),
                    )?;
                }

                let page_i = self.input.get_page() as u8;
                let page_count = self.input.count_pages() as u8;
                let page_footer = if page_count != 1 { 2 } else { 0 };
                queue!(
                    writer,
                    style.begin(Page(page_i, page_count)),
                    style.end(Page(page_i, page_count)),
                )?;
                Ok((2 + page_end - page_start + page_footer) as u16)
            }
        })
    }
}

#[cfg(feature = "terminal")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::key_listener::Typeable;
    use crossterm::event::{KeyCode, KeyEvent};

    #[test]
    fn set_initial_value() {
        let mut prompt = Select::new("", ["foo", "bar"]);

        assert_eq!(prompt.input.focused, 0);
        prompt.selected(1);
        assert_eq!(prompt.input.focused, 1);
    }

    #[test]
    fn set_loop_mode() {
        let mut prompt = Select::new("", ["foo", "bar"]);

        prompt.in_loop(false);
        assert!(!prompt.input.loop_mode);
        prompt.in_loop(true);
        assert!(prompt.input.loop_mode);
    }

    #[test]
    fn set_custom_formatter() {
        let mut prompt = Select::new("", ["foo", "bar"]);
        let draw_time = DrawTime::First;
        const EXPECTED_VALUE: &str = "foo";

        prompt.format(|_, _, out| out.push(EXPECTED_VALUE.into()));
        let mut out = ColoredStrings::new();
        (prompt.formatter)(&prompt, draw_time, &mut out);
        assert_eq!(format!("{}", out), EXPECTED_VALUE);
    }

    #[test]
    fn submit_selected_value() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = Select::new("", ["foo", "bar"]);
            let simulated_key = KeyEvent::from(event);

            prompt.selected(1);

            let submit = prompt.handle_key(&simulated_key);
            assert_eq!(prompt.input.focused, 1);
            assert!(submit);
        }
    }

    #[test]
    fn not_submit_disabled() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = Select::new_complex("", vec![SelectOption::new("foo").disabled(true)]);

            let submit = prompt.handle_key(&KeyEvent::from(event));
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
                let mut prompt = Select::new("", ["foo", "bar"]);
                let simulated_key = KeyEvent::from(key);

                prompt.selected(initial);
                prompt.in_loop(in_loop);
                prompt.handle_key(&simulated_key);
                assert_eq!(prompt.input.focused, expected);
            }
        }

        for key in down_keys {
            for (in_loop, initial, expected) in down_cases {
                let mut prompt = Select::new("", ["foo", "bar"]);
                let simulated_key = KeyEvent::from(key);

                prompt.selected(initial);
                prompt.in_loop(in_loop);
                prompt.handle_key(&simulated_key);
                assert_eq!(prompt.input.focused, expected);
            }
        }
    }
}
