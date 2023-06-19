use bevy::prelude::*;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use crate::DrawTime;
use crate::Typeable;
use crate::utils::renderer::{Renderer, Printable};
use std::io;
use colored::{Colorize, ColoredString, ColoredStrings, Color as Colored, Color::TrueColor};
use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

#[derive(Component, Debug)]
// pub struct Asky<T: Printable + for<'a> Typeable<KeyEvent<'a>>>(pub T);
pub struct Asky<T: Printable + Typeable<KeyEvent>>(pub T);

// impl<'a, T: Printable + Typeable<KeyEvent<'a>>> Asky<'a,T> {
//     fn new(x: T) -> Self {
//         Asky(x, PhantomData)
//     }
// }

impl<T: Printable + Typeable<KeyEvent>> Deref for Asky<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Printable + Typeable<KeyEvent>> DerefMut for Asky<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

pub struct KeyEvent{
    pub chars: Vec<char>,
    // pub keys: &'w Res<'w, Input<KeyCode>>,
    pub codes: Vec<KeyCode>,
}

impl<T: Typeable<KeyCode>> Typeable<KeyEvent> for T {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        let mut result = false;
        for code in &key.codes {
            result |= self.handle_key(code);
        }
        return result;
    }
}

impl KeyEvent {
    pub fn new(
        mut char_evr: EventReader<ReceivedCharacter>,
        // keys: &'w Res<'w, Input<KeyCode>>,
        mut key_evr: EventReader<KeyboardInput>,
    ) -> Self {
        Self {
            chars: char_evr.iter().map(|e| e.char).collect(),
            // keys,
            codes: key_evr.iter().filter_map(|e| if e.state == bevy::input::ButtonState::Pressed {
                e.key_code
            } else {
                None
            }).collect(),
        }
    }
}

#[derive(Resource, Debug)]
pub struct BevyAskySettings {
    pub style: TextStyle,
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


#[derive(Debug, Default)]
pub struct BevyRendererState {
    draw_time: DrawTime,
    cursor_visible: bool,
    cursor_pos: [usize; 2],
}

impl BevyRendererState {
    pub fn clear(&mut self) {
        self.draw_time = DrawTime::First;
        self.cursor_visible = true;
        self.cursor_pos[0] = 0;
        self.cursor_pos[1] = 0;
    }
}

#[derive(Debug)]
pub struct BevyRenderer<'a> {
    state: &'a mut BevyRendererState,
    text: &'a mut Text,
    settings: &'a BevyAskySettings,
    pub children: Vec<TextBundle>,
}

impl<'a> BevyRenderer<'a> {
    pub fn new(settings: &'a BevyAskySettings, state: &'a mut BevyRendererState, text: &'a mut Text) -> Self {
        BevyRenderer {
            settings,
            state,
            text,
            children: Vec::new()
        }
    }

    pub fn to_text(&mut self, strings: ColoredStrings) {
        self.text.sections.clear();
        for s in strings.0.iter() {
            let mut style = self.settings.style.clone();
            if let Some(fg) = s.fgcolor() {
                style.color = convert(fg);
            }
            self.text.sections.push(TextSection::new(s.input.to_owned(), style));
        }
    }
    pub fn build_text_bundle(&self, s: ColoredString) -> TextBundle {
        let mut style = self.settings.style.clone();
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

impl<'a> Renderer for BevyRenderer<'a> {

    fn draw_time(&self) -> DrawTime {
        self.state.draw_time
    }

    fn update_draw_time(&mut self) {
        self.state.draw_time = match self.state.draw_time {
            DrawTime::First => DrawTime::Update,
            _ => DrawTime::Last,
        }
    }

    fn print(&mut self, strings: ColoredStrings) -> io::Result<()> {
        // for bundle in strings.0.into_iter().map(|s| self.build_text_bundle(s)).collect::<Vec<_>>() {
        //     self.children.push(bundle);
        // }
        self.children.extend(strings.0.into_iter().map(|s| self.build_text_bundle(s)).collect::<Vec<_>>());
        // self.children.extend(bundles);

        // self.to_text(strings);
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
        if self.state.draw_time == DrawTime::Last {
            return Ok(());
        }
        self.state.cursor_pos[0] = x;
        self.state.cursor_pos[0] = y;
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.state.cursor_visible = false;
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.state.cursor_visible = true;
        Ok(())
    }
}

// impl Default for BevyRenderer {
//     fn default() -> Self {
//         Self::new()
//     }
// }
