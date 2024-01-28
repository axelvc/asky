use crate::{utils::renderer::Renderer, DrawTime};
use std::fmt;
use std::io;
use text_style::{self, Color, Style, StyledString};

#[derive(Debug, Clone, Default)]
pub struct StyledStringWriter {
    pub style: Option<text_style::Style>,
    pub strings: Vec<StyledString>,
    pub state: RendererState,
    newline_count: u16,
}

impl StyledStringWriter {
    pub fn clear(&mut self) {
        self.state = RendererState::default();
    }
}

#[derive(Debug, Default, Clone)]
pub struct RendererState {
    pub(crate) draw_time: DrawTime,
    pub(crate) cursor_visible: bool,
    pub(crate) cursor_pos: [usize; 2],
}

impl std::io::Write for StyledStringWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = std::str::from_utf8(buf).expect("Not a utf8 string");
        let ss = match self.strings.pop() {
            None => StyledString::new(s.to_string(), self.style),
            Some(mut text) => {
                if text.style == self.style {
                    text.s.push_str(s);
                    text
                } else {
                    self.strings.push(text);
                    StyledString::new(s.to_string(), self.style)
                }
            }
        };
        self.strings.push(ss);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl std::fmt::Write for StyledStringWriter {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        // let s = std::str::from_utf8(buf).expect("Not a utf8 string");
        let ss = match self.strings.pop() {
            None => StyledString::new(s.to_string(), self.style),
            Some(mut text) => {
                if text.style == self.style {
                    text.s.push_str(s);
                    text
                } else {
                    self.strings.push(text);
                    StyledString::new(s.to_string(), self.style)
                }
            }
        };
        self.strings.push(ss);
        Ok(())
    }
}

impl Renderer for StyledStringWriter {
    fn draw_time(&self) -> DrawTime {
        self.state.draw_time
    }

    fn newline_count(&mut self) -> &mut u16 {
        &mut self.newline_count
    }

    fn update_draw_time(&mut self) {
        self.state.draw_time = match self.state.draw_time {
            DrawTime::First => DrawTime::Update,
            _ => DrawTime::Last,
        }
    }

    fn set_foreground(&mut self, color: Color) -> io::Result<()> {
        let style = self.style.get_or_insert(Style::default());
        style.fg = Some(color);
        Ok(())
    }

    fn set_background(&mut self, color: Color) -> io::Result<()> {
        let style = self.style.get_or_insert(Style::default());
        style.bg = Some(color);
        Ok(())
    }

    fn reset_color(&mut self) -> io::Result<()> {
        let style = self.style.get_or_insert(Style::default());
        style.fg = None;
        style.bg = None;
        Ok(())
    }

    fn pre_prompt(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn post_prompt(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Utility function for line input.
    /// Set initial position based on the position after drawing.
    fn move_cursor(&mut self, [x, y]: [usize; 2]) -> io::Result<()> {
        if self.state.draw_time == DrawTime::Last {
            return Ok(());
        }
        self.state.cursor_pos[0] += x;
        self.state.cursor_pos[1] += y;
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
