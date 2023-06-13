use bevy::prelude::*;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use crate::DrawTime;
use crate::utils::renderer::Renderer;
use std::io;

pub(crate) struct KeyEvent<'w, 's> {
    pub(crate) char_evr: EventReader<'w, 's, ReceivedCharacter>,
    pub(crate) keys: Res<'w, Input<KeyCode>>,
    pub(crate) key_evr: EventReader<'w, 's, KeyboardInput>
}

impl<'w, 's> KeyEvent<'w, 's> {

    pub(crate) fn codes(&mut self) -> impl Iterator<Item = KeyCode> + '_ {
        self.key_evr.iter().filter_map(|e| if e.state == ButtonState::Pressed {
            e.key_code
        } else {
            None
        })

    }
}

pub struct BevyRenderer {
    draw_time: DrawTime,
    cursor_visible: bool,
    cursor_pos: [usize; 2],
    // out: io::Stdout,
}

impl BevyRenderer {
    pub fn new() -> Self {
        BevyRenderer {
            draw_time: DrawTime::First,
            cursor_visible: false,
            cursor_pos: [0, 0]
        }
    }
}

impl Renderer for BevyRenderer {

    fn draw_time(&self) -> DrawTime {
        self.draw_time
    }

    fn update_draw_time(&mut self) {
        self.draw_time = match self.draw_time {
            DrawTime::First => DrawTime::Update,
            _ => DrawTime::Last,
        }
    }

    fn print(&mut self, mut text: String) -> io::Result<()> {
        // if self.draw_time != DrawTime::First {
        //     queue!(
        //         self.out,
        //         cursor::RestorePosition,
        //         terminal::Clear(terminal::ClearType::FromCursorDown),
        //     )?;
        // }

        // if !text.ends_with('\n') {
        //     text.push('\n')
        // }

        // queue!(self.out, Print(&text))?;

        // // Saved position is updated each draw because the text lines could be different
        // // between draws. The last draw is ignored to always set the cursor at the end
        // //
        // // The position is saved this way to ensure the correct position when the cursor is at
        // // the bottom of the terminal. Otherwise, the saved position will be the last row
        // // and when trying to restore, the next draw will be below the last row.
        // if self.draw_time != DrawTime::Last {
        //     let (col, row) = cursor::position()?;
        //     let text_lines = text.lines().count() as u16;

        //     queue!(
        //         self.out,
        //         cursor::MoveToPreviousLine(text_lines),
        //         cursor::SavePosition,
        //         cursor::MoveTo(col, row)
        //     )?;
        // }

        // self.out.flush()
        Ok(())
    }

    /// Utility function for line input
    /// Set initial position based on the position after drawing
    fn set_cursor(&mut self, [x, y]: [usize; 2]) -> io::Result<()> {
        if self.draw_time == DrawTime::Last {
            return Ok(());
        }
        self.cursor_pos[0] = x;
        self.cursor_pos[0] = y;
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.cursor_visible = false;
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.cursor_visible = true;
        Ok(())
    }
}

#[cfg(feature="terminal")]
impl Default for TermRenderer {
    fn default() -> Self {
        Self::new()
    }
}
