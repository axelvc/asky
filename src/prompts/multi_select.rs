use std::io;

use crate::Error;
#[cfg(feature = "bevy")]
use crate::bevy::*;
use crate::Valuable;
#[cfg(feature = "bevy")]
use bevy::input::keyboard::KeyCode as BKeyCode;

#[cfg(feature = "terminal")]
use crossterm::event::{KeyCode, KeyEvent};

#[cfg(feature = "terminal")]
use crate::utils::key_listener;
use crate::utils::key_listener::Typeable;

use crate::utils::{
    renderer::{DrawTime, Printable, Renderer},
    theme,
};

use super::select::{Direction, SelectInput, SelectOption};

use colored::ColoredStrings;

type Formatter<'a, T> = dyn Fn(&MultiSelect<T>, DrawTime, &mut ColoredStrings) + 'a + Send + Sync;

/// Prompt to select multiple items from a list.
///
/// To allow only one item to be selected, it is recommended to use [`Select`] struct instead.
/// # Key Events
///
/// | Key                  | Action                          |
/// | -------------------- | ------------------------------- |
/// | `Enter`, `Backspace` | Submit current/initial value    |
/// | `Space`              | Toggle selected in focused item |
/// | `Up`, `k`, `K`       | Focus next item                 |
/// | `Down`, `j`, `J`     | Focus previous item             |
/// | `Left`, `h`, `H`     | Focus next page                 |
/// | `Right`, `l`, `L`    | Focus previous page             |
///
/// # Examples
///
/// ```no_run
/// use asky::MultiSelect;
///
/// # fn main() -> std::io::Result<()> {
/// let options = ["Horror", "Romance", "Action", "Comedy"];
/// let answer = MultiSelect::new("What genre do you like?", options).prompt()?;
/// # Ok(())
/// # }
/// ```
/// [`Select`]: crate::Select
pub struct MultiSelect<'a, T> {
    /// Message used to display in the prompt.
    pub message: &'a str,
    /// List of options.
    pub options: Vec<SelectOption<'a, T>>,
    /// Minimum number of items required to be selected.
    pub min: Option<usize>,
    /// Maximum number of items allowed to be selected.
    pub max: Option<usize>,
    /// Input state.
    pub input: SelectInput,
    selected_count: usize,
    formatter: Box<Formatter<'a, T>>,
}

impl<'a, T: 'a> MultiSelect<'a, T> {
    /// Create a new multi-select prompt.
    pub fn new<I>(message: &'a str, iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: ToString,
    {
        let options = iter.into_iter().map(|o| SelectOption::new(o)).collect();
        Self::new_complex(message, options)
    }

    /// Create a new multi-select prompt with custom [`SelectOption`] items.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use asky::{MultiSelect, SelectOption};
    ///
    /// # fn main() -> std::io::Result<()> {
    /// let options = vec![
    ///     SelectOption::new("Reading"),
    ///     SelectOption::new("Watching TV"),
    ///     SelectOption::new("Playing video games"),
    ///     SelectOption::new("Sleeping"),
    /// ];
    ///
    /// MultiSelect::new_complex("How do you like to spend your free time?", options).prompt()?;
    /// # Ok(())
    /// # }
    pub fn new_complex(message: &'a str, options: Vec<SelectOption<'a, T>>) -> Self {
        let options_len = options.len();

        MultiSelect {
            message,
            options,
            min: None,
            max: None,
            selected_count: 0,
            input: SelectInput::new(options_len),
            formatter: Box::new(theme::fmt_multi_select2),
        }
    }

    /// Set initial selected indices.
    pub fn selected(&mut self, indices: &[usize]) -> &mut Self {
        for i in indices {
            if let Some(option) = self.options.get_mut(*i) {
                option.active = true;
                self.selected_count += 1;
            }
        }

        self
    }

    /// Set whether the cursor should go to the first option when it reaches the last option and vice-versa.
    pub fn in_loop(&mut self, is_loop: bool) -> &mut Self {
        self.input.set_loop_mode(is_loop);
        self
    }

    /// Set number of items per page to display.
    pub fn items_per_page(&mut self, items_per_page: usize) -> &mut Self {
        self.input.set_items_per_page(items_per_page);
        self
    }

    /// Set minimum number of items required to be selected.
    pub fn min(&mut self, min: usize) -> &mut Self {
        self.min = Some(min);
        self
    }

    /// Set maximum number of items allowed to be selected.
    pub fn max(&mut self, max: usize) -> &mut Self {
        self.max = Some(max);
        self
    }

    /// Set custom closure to format the prompt.
    ///
    /// See: [`Customization`](index.html#customization).
    pub fn format<F>(&mut self, formatter: F) -> &mut Self
    where
        F: Fn(&MultiSelect<T>, DrawTime, &mut ColoredStrings) + 'a + Send + Sync,
    {
        self.formatter = Box::new(formatter);
        self
    }

    #[cfg(feature = "terminal")]
    /// Display the prompt and return the user answer.
    pub fn prompt(&mut self) -> io::Result<Vec<T>> {
        key_listener::listen(self, true)?;

        let (selected, _): (Vec<_>, Vec<_>) = self.options.drain(..).partition(|x| x.active);
        let selected = selected.into_iter().map(|x| x.value).collect();

        Ok(selected)
    }
}

impl<T> MultiSelect<'_, T> {
    fn toggle_focused(&mut self) {
        let selected = self.input.focused;
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

    /// Only submit if the minimum are selected
    fn validate_to_submit(&self) -> bool {
        match self.min {
            None => true,
            Some(min) => self.selected_count >= min,
        }
    }
}

impl<T> Valuable for MultiSelect<'_, T> {
    type Output = u8;
    fn value(&self) -> Result<u8, Error> {
        todo!("implement as bitfield");
    }
}

#[cfg(feature = "terminal")]
impl<T> Typeable<KeyEvent> for MultiSelect<'_, T> {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
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

#[cfg(feature = "bevy")]
impl<T> Typeable<BKeyCode> for MultiSelect<'_, T> {
    fn handle_key(&mut self, key: &BKeyCode) -> bool {
        let mut submit = false;

        match key {
            // submit
            BKeyCode::Return | BKeyCode::Back => submit = self.validate_to_submit(),
            BKeyCode::Space => self.toggle_focused(),
            // update value
            BKeyCode::Up | BKeyCode::K => self.input.move_cursor(Direction::Up),
            BKeyCode::Down | BKeyCode::J => self.input.move_cursor(Direction::Down),
            BKeyCode::Left | BKeyCode::H => self.input.move_cursor(Direction::Left),
            BKeyCode::Right | BKeyCode::L => self.input.move_cursor(Direction::Right),
            _ => (),
        }

        submit
    }
}

impl<T> Printable for MultiSelect<'_, T> {
    fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
        let mut out = ColoredStrings::default();
        (self.formatter)(self, renderer.draw_time(), &mut out);
        renderer.print(out)
    }
}

#[cfg(feature = "bevy")]
impl<T> Printable for AskyNode<MultiSelect<'_, T>> {
    fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
        let mut out = ColoredStrings::default();
        (self.formatter)(self, renderer.draw_time(), &mut out);
        renderer.print(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_selected_values() {
        let mut prompt = MultiSelect::new("", ["a", "b", "c"]);

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
        let mut prompt = MultiSelect::new("", ["a", "b", "c"]);

        prompt.in_loop(false);
        assert!(!prompt.input.loop_mode);
        prompt.in_loop(true);
        assert!(prompt.input.loop_mode);
    }

    #[test]
    fn set_custom_formatter() {
        let mut prompt: MultiSelect<u8> = MultiSelect::new("", vec![]);
        let draw_time = DrawTime::First;
        const EXPECTED_VALUE: &str = "foo";

        prompt.format(|_, _, out| out.push(EXPECTED_VALUE.into()));
        let mut out = ColoredStrings::new();
        (prompt.formatter)(&prompt, draw_time, &mut out);
        assert_eq!(format!("{}", out), EXPECTED_VALUE);
    }

    #[test]
    fn submit_keys() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = MultiSelect::new("", ["a", "b", "c"]);
            let simulated_key = KeyEvent::from(event);

            let submit = prompt.handle_key(&simulated_key);
            assert!(submit);
        }
    }

    #[test]
    fn not_submit_without_min() {
        let mut prompt = MultiSelect::new("", ["a", "b", "c"]);

        prompt.min(1);
        let mut submit = prompt.handle_key(&KeyEvent::from(KeyCode::Enter));

        assert!(!submit);

        prompt.handle_key(&KeyEvent::from(KeyCode::Char(' ')));
        submit = prompt.handle_key(&KeyEvent::from(KeyCode::Enter));

        assert!(submit);
    }

    #[test]
    fn move_cursor() {
        let mut prompt = MultiSelect::new("", ["a", "b", "c"]);
        let prev_keys = [KeyCode::Up, KeyCode::Char('k'), KeyCode::Char('K')];
        let next_keys = [KeyCode::Down, KeyCode::Char('j'), KeyCode::Char('j')];

        // move next
        prompt.in_loop(false);

        for key in next_keys {
            prompt.input.focused = 0;
            prompt.handle_key(&KeyEvent::from(key));

            assert_eq!(prompt.input.focused, 1);
        }

        // move next in loop
        prompt.in_loop(true);

        for key in next_keys {
            prompt.input.focused = 2;
            prompt.handle_key(&KeyEvent::from(key));

            assert_eq!(prompt.input.focused, 0);
        }

        // move next
        prompt.in_loop(false);

        for key in prev_keys {
            prompt.input.focused = 2;
            prompt.handle_key(&KeyEvent::from(key));

            assert_eq!(prompt.input.focused, 1);
        }

        // move next in loop
        prompt.in_loop(true);

        for key in prev_keys {
            prompt.input.focused = 0;
            prompt.handle_key(&KeyEvent::from(key));

            assert_eq!(prompt.input.focused, 2);
        }
    }

    #[test]
    fn update_focused_selected() {
        let mut prompt = MultiSelect::new("", ["a", "b", "c"]);

        prompt.max(1);

        assert!(!prompt.options[1].active);
        assert!(!prompt.options[2].active);

        prompt.input.focused = 1;
        prompt.handle_key(&KeyEvent::from(KeyCode::Char(' ')));

        // must not update over limit
        prompt.input.focused = 2;
        prompt.handle_key(&KeyEvent::from(KeyCode::Char(' ')));

        assert!(prompt.options[1].active);
        assert!(!prompt.options[2].active);
    }
}
