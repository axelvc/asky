use std::io::{self, Write};

use crossterm::{cursor, execute, queue, style::Print, terminal};

pub trait Printable {
    fn draw(&self, renderer: &mut Renderer) -> io::Result<()>;
}

/// Enum that indicates the current draw time to format closures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DrawTime {
    /// First time that a prompt is displayed.
    #[default]
    First,
    /// The prompt state has been updated.
    Update,
    /// The last time that a prompt is displayed.
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

    pub fn print(&mut self, mut text: String) -> io::Result<()> {
        if self.draw_time != DrawTime::First {
            queue!(
                self.out,
                cursor::RestorePosition,
                terminal::Clear(terminal::ClearType::FromCursorDown),
            )?;
        }

        if !text.ends_with('\n') {
            text.push('\n')
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

    /// Utility function for line input
    /// Set initial position based on the position after drawing
    pub fn set_cursor(&mut self, [x, y]: [usize; 2]) -> io::Result<()> {
        if self.draw_time == DrawTime::Last {
            return Ok(());
        }

        queue!(self.out, cursor::RestorePosition)?;

        if y > 0 {
            queue!(self.out, cursor::MoveDown(y as u16))?;
        }

        if x > 0 {
            queue!(self.out, cursor::MoveRight(x as u16))?;
        }

        self.out.flush()
    }

    pub fn hide_cursor(&mut self) -> io::Result<()> {
        execute!(self.out, cursor::Hide)
    }

    pub fn show_cursor(&mut self) -> io::Result<()> {
        execute!(self.out, cursor::Show)
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
