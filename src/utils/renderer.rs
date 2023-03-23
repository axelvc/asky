use std::io;

use crossterm::{cursor, queue, style::Print, terminal};

use crate::prompts::{
    confirm::Confirm,
    multi_select::MultiSelect,
    number::{Num, Number},
    password::Password,
    select::Select,
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
            &state.input.value,
            &state.placeholder,
            &state.default_value,
            &state.validator_result,
        );

        self.print(&text)?;
        self.update_cursor(state.input.col, cursor)
    }

    pub fn password(&mut self, state: &Password) -> io::Result<()> {
        let (text, cursor) = state.theme.fmt_password(
            state.message,
            &self.draw_time,
            &state.input.value,
            &state.placeholder,
            &state.default_value,
            &state.validator_result,
            state.hidden,
        );

        let cursor_col = match state.hidden {
            true => 0,
            false => state.input.col,
        };

        self.print(&text)?;
        self.update_cursor(cursor_col, cursor)
    }

    pub fn number<T: Num>(&mut self, state: &Number<T>) -> io::Result<()> {
        let (text, cursor) = state.theme.fmt_number(
            state.message,
            &self.draw_time,
            &state.input.value,
            &state.placeholder,
            &state.default_value.as_ref().map(|x| x.as_str()),
            &state.validator_result,
        );

        self.print(&text)?;
        self.update_cursor(state.input.col, cursor)
    }

    pub fn toggle(&mut self, state: &Toggle) -> io::Result<()> {
        let text =
            state
                .theme
                .fmt_toggle(state.message, &self.draw_time, state.active, state.options);

        self.print(&text)
    }

    pub fn confirm(&mut self, state: &Confirm) -> io::Result<()> {
        let text = state
            .theme
            .fmt_confirm(state.message, &self.draw_time, state.active);

        self.print(&text)
    }

    pub fn select<T>(&mut self, state: &Select<T>) -> io::Result<()> {
        let text = state.theme.fmt_select(
            state.message,
            &self.draw_time,
            state.options.iter().map(|x| &x.data).collect(),
            state.selected,
        );

        self.print(&text)
    }

    pub fn multi_select<T>(&mut self, state: &MultiSelect<T>) -> io::Result<()> {
        let text = state.theme.fmt_multi_select(
            state.message,
            &self.draw_time,
            state.options.iter().map(|x| &x.data).collect(),
            state.focused,
            state.min,
            state.max,
        );

        self.print(&text)
    }

    pub fn update_draw_time(&mut self) {
        self.draw_time = match self.draw_time {
            DrawTime::First => DrawTime::Update,
            _ => DrawTime::Last,
        }
    }

    fn print(&mut self, text: &str) -> io::Result<()> {
        if self.draw_time != DrawTime::First {
            queue!(
                self.out,
                cursor::RestorePosition,
                terminal::Clear(terminal::ClearType::FromCursorDown),
            )?;
        }

        queue!(self.out, Print(&text))?;

        // Saved position is updated each draw because the text lines could be different
        // between draws. The last draw is ignored to always set the cursor at the end
        //
        // The position is saved this way to ensure the correct position when the cursor is at
        // the bottom of the terminal. Otherwise, the saved position will be the last row
        // and when trying to restore, the next draw will be below the last row.
        if self.draw_time != DrawTime::Last {
            let (col, row) = cursor::position()?;
            let text_lines = text.lines().count() as u16;

            queue!(
                self.out,
                cursor::MoveToPreviousLine(text_lines),
                cursor::SavePosition,
                cursor::MoveTo(col, row)
            )?;
        }

        self.out.flush()
    }

    fn update_cursor(
        &mut self,
        cursor_col: usize,
        initial_position: Option<(u16, u16)>,
    ) -> io::Result<()> {
        if self.draw_time != DrawTime::Last {
            if let Some((row, col)) = initial_position {
                queue!(self.out, cursor::RestorePosition)?;

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
