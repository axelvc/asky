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
    /// Returns (`prefix`, `text`, `placeholder`)
    fn fmt_text(&self, text: &str, placeholder: &str, has_error: bool) -> (String, String, String) {
        let prefix = match has_error {
            true => self.text_prefix().red(),
            false => self.text_prefix().blue(),
        };

        (
            prefix.to_string(),
            text.to_string(),
            placeholder.bright_black().to_string(),
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
            false => option.black().on_blue(),
            true => option.white().on_bright_black(),
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
    fn fmt_select_option(&self, option: &str, selected: bool) -> String {
        let (prefix, option) = if selected {
            (self.select_prefix(true).blue(), option.blue())
        } else {
            (self.select_prefix(false).bright_black(), option.normal())
        };

        format!("{}{}", prefix, option)
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
    fn fmt_multi_select_option(&self, option: &str, selected: bool, focused: bool) -> String {
        let mut option = option.normal();

        let mut prefix = if selected {
            self.multi_select_prefix(true).normal()
        } else {
            self.multi_select_prefix(false).bright_black()
        };

        if focused {
            prefix = prefix.blue();
            option = option.blue();
        }

        format!("{}{}", prefix, option)
    }
}

pub struct DefaultTheme;
impl Theme for DefaultTheme {}
