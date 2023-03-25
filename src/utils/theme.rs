use colored::Colorize;

use crate::prompts::select::SelectOptionData;

use super::renderer::DrawTime;

pub trait Theme {
    /// Formats `Line` prompts
    /// Due a important part of this prompt is the cursor position
    /// you can return an optional tuple of (`row`, `col`) to set the initial cursor position.
    /// If you return `None` then the cursor position will be at the end of the formatted string.
    ///
    /// ---
    /// The cursor position will be relative to the position before print the formatted string
    ///
    /// Example:
    ///
    /// ```md
    /// |What's your name?
    /// ~
    /// <error>
    /// ```
    ///
    /// Where "`|`" is the initial cursor position, if you want the cursor after the "`~`" character
    /// must be return (1, 2), which will set the cursor in the following position
    ///
    /// Example:
    ///
    /// ```md
    /// What's your name?
    /// ~ |
    /// <error>
    /// ```
    fn fmt_text(
        &self,
        message: &str,
        draw_time: &DrawTime,
        input: &str,
        placeholder: &Option<&str>,
        default_value: &Option<&str>,
        validator_result: &Result<(), &str>,
    ) -> (String, Option<(u16, u16)>);

    /// Formats `Password` prompt
    /// Due a important part of this prompt is the cursor position
    /// you can return an optional tuple of (`row`, `col`) to set the initial cursor position.
    /// If you return `None` then the cursor position will be at the end of the formatted string.
    ///
    /// ---
    /// The cursor position will be relative to the position before print the formatted string
    ///
    /// Example:
    ///
    /// ```md
    /// |What's your name?
    /// ~
    /// <error>
    /// ```
    ///
    /// Where "`|`" is the initial cursor position, if you want the cursor after the "`~`" character
    /// must be return (1, 2), which will set the cursor in the following position
    ///
    /// Example:
    ///
    /// ```md
    /// What's your name?
    /// ~ |
    /// <error>
    /// ```
    fn fmt_password(
        &self,
        message: &str,
        draw_time: &DrawTime,
        input: &str,
        placeholder: &Option<&str>,
        default_value: &Option<&str>,
        validator_result: &Result<(), &str>,
        is_hidden: bool,
    ) -> (String, Option<(u16, u16)>);

    /// Formats `Number` prompt
    /// Due a important part of this prompt is the cursor position
    /// you can return an optional tuple of (`row`, `col`) to set the initial cursor position.
    /// If you return `None` then the cursor position will be at the end of the formatted string.
    ///
    /// ---
    /// The cursor position will be relative to the position before print the formatted string
    ///
    /// Example:
    ///
    /// ```md
    /// |What's your name?
    /// ~
    /// <error>
    /// ```
    ///
    /// Where "`|`" is the initial cursor position, if you want the cursor after the "`~`" character
    /// must be return (1, 2), which will set the cursor in the following position
    ///
    /// Example:
    ///
    /// ```md
    /// What's your name?
    /// ~ |
    /// <error>
    /// ```
    fn fmt_number(
        &self,
        message: &str,
        draw_time: &DrawTime,
        input: &str,
        placeholder: &Option<&str>,
        default_value: &Option<&str>,
        validator_result: &Result<(), &str>,
    ) -> (String, Option<(u16, u16)>);

    /// Formats `Toggle` prompt
    fn fmt_toggle(
        &self,
        message: &str,
        draw_time: &DrawTime,
        active: bool,
        options: (&str, &str),
    ) -> String;

    /// Formats `Confirm` prompt
    fn fmt_confirm(&self, message: &str, draw_time: &DrawTime, active: bool) -> String;

    // Formats `Select` prompt
    fn fmt_select(
        &self,
        message: &str,
        draw_time: &DrawTime,
        options: Vec<&SelectOptionData>,
        selected: usize,
        items_per_page: usize,
        page: usize,
        pages: usize,
    ) -> String;

    // Formats `MultiSelect` prompt
    fn fmt_multi_select(
        &self,
        message: &str,
        draw_time: &DrawTime,
        options: Vec<&SelectOptionData>,
        focused: usize,
        min: Option<usize>,
        max: Option<usize>,
        items_per_page: usize,
        page: usize,
        pages: usize,
    ) -> String;
}

pub struct DefaultTheme;

impl DefaultTheme {
    #[inline]
    fn text_prefix(&self, validator_result: &Result<(), &str>) -> String {
        let error = match validator_result {
            Ok(_) => "›".blue(),
            Err(_) => "›".red(),
        };

        error.to_string()
    }

    #[inline]
    fn text_default_value(&self, default_value: &Option<&str>) -> String {
        let value = match default_value {
            Some(value) => format!("Default: {}", value).bright_black(),
            None => "".normal(),
        };

        value.to_string()
    }

    #[inline]
    fn text_error(&self, validator_result: &Result<(), &str>) -> String {
        let error = match validator_result {
            Ok(_) => String::new(),
            Err(e) => format!("{}\n", e.red()),
        };

        error.to_string()
    }

    #[inline]
    fn text_input(&self, input: &str, placeholder: &Option<&str>, is_number: bool) -> String {
        let input = match (input.is_empty(), is_number) {
            (true, _) => placeholder.unwrap_or_default().bright_black(),
            (false, false) => input.normal(),
            (false, true) => input.yellow(),
        };

        input.to_string()
    }

    #[inline]
    fn select_prefix(&self, selected: bool) -> &'static str {
        match selected {
            true => "● ",
            false => "○ ",
        }
    }

    #[inline]
    fn message(&self, message: &str, draw_time: &DrawTime) -> String {
        let prefix = match draw_time {
            DrawTime::Last => "✓ ".green(),
            _ => "? ".blue(),
        };

        format!("{}{}", prefix, message)
    }

    #[inline]
    fn min_max_message(&self, min: Option<usize>, max: Option<usize>) -> String {
        match (min, max) {
            (None, None) => String::new(),
            (None, Some(max)) => format!("Max: {}", max),
            (Some(min), None) => format!("Min: {}", min),
            (Some(min), Some(max)) => format!("Min: {} · Max: {}", min, max),
        }
        .bright_black()
        .to_string()
    }

    #[inline]
    fn toggle_option(&self, option: &str, active: bool) -> String {
        let option = format!(" {} ", option);
        let option = match active {
            true => option.black().on_blue(),
            false => option.white().on_bright_black(),
        };

        option.to_string()
    }

    fn select_option_title(
        &self,
        SelectOptionData {
            title,
            description,
            disabled,
            ..
        }: &SelectOptionData,
        focused: bool,
    ) -> String {
        let title = match (disabled, focused) {
            (true, _) => title.bright_black().strikethrough(),
            (false, true) => title.blue(),
            (false, false) => title.normal(),
        };

        let make_description = |s: &str| format!(" · {}", s).bright_black();
        let description = match (focused, disabled, description) {
            (true, true, _) => make_description("(Disabled)"),
            (true, false, Some(description)) => make_description(description),
            _ => "".normal(),
        };

        format!("{} {}", title, description)
    }

    fn select_option(&self, option: &SelectOptionData, focused: bool) -> String {
        let prefix = self.select_prefix(focused);
        let prefix = match (focused, option.disabled) {
            (false, _) => prefix.bright_black(),
            (true, true) => prefix.yellow(),
            (true, false) => prefix.blue(),
        };

        format!("{}{}", prefix, self.select_option_title(option, focused))
    }

    fn multi_select_option(&self, option: &SelectOptionData, focused: bool) -> String {
        let prefix = self.select_prefix(option.active);
        let prefix = match (focused, option.disabled, option.active) {
            (true, true, _) => prefix.yellow(),
            (true, false, _) => prefix.blue(),
            (false, _, true) => prefix.normal(),
            (false, _, false) => prefix.bright_black(),
        };

        format!("{}{}", prefix, self.select_option_title(option, focused))
    }

    fn select_pagination(&self, page: usize, pages: usize) -> String {
        if pages == 1 {
            return String::new();
        }

        let icon = "•";

        format!(
            "\n\n  {}{}{}",
            icon.repeat(page).bright_black(),
            icon,
            icon.repeat(pages.saturating_sub(page + 1)).bright_black(),
        )
    }
}

impl Theme for DefaultTheme {
    fn fmt_text(
        &self,
        message: &str,
        draw_time: &DrawTime,
        input: &str,
        placeholder: &Option<&str>,
        default_value: &Option<&str>,
        validator_result: &Result<(), &str>,
    ) -> (String, Option<(u16, u16)>) {
        (
            format!(
                "{} {}\n{} {}\n{}",
                self.message(message, draw_time),
                self.text_default_value(default_value),
                self.text_prefix(validator_result),
                self.text_input(input, placeholder, false),
                self.text_error(validator_result),
            ),
            Some((1, 2)),
        )
    }

    fn fmt_password(
        &self,
        message: &str,
        draw_time: &DrawTime,
        input: &str,
        placeholder: &Option<&str>,
        default_value: &Option<&str>,
        validator_result: &Result<(), &str>,
        is_hidden: bool,
    ) -> (String, Option<(u16, u16)>) {
        let text = match is_hidden {
            true => String::new(),
            false => "*".repeat(input.len()),
        };

        self.fmt_text(
            message,
            draw_time,
            &text,
            placeholder,
            default_value,
            validator_result,
        )
    }

    fn fmt_number(
        &self,
        message: &str,
        draw_time: &DrawTime,
        input: &str,
        placeholder: &Option<&str>,
        default_value: &Option<&str>,
        validator_result: &Result<(), &str>,
    ) -> (String, Option<(u16, u16)>) {
        (
            format!(
                "{} {}\n{} {}\n{}",
                self.message(message, draw_time),
                self.text_default_value(default_value),
                self.text_prefix(validator_result),
                self.text_input(input, placeholder, true),
                self.text_error(validator_result),
            ),
            Some((1, 2)),
        )
    }

    fn fmt_toggle(
        &self,
        message: &str,
        draw_time: &DrawTime,
        active: bool,
        options: (&str, &str),
    ) -> String {
        format!(
            "{}\n{}  {}\n",
            self.message(message, draw_time),
            self.toggle_option(options.0, active == false),
            self.toggle_option(options.1, active == true),
        )
    }

    fn fmt_confirm(&self, message: &str, draw_time: &DrawTime, active: bool) -> String {
        self.fmt_toggle(message, draw_time, active, ("No", "Yes"))
    }

    fn fmt_select(
        &self,
        message: &str,
        draw_time: &DrawTime,
        options: Vec<&SelectOptionData>,
        selected: usize,
        items_per_page: usize,
        page: usize,
        pages: usize,
    ) -> String {
        let page_start_idx = page * items_per_page;
        let page_end_idx = (page_start_idx + items_per_page).min(options.len());
        let page_options = &options[page_start_idx..page_end_idx];
        let selected = selected % items_per_page;

        let page_options: Vec<String> = page_options
            .iter()
            .enumerate()
            .map(|(i, option)| self.select_option(option, selected == i))
            .collect();

        format!(
            "{}\n{}{}{}\n",
            self.message(message, draw_time),
            page_options.join("\n"),
            "\n".repeat(items_per_page - page_options.len()),
            self.select_pagination(page, pages),
        )
    }

    fn fmt_multi_select(
        &self,
        message: &str,
        draw_time: &DrawTime,
        options: Vec<&SelectOptionData>,
        focused: usize,
        min: Option<usize>,
        max: Option<usize>,
        items_per_page: usize,
        page: usize,
        pages: usize,
    ) -> String {
        let page_start_idx = page * items_per_page;
        let page_end_idx = (page_start_idx + items_per_page).min(options.len());
        let page_options = &options[page_start_idx..page_end_idx];
        let focused = focused % items_per_page;

        let page_options: Vec<String> = page_options
            .iter()
            .enumerate()
            .map(|(i, option)| self.multi_select_option(option, i == focused))
            .collect();

        format!(
            "{} {}\n{}{}{}\n",
            self.message(message, draw_time),
            self.min_max_message(min, max),
            page_options.join("\n"),
            "\n".repeat(items_per_page - page_options.len()),
            self.select_pagination(page, pages),
        )
    }
}
