use std::io::{self, Write};

use crossterm::{cursor, queue, style::Print, terminal};

pub trait Printable {
    fn draw(&self, renderer: &mut Renderer) -> io::Result<()>;
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum DrawTime {
    First,
    Update,
    Last,
}

pub struct Renderer {
    pub draw_time: DrawTime,
    out: io::Stdout,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            draw_time: DrawTime::First,
            out: io::stdout(),
        }
    }

    pub fn update_draw_time(&mut self) {
        self.draw_time = match self.draw_time {
            DrawTime::First => DrawTime::Update,
            _ => DrawTime::Last,
        }
    }

    pub fn print(&mut self, text: &str) -> io::Result<()> {
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

    pub fn update_cursor(
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
