use std::io;

use crossterm::{cursor, queue, style::Print, terminal};

use crate::prompts::{
    multi_select::MultiSelect,
    number::Number,
    password::Password,
    select::{Select, SelectOption, SelectOptionData},
    text::Text,
    toggle::Toggle,
};

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
        let (text, cursor) = state.theme.fmt_text(
            state.message,
            &self.draw_time,
            &state.value,
            &state.default_value,
            &state.validator_result,
        );

        self.render_line_prompt(text, state.cursor_col, cursor)?;
        self.out.flush()
    }

    pub fn password(&mut self, state: &Password) -> io::Result<()> {
        let (text, cursor) = state.handler.theme.fmt_password(
            state.handler.message,
            &self.draw_time,
            &state.handler.value,
            &state.handler.default_value,
            &state.handler.validator_result,
            state.hidden,
        );

        let cursor_col = match state.hidden {
            true => 0,
            false => state.handler.cursor_col,
        };

        self.render_line_prompt(text, cursor_col, cursor)?;
        self.out.flush()
    }

    pub fn number(&mut self, state: &Number) -> io::Result<()> {
        let (text, cursor) = state.handler.theme.fmt_number(
            state.handler.message,
            &self.draw_time,
            &state.handler.value,
            &state.handler.default_value,
            &state.handler.validator_result,
        );

        self.render_line_prompt(text, state.handler.cursor_col, cursor)?;
        self.out.flush()
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

    pub fn select<T>(&mut self, state: &Select<T>) -> io::Result<()> {
        if self.draw_time != DrawTime::First {
            queue!(self.out, cursor::RestorePosition)?;
        }

        queue!(
            self.out,
            cursor::SavePosition,
            terminal::Clear(terminal::ClearType::FromCursorDown),
            Print(state.theme.fmt_select(
                state.message,
                &self.draw_time,
                Self::get_select_options_data(&state.options),
                state.selected
            )),
        )?;

        self.out.flush()
    }

    pub fn multi_select<T>(&mut self, state: &MultiSelect<T>) -> io::Result<()> {
        if self.draw_time != DrawTime::First {
            queue!(self.out, cursor::RestorePosition)?;
        }

        queue!(
            self.out,
            cursor::SavePosition,
            terminal::Clear(terminal::ClearType::FromCursorDown),
            Print(state.theme.fmt_multi_select(
                state.message,
                &self.draw_time,
                Self::get_select_options_data(&state.options),
                state.focused
            )),
        )?;

        self.out.flush()
    }

    pub fn update_draw_time(&mut self) {
        self.draw_time = match self.draw_time {
            DrawTime::First => DrawTime::Update,
            _ => DrawTime::Last,
        }
    }

    fn render_line_prompt(
        &mut self,
        text: String,
        cursor_col: usize,
        initial_position: Option<(u16, u16)>,
    ) -> io::Result<()> {
        if self.draw_time != DrawTime::First {
            queue!(self.out, cursor::RestorePosition)?;
        }

        queue!(
            self.out,
            cursor::SavePosition,
            terminal::Clear(terminal::ClearType::FromCursorDown),
            Print(text)
        )?;

        if self.draw_time != DrawTime::Last {
            if let Some((row, col)) = initial_position {
                queue!(self.out, cursor::RestorePosition, cursor::SavePosition)?;

                if row > 0 {
                    queue!(self.out, cursor::MoveDown(row))?;
                }

                if col > 0 {
                    queue!(self.out, cursor::MoveRight(col))?;
                }
            }

            if cursor_col > 0 {
                queue!(self.out, cursor::MoveRight(cursor_col as u16))?;
            }
        }

        Ok(())
    }

    #[inline]
    fn get_select_options_data<'a, T>(
        options: &'a Vec<SelectOption<'a, T>>,
    ) -> Vec<SelectOptionData<'a>> {
        options.iter().map(|x| SelectOptionData::from(x)).collect()
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
    use crate::{
        prompts::select::SelectOption,
        utils::theme::{DefaultTheme, Theme},
    };

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
