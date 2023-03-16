use std::io;

use crossterm::{cursor, queue, style::Print, terminal};

use crate::prompts::{multi_select::MultiSelect, select::Select, text::Text, toggle::Toggle};

use super::theme::Theme;

#[derive(PartialEq, Debug)]
pub enum DrawTime {
    First,
    Update,
    Last,
}

pub struct Renderer<W: io::Write> {
    pub draw_time: DrawTime,
    out: W,
}

impl<W: io::Write> Renderer<W> {
    pub fn text(&mut self, state: &Text) -> io::Result<()> {
        self.draw_line(
            state.message,
            &state.value,
            &state.default_value,
            &state.validator_result,
            state.cursor_col,
            state.theme,
        )
    }

    pub fn password(&mut self, state: &Text, is_hidden: bool) -> io::Result<()> {
        let (value, cursor_col) = match is_hidden {
            true => (String::new(), 0),
            false => (
                state.theme.password_char().repeat(state.value.len()),
                state.cursor_col,
            ),
        };

        self.draw_line(
            state.message,
            &value,
            &state.default_value,
            &state.validator_result,
            cursor_col,
            state.theme,
        )
    }

    pub fn toggle(&mut self, state: &Toggle) -> io::Result<()> {
        if self.draw_time != DrawTime::First {
            queue!(self.out, cursor::RestorePosition)?;
        }

        queue!(
            self.out,
            cursor::SavePosition,
            Print(state.theme.fmt_toggle(
                state.message,
                &self.draw_time,
                state.active,
                state.options,
            ))
        )?;

        self.out.flush()
    }

    pub fn select<T: Copy>(&mut self, state: &Select<T>) -> io::Result<()> {
        if self.draw_time != DrawTime::First {
            queue!(self.out, cursor::RestorePosition)?;
        }

        // TODO: move this to theme module (for consistencie)
        let options: Vec<String> = state
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                state.theme.fmt_select_option(
                    option.title,
                    option.description,
                    option.disabled,
                    state.selected == i,
                )
            })
            .collect();

        queue!(
            self.out,
            cursor::SavePosition,
            terminal::Clear(terminal::ClearType::FromCursorDown),
            Print(
                state
                    .theme
                    .fmt_select(state.message, &self.draw_time, &options)
            ),
        )?;

        self.out.flush()
    }

    pub fn multi_select<T: Copy>(&mut self, state: &MultiSelect<T>) -> io::Result<()> {
        if self.draw_time != DrawTime::First {
            queue!(self.out, cursor::RestorePosition)?;
        }

        // TODO: move this to theme module (for consistencie)
        let options: Vec<String> = state
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                state.theme.fmt_multi_select_option(
                    option.title,
                    option.description,
                    option.disabled,
                    option.active,
                    i == state.focused,
                )
            })
            .collect();

        queue!(
            self.out,
            cursor::SavePosition,
            terminal::Clear(terminal::ClearType::FromCursorDown),
            Print(
                state
                    .theme
                    .fmt_multi_select(state.message, &self.draw_time, &options)
            ),
        )?;

        self.out.flush()
    }

    pub fn update_draw_time(&mut self) {
        self.draw_time = match self.draw_time {
            DrawTime::First => DrawTime::Update,
            _ => DrawTime::Last,
        }
    }

    // NOTE: try to find a way to let the user change the all the prompt
    fn draw_line(
        &mut self,
        message: &str,
        value: &str,
        default_value: &str,
        validator_result: &Result<(), String>,
        _cursor_col: usize,
        theme: &dyn Theme,
    ) -> io::Result<()> {
        if self.draw_time != DrawTime::First {
            queue!(self.out, cursor::RestorePosition)?;
        }

        let (prefix, value, placeholder) =
            theme.fmt_text(value, default_value, validator_result.is_err());

        // draw message
        queue!(
            self.out,
            cursor::SavePosition,
            Print(theme.fmt_message(message, &self.draw_time)),
            cursor::MoveToNextLine(1),
            terminal::Clear(terminal::ClearType::FromCursorDown),
        )?;

        // draw error
        match validator_result {
            Ok(_) => (),
            Err(error) => queue!(
                self.out,
                cursor::MoveToNextLine(1),
                Print(theme.fmt_error(error)),
                cursor::MoveToPreviousLine(1),
            )?,
        };

        // draw value or placeholder
        match value.is_empty() {
            false => queue!(self.out, Print(prefix), Print(value))?,
            true => queue!(
                self.out,
                Print(&prefix),
                Print(&placeholder),
                // reprint prefix to set cursor position
                // use `SavePosition` cause bugs on the updates renders
                cursor::MoveToColumn(0),
                Print(&prefix),
            )?,
        }

        // new line on last draw
        if let DrawTime::Last = self.draw_time {
            queue!(self.out, cursor::MoveToNextLine(1))?;
        }

        self.out.flush()
    }
}

impl Renderer<io::Stdout> {
    pub fn new() -> Renderer<io::Stdout> {
        Renderer {
            draw_time: DrawTime::First,
            out: io::stdout(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{prompts::select::SelectOption, utils::theme::DefaultTheme};

    use super::*;

    impl Renderer<Vec<u8>> {
        fn to_test() -> Renderer<Vec<u8>> {
            Renderer {
                draw_time: DrawTime::First,
                out: Vec::new(),
            }
        }
    }

    #[test]
    fn render_text_with_value() {
        let state = Text {
            message: "What's your name?",
            value: "Leopoldo".to_string(),
            default_value: "Goofy".to_string(),
            validator: None,
            validator_result: Ok(()),
            cursor_col: 0,
            submit: false,
            theme: &DefaultTheme,
        };

        let mut renderer = Renderer::to_test();
        renderer.text(&state).unwrap();

        let out_str = String::from_utf8(renderer.out).unwrap();

        assert!(out_str.contains(state.message));
        assert!(out_str.contains(&state.value));
        assert!(!out_str.contains(&state.default_value));
    }

    #[test]
    fn render_text_with_default_value() {
        let state = Text {
            message: "What's your name?",
            value: String::new(),
            default_value: "Goofy".to_string(),
            validator: None,
            validator_result: Ok(()),
            cursor_col: 0,
            submit: false,
            theme: &DefaultTheme,
        };

        let mut renderer = Renderer::to_test();

        renderer.text(&state).unwrap();

        let out_str = String::from_utf8(renderer.out).unwrap();

        assert!(out_str.contains(state.message));
        assert!(out_str.contains(&state.default_value));
    }

    #[test]
    fn render_text_with_error() {
        let state = Text {
            message: "What's your name?",
            value: "Leopoldo".to_string(),
            default_value: String::new(),
            validator: None,
            validator_result: Err("Invalid name".to_string()),
            cursor_col: 0,
            submit: false,
            theme: &DefaultTheme,
        };

        let mut renderer = Renderer::to_test();
        renderer.text(&state).unwrap();

        let out_str = String::from_utf8(renderer.out).unwrap();

        assert!(out_str.contains(state.message));
        assert!(out_str.contains(&state.value));
        assert!(out_str.contains(&state.validator_result.unwrap_err()));
    }

    #[test]
    fn render_toggle() {
        let cases = [false, true];

        for active in cases {
            let state = &Toggle {
                message: "Do you like pizza?",
                options: ("foo", "bar"),
                active,
                submit: false,
                theme: &DefaultTheme,
            };

            let mut renderer = Renderer::to_test();

            renderer.toggle(&state).unwrap();

            let out_str = String::from_utf8(renderer.out).unwrap();
            let expeted_option_str =
                DefaultTheme.fmt_toggle(state.message, &renderer.draw_time, active, state.options);
            assert!(out_str.contains(&state.message));
            assert!(out_str.contains(&expeted_option_str));
        }
    }

    #[test]
    fn render_select() {
        let message = "What's your favorite number?";
        let mut renderer = Renderer::to_test();
        let state = Select {
            message,
            options: vec![
                SelectOption::new("1", "1"),
                SelectOption::new("2", "2"),
                SelectOption::new("fish", "fish"),
            ],
            selected: 2,
            is_loop: false,
            submit: false,
            theme: &DefaultTheme,
        };

        renderer.select(&state).unwrap();

        let out_str = String::from_utf8(renderer.out).unwrap();
        assert!(out_str.contains(&message));
        assert!(out_str.contains("1"));
        assert!(out_str.contains("2"));
        assert!(out_str.contains("fish"));
    }
}
