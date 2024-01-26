use std::io;
use std::fmt;
use text_style::{self, StyledString, Style, Color};
use crate::{DrawTime, utils::renderer::Renderer};

#[derive(Debug, Clone, Default)]
pub struct StyledStringWriter {
    pub style: Option<text_style::Style>,
    pub strings: Vec<StyledString>,
    pub state: RendererState,
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
                    StyledString::new(s.to_string(),
                                      self.style)
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
                    StyledString::new(s.to_string(),
                                      self.style)
                }
            }
        };
        self.strings.push(ss);
        Ok(())
    }
}

impl Renderer for StyledStringWriter {
    type Writer = StyledStringWriter;
    fn draw_time(&self) -> DrawTime {
        self.state.draw_time
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

    // fn print(&mut self, strings: ColoredStrings) -> io::Result<()> {
    fn print2<F>(&mut self, _draw_text: F) -> io::Result<()>
    where
        F: FnOnce(&mut Self::Writer) -> io::Result<u16> {
        Ok(())
    }

    fn print_prompt<F>(&mut self, draw_prompt: F) -> io::Result<()>
    where
        F: FnOnce(&mut Self) -> io::Result<u16> {
        let _text_lines = draw_prompt(self)? - 1;
        Ok(())
    }

    /// Utility function for line input.
    /// Set initial position based on the position after drawing.
    fn set_cursor(&mut self, [x, y]: [usize; 2]) -> io::Result<()> {
        if self.state.draw_time == DrawTime::Last {
            return Ok(());
        }
        self.state.cursor_pos[0] = x;
        self.state.cursor_pos[1] = y;
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
