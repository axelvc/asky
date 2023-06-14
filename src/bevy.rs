use bevy::prelude::*;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use crate::DrawTime;
use crate::utils::renderer::Renderer;
use std::io;
use colored::{Colorize, ColoredString, ColoredStrings, Color as Colored, Color::TrueColor};

pub struct KeyEvent<'w> {
    pub chars: Vec<char>,
    pub keys: &'w Res<'w, Input<KeyCode>>,
    pub key_codes: Vec<KeyCode>,
}

impl<'w> KeyEvent<'w> {
    pub fn new(
        mut char_evr: EventReader<ReceivedCharacter>,
        keys: &'w Res<'w, Input<KeyCode>>,
        mut key_evr: EventReader<KeyboardInput>,
    ) -> Self {
        Self {
            chars: char_evr.iter().map(|e| e.char).collect(),
            keys,
            key_codes: key_evr.iter().filter_map(|e| if e.state == bevy::input::ButtonState::Pressed {
                e.key_code
            } else {
                None
            }).collect(),
        }
    }
}

#[derive(Debug, Default)]
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

#[derive(Resource)]
pub struct ColoredBuilder {
    pub style: TextStyle,
}

impl ColoredBuilder {

    pub fn to_text(&self, strings: ColoredStrings, out: &mut Text) {
        out.sections.clear();
        for s in strings.0.iter() {
            let mut style = self.style.clone();
            if let Some(fg) = s.fgcolor() {
                style.color = convert(fg);
            }
            out.sections.push(TextSection::new(s.input.to_owned(), style));
        }
    }
    pub fn build_text_bundle(&self, s: ColoredString) -> TextBundle {
        let mut style = self.style.clone();
        if let Some(fg) = s.fgcolor() {
            style.color = convert(fg);
        }
        let mut bundle = TextBundle::from_section(
                format!("{}", s),
                style);
        if let Some(bg) = s.bgcolor() {
            bundle.background_color = BackgroundColor(convert(bg));
        }
        bundle
    }
}

fn convert(c: Colored) -> Color {
    match c {
        Colored::Black => Color::BLACK,
        Colored::Red => Color::rgb_u8(204, 0, 0),
        Colored::Green => Color::rgb_u8(78, 154, 6),
        Colored::Yellow => Color::rgb_u8(196, 160, 0),
        Colored::Blue => Color::rgb_u8(114, 159, 207),
        Colored::Magenta => Color::rgb_u8(117, 80, 123),
        Colored::Cyan => Color::rgb_u8(6, 152, 154),
        Colored::White => Color::rgb_u8(211, 215, 207),
        Colored::BrightBlack => Color::rgb_u8(85, 87, 83),
        Colored::BrightRed => Color::rgb_u8(239, 41, 41),
        Colored::BrightGreen => Color::rgb_u8(138, 226, 52),
        Colored::BrightYellow => Color::rgb_u8(252, 233, 79),
        Colored::BrightBlue => Color::rgb_u8(50, 175, 255),
        Colored::BrightMagenta => Color::rgb_u8(173, 127, 168),
        Colored::BrightCyan => Color::rgb_u8(52, 226, 226),
        Colored::BrightWhite => Color::rgb_u8(255, 255, 255),
        Colored::TrueColor { r, g, b } => Color::rgb_u8(r, g, b),
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

    fn print(&mut self, mut text: ColoredStrings) -> io::Result<()> {
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

// impl Default for BevyRenderer {
//     fn default() -> Self {
//         Self::new()
//     }
// }
