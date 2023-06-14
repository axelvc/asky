use std::io;

#[cfg(feature="bevy")]
use bevy::prelude::*;
#[cfg(feature="bevy")]
use crate::bevy::*;

#[cfg(feature="terminal")]
use crossterm::event::{KeyCode, KeyEvent};

use crate::utils::key_listener::Typeable;
#[cfg(feature="terminal")]
use crate::utils::key_listener;

use crate::utils::{
    renderer::{DrawTime, Printable, Renderer},
    theme,
};

use colored::{Colorize, ColoredString, ColoredStrings};
use super::select::{Direction, SelectInput, SelectOption};
use crate::utils::theme::*;

// type Formatter<'a, T> = dyn Fn(&MultiSelect<T>, DrawTime) -> String + 'a;

struct DefaultFormatter;

impl DefaultFormatter {

    fn fmt_message(&self, msg: &str, min: Option<usize>, max: Option<usize>, out: &mut ColoredStrings) {
        let min_max = match (min, max) {
            (None, None) => String::new(),
            (None, Some(max)) => format!("Max: {}", max),
            (Some(min), None) => format!("Min: {}", min),
            (Some(min), Some(max)) => format!("Min: {} · Max: {}", min, max),
        }
        .bright_black();
        fmt_message2(msg, out);
        out.0.extend([" ".into(), min_max]);
    }

    fn fmt_select_page_options<T>(&self,
    options: &[SelectOption<T>],
    input: &SelectInput,
    is_multiple: bool,
    out: &mut ColoredStrings
) {
    let items_per_page = input.items_per_page;
    let total = input.total_items;

    let page_len = items_per_page.min(total);
    let page_start = input.get_page() * items_per_page;
    let page_end = (page_start + page_len).min(total);
    let page_focused = input.focused % items_per_page;

    let mut page_options: Vec<ColoredStrings> = options[page_start..page_end]
        .iter()
        .enumerate()
        .map(|(i, option)| { let mut out = ColoredStrings::default();
                             self.fmt_select_option(option, page_focused == i, is_multiple, &mut out);
                             out })
        .collect();

    page_options.resize(page_len, ColoredStrings::default());
    // out.0.extend((*page_options.as_slice()).join("\n".into().into()));
    for page in page_options {
        out.0.extend(page.0);
        out.0.push("\n".into());
    }
}

fn fmt_select_pagination(&self, page: usize, pages: usize, out: &mut ColoredStrings) {
    if pages == 1 {
        return;
    }

    let icon = "•";

    out.0.extend([
        "\n  ".into(),
        icon.repeat(page).bright_black(),
        icon.into(),
        icon.repeat(pages.saturating_sub(page + 1)).bright_black(),
        ]);
}

fn fmt_select_option<T>(&self, option: &SelectOption<T>, focused: bool, multiple: bool, out: &mut ColoredStrings) {
    let prefix = if multiple {
        let prefix = match (option.active, focused) {
            (true, true) => "◉",
            (true, false) => "●",
            _ => "○",
        };

        match (focused, option.active, option.disabled) {
            (true, _, true) => prefix.red(),
            (true, _, false) => prefix.blue(),
            (false, true, _) => prefix.normal(),
            (false, false, _) => prefix.bright_black(),
        }
    } else {
        match (focused, option.disabled) {
            (false, _) => "○".bright_black(),
            (true, true) => "○".red(),
            (true, false) => "●".blue(),
        }
    };

    let title = &option.title;
    let title = match (option.disabled, focused) {
        (true, _) => title.bright_black().strikethrough(),
        (false, true) => title.blue(),
        (false, false) => title.normal(),
    };

    let make_description = |s: &str| format!(" · {}", s).bright_black();
    let description = match (focused, option.disabled, option.description) {
        (true, true, _) => make_description("(Disabled)"),
        (true, false, Some(description)) => make_description(description),
        _ => "".normal(),
    };

    out.0.extend([prefix, " ".into(), title, " ".into(), description]);
    // format!("{} {} {}", prefix, title, description)
}

}

pub trait Formatter<T> {
    fn format(&self, prompt: &MultiSelect<T>, time: DrawTime, out: &mut ColoredStrings<'_>);
}

impl<T> Formatter<T> for DefaultFormatter {

    fn format(&self, prompt: &MultiSelect<T>, draw_time: DrawTime, out: &mut ColoredStrings) {
        if draw_time == DrawTime::Last {
            return fmt_last_message2(
                prompt.message,
                &format!(
                    "[{}]",
                    prompt
                        .options
                        .iter()
                        .filter(|opt| opt.active)
                        .map(|opt| opt.title.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
                out
            );
        }

        self.fmt_message(prompt.message, prompt.min, prompt.max, out);
        out.0.push("\n".into());
        self.fmt_select_page_options(&prompt.options, &prompt.input, true, out);
        out.0.push("\n".into());
        self.fmt_select_pagination(prompt.input.get_page(), prompt.input.count_pages(), out);
    }
}
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
    formatter: Box<dyn Formatter<T>>,
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
            formatter: Box::new(DefaultFormatter)
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
    pub fn format<F: Formatter<T>>(&mut self, formatter: F) -> &mut Self
    {
        self.formatter = Box::new(formatter);
        self
    }

    #[cfg(feature="terminal")]
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

#[cfg(feature="terminal")]
impl<T> Typeable<KeyEvent> for MultiSelect<'_, T> {
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

// #[cfg(feature="bevy")]
// impl<T> Typeable<KeyEvent<'_, '_>> for MultiSelect<'_, T> {
//     fn handle_key(&mut self, mut key: KeyEvent) -> bool {
//         let mut submit = false;

//         for code in key.codes() {
//             match code {
//                 // submit
//                 KeyCode::Return | KeyCode::Back => submit = self.validate_to_submit(),
//                 // select/unselect
//                 KeyCode::Space => self.toggle_focused(),
//                 // update focus
//                 KeyCode::Up | KeyCode::K => self.input.move_cursor(Direction::Up),
//                 KeyCode::Down | KeyCode::J => self.input.move_cursor(Direction::Down),
//                 KeyCode::Left | KeyCode::H => self.input.move_cursor(Direction::Left),
//                 KeyCode::Right | KeyCode::L => self.input.move_cursor(Direction::Right),
//                 _ => (),
//             }
//         }

//         submit
//     }
// }

impl<T> Printable for MultiSelect<'_, T> {
    fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
        let out = default();
        self.formatter.format(self, renderer.draw_time(), &mut out);
        renderer.print(out);
        Ok(())
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

        prompt.format(|_, _| String::from(EXPECTED_VALUE));

        assert_eq!((prompt.formatter)(&prompt, draw_time), EXPECTED_VALUE);
    }

    #[test]
    fn submit_keys() {
        let events = [KeyCode::Enter, KeyCode::Backspace];

        for event in events {
            let mut prompt = MultiSelect::new("", ["a", "b", "c"]);
            let simulated_key = KeyEvent::from(event);

            let submit = prompt.handle_key(simulated_key);
            assert!(submit);
        }
    }

    #[test]
    fn not_submit_without_min() {
        let mut prompt = MultiSelect::new("", ["a", "b", "c"]);

        prompt.min(1);
        let mut submit = prompt.handle_key(KeyEvent::from(KeyCode::Enter));

        assert!(!submit);

        prompt.handle_key(KeyEvent::from(KeyCode::Char(' ')));
        submit = prompt.handle_key(KeyEvent::from(KeyCode::Enter));

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
        let mut prompt = MultiSelect::new("", ["a", "b", "c"]);

        prompt.max(1);

        assert!(!prompt.options[1].active);
        assert!(!prompt.options[2].active);

        prompt.input.focused = 1;
        prompt.handle_key(KeyEvent::from(KeyCode::Char(' ')));

        // must not update over limit
        prompt.input.focused = 2;
        prompt.handle_key(KeyEvent::from(KeyCode::Char(' ')));

        assert!(prompt.options[1].active);
        assert!(!prompt.options[2].active);
    }
}
