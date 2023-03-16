use colored::Colorize;

use super::renderer::DrawTime;

pub trait Theme {
    /// Prefix used in `Line` prompts, before the user input
    fn text_prefix(&self) -> &'static str {
        "› "
    }

    /// Character that replaces the text at the `Password` prompt when it is not hidden
    fn password_char(&self) -> &'static str {
        "*"
    }

    /// Prefix used in the prompt message
    fn message_prefix(&self, draw_time: &DrawTime) -> &'static str {
        match draw_time {
            DrawTime::Last => "✓ ",
            _ => "? ",
        }
    }

    /// Prefix used in each prompt option of type `Select`.
    fn select_prefix(&self, selected: bool) -> &'static str {
        match selected {
            true => "● ",
            false => "○ ",
        }
    }

    /// Prefix used in each prompt option of type `MultiSelect`.
    fn multi_select_prefix(&self, selected: bool) -> &'static str {
        self.select_prefix(selected)
    }

    /// Formats message of any prompt
    fn fmt_message(&self, message: &str, draw_time: &DrawTime) -> String {
        let prefix = match draw_time {
            DrawTime::Last => self.message_prefix(draw_time).green(),
            _ => self.message_prefix(draw_time).blue(),
        };

        format!("{}{}", prefix, message)
    }

    /// Formats error messages
    fn fmt_error(&self, error: &str) -> String {
        error.red().to_string()
    }

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
        text: &str,
        placeholder: &str,
        validator_result: &Result<(), String>,
    ) -> (String, Option<(u16, u16)>) {
        let (prefix, error) = match validator_result {
            Err(e) => (self.text_prefix().red(), (String::from("\n") + e).red()),
            Ok(_) => (self.text_prefix().blue(), String::new().normal()),
        };

        let text = if text.is_empty() {
            placeholder.bright_black()
        } else {
            text.normal()
        };

        (
            format!(
                "{}\n{}{}{}\n",
                self.fmt_message(message, draw_time),
                prefix,
                text,
                error,
            ),
            Some((1, 2)),
        )
    }

    /// Formats `Toggle` prompt
    fn fmt_toggle(
        &self,
        message: &str,
        draw_time: &DrawTime,
        active: bool,
        options: (&str, &str),
    ) -> String {
        format!(
            "{}\n{}  {}\n",
            self.fmt_message(message, draw_time),
            self.fmt_toggle_option(options.0, active == false),
            self.fmt_toggle_option(options.1, active == true),
        )
    }

    /// Formats `Toggle` prompt option
    fn fmt_toggle_option(&self, option: &str, active: bool) -> String {
        let option = format!(" {} ", option);
        let option = match active {
            true => option.black().on_blue(),
            false => option.white().on_bright_black(),
        };

        option.to_string()
    }

    // Formats `Select` prompt
    fn fmt_select(&self, message: &str, draw_time: &DrawTime, options: &Vec<String>) -> String {
        format!(
            "{}\n{}\n",
            self.fmt_message(message, draw_time),
            options.join("\n")
        )
    }

    /// Formats `Select` prompt option
    fn fmt_select_option(
        &self,
        title: &str,
        description: Option<&str>,
        disabled: bool,
        active: bool,
    ) -> String {
        // prefix
        let prefix = self.select_prefix(active);
        let prefix = match (active, disabled) {
            (false, _) => prefix.bright_black(),
            (true, true) => prefix.yellow(),
            (true, false) => prefix.blue(),
        };

        // title
        let title = match (disabled, active) {
            (true, _) => title.bright_black().strikethrough(),
            (false, true) => title.blue(),
            (false, false) => title.normal(),
        };

        // description
        let make_description = |s: &str| format!(" · {}", s).bright_black();
        let description = match (active, disabled) {
            (false, _) => "".normal(),
            (true, true) => make_description("(Disabled)"),
            (true, false) => make_description(description.unwrap_or_default()),
        };

        format!("{}{}{}", prefix, title, description)
    }

    // Formats `MultiSelect` prompt
    fn fmt_multi_select(
        &self,
        message: &str,
        draw_time: &DrawTime,
        options: &Vec<String>,
    ) -> String {
        self.fmt_select(message, draw_time, options)
    }

    /// Formats `MultiSelect` prompt option
    fn fmt_multi_select_option(
        &self,
        title: &str,
        description: Option<&str>,
        disabled: bool,
        active: bool,
        focused: bool,
    ) -> String {
        // prefix
        let prefix = self.select_prefix(active);
        let prefix = match (focused, disabled, active) {
            (true, true, _) => prefix.yellow(),
            (true, false, _) => prefix.blue(),
            (false, _, true) => prefix.normal(),
            (false, _, false) => prefix.bright_black(),
        };

        // title
        let title = match (disabled, focused) {
            (true, _) => title.bright_black().strikethrough(),
            (false, true) => title.blue(),
            (false, false) => title.normal(),
        };

        // description
        let make_description = |s: &str| format!(" · {}", s).bright_black();
        let description = match (focused, disabled) {
            (false, _) => "".normal(),
            (true, true) => make_description("(Disabled)"),
            (true, false) => make_description(description.unwrap_or_default()),
        };

        format!("{}{}{}", prefix, title, description)
    }
}

pub struct DefaultTheme;
impl Theme for DefaultTheme {}