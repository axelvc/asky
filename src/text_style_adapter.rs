use crate::{utils::renderer::Renderer, DrawTime};
use std::fmt;
use std::io;
use text_style::{Color, Style, StyledString};

#[derive(Debug, Clone, Default)]
pub struct StyledStringWriter {
    pub style: Option<text_style::Style>,
    pub strings: Vec<StyledString>,
    pub state: RendererState,
    pub(crate) cursor_pos: Option<CursorPos>,
    pub(crate) cursor_pos_save: Option<CursorPos>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CursorPos {
    index: usize,
    len: usize
}

impl StyledStringWriter {
    pub fn clear(&mut self) {
        self.state = RendererState::default();
        self.cursor_pos = None;
        self.cursor_pos_save = None;
    }

    fn get_cursor_pos(&mut self) -> CursorPos {
        if self.strings.len() == 0 {
            self.strings.push(StyledString::new(String::new(), self.style));
        }
        match &self.cursor_pos {
            None => CursorPos { index: self.strings.len() - 1, len: self.strings.last().unwrap().s.chars().count() },
            Some(c) => c.clone()
        }
    }

    fn set_cursor_pos(&mut self, cursor_pos: CursorPos) {
        // if cursor_pos.len < 0 {
        //     cursor_pos.index -= 1;
        //     cursor_pos.len += self.strings[cursor_pos.index].s.chars().count();
        // }
        self.cursor_pos = Some(cursor_pos);
    }

    pub(crate) fn drain_with_styled_cursor(&mut self, color: text_style::Color) -> Vec<StyledString> {
        let cursor_pos = self.get_cursor_pos();
        let mut strings = std::mem::take(&mut self.strings);
        let styled_string = std::mem::replace(&mut strings[cursor_pos.index], StyledString::new(String::new(), None));

        // eprintln!("cursor {:?} str len {}", cursor_pos, styled_string.s.len());
        let _ = strings.splice(cursor_pos.index..cursor_pos.index + 1, cursorify(styled_string, cursor_pos.len, color));
        strings
    }
}

// fn no_cursorify(
//     cs: StyledString,
//     i: usize,
//     cursor_color: text_style::Color,
// ) -> impl Iterator<Item = StyledString> {
//     std::iter::once(cs)
// }

/// Splits StyledString into possibly three pieces: (left string portion, the
/// cursor, right string portion). The character index `i`'s range is not the
/// usual _[0, N)_ where _N_ is the character count; it is _[0,N]_ inclusive so
/// that a cursor may be specified essentially at the end of the strin g.
fn cursorify(
    cs: StyledString,
    i: usize,
    cursor_color: text_style::Color,
) -> impl Iterator<Item = StyledString> {
    let (string, style) = (cs.s, cs.style);
    assert!(i <= string.chars().count(),
            "i {} <= str.chars().count() {}", i, string.chars().count());
    let (mut input, right) = match string.char_indices().nth(i + 1) {
        Some((byte_index, _char)) => {
            let (l, r) = string.split_at(byte_index);
            (l.to_owned(),Some(StyledString::new(r.to_owned(), style)))
        },
        None => {
            let mut s = string;
            if s.chars().count() == i {
                s.push(' ');
            }
            (s, None)
        }
    };
    let cursor = Some(
        StyledString::new(
            input
                .pop()
                // Newline is not printed. So use a space if necessary.
                // .map(|c| if c == '\n' { ' ' } else { c })
                // .unwrap()//_or(' ')
                .expect("Could not get cursor position")
                .to_string(),
            style
        )
        .on(cursor_color)
    );
    let left = Some(StyledString::new(input, style));
    left.into_iter().chain(cursor.into_iter().chain(right))
}

#[derive(Debug, Default, Clone)]
pub struct RendererState {
    pub(crate) draw_time: DrawTime,
    pub(crate) cursor_visible: bool,
    pub(crate) newline_count: u16,
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
        &mut self.state.newline_count
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
    fn move_cursor(&mut self, [x, _y]: [usize; 2]) -> io::Result<()> {
        if self.state.draw_time == DrawTime::Last {
            return Ok(());
        }
        let mut c = self.get_cursor_pos();
        c.len += x;
        self.set_cursor_pos(c);

        // self.state.cursor_pos[0] += x;
        // self.state.cursor_pos[1] += y;
        Ok(())
    }

    fn save_cursor(&mut self) -> io::Result<()> {
        self.cursor_pos_save = Some(self.get_cursor_pos());
        Ok(())
    }

    fn restore_cursor(&mut self) -> io::Result<()> {
        self.cursor_pos = self.cursor_pos_save.clone();
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

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Write;
    use text_style::{self, AnsiColor, StyledString};
    #[test]
    fn test_cursorify() {
        let mut w = StyledStringWriter::default();
        let v = w.drain_with_styled_cursor(AnsiColor::White.dark());
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn test_cursorify2() -> std::io::Result<()> {
        let mut w = StyledStringWriter::default();
        write!(w, "what the fuck")?;
        w.set_foreground(AnsiColor::Black.light())?;
        write!(w, "huh")?;
        let v = w.drain_with_styled_cursor(AnsiColor::White.dark());
        assert_eq!(v.len(), 3);
        Ok(())
    }

    #[test]
    fn test_cursorify3() {
        let s = StyledString::new(" ".into(), None);
        let v: Vec<_> = cursorify(s, 0, AnsiColor::White.dark()).collect();
        assert_eq!(v.len(), 2);
        assert_eq!(&v[0].s, "");
        assert_eq!(v[0].style, None);
        assert_eq!(&v[1].s, " ");
        assert_ne!(v[1].style, None);
    }

    #[test]
    fn test_cursorify5() {
        let s = StyledString::new("a".into(), None);
        let v: Vec<_> = cursorify(s, 1, AnsiColor::White.dark()).collect();
        assert_eq!(v.len(), 2);
        assert_eq!(&v[0].s, "a");
        assert_eq!(v[0].style, None);
        assert_eq!(&v[1].s, " ");
        assert_ne!(v[1].style, None);
    }

    #[test]
    fn test_cursorify4() {
        let s = StyledString::new("".into(), None);
        let v: Vec<_> = cursorify(s, 0, AnsiColor::White.dark()).collect();
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].style, None);
        assert_ne!(v[1].style, None);
    }

    mod unicode {
        use super::*;
        use std::io::Write;
        use text_style::{self, AnsiColor, StyledString};
        #[test]
        fn test_cursorify() {
            let mut w = StyledStringWriter::default();
            let v = w.drain_with_styled_cursor(AnsiColor::White.dark());
            assert_eq!(v.len(), 2);
        }

        #[test]
        fn test_cursorify2() -> std::io::Result<()> {
            let mut w = StyledStringWriter::default();
            write!(w, "▣what the fuck")?;
            w.set_foreground(AnsiColor::Black.light())?;
            write!(w, "▣huh")?;
            let v = w.drain_with_styled_cursor(AnsiColor::White.dark());
            assert_eq!(v.len(), 3);
            Ok(())
        }

        #[test]
        fn test_cursorify3() {
            let s = StyledString::new("▣".into(), None);
            let v: Vec<_> = cursorify(s, 0, AnsiColor::White.dark()).collect();
            assert_eq!(v.len(), 2);
            assert_eq!(&v[0].s, "");
            assert_eq!(v[0].style, None);
            assert_eq!(&v[1].s, "▣");
            assert_ne!(v[1].style, None);
        }

        #[test]
        fn test_unicode_cursorify5() {
            let s = StyledString::new("▣".into(), None);
            let v: Vec<StyledString> = cursorify(s, 1, AnsiColor::White.dark()).collect();
            assert_eq!(v.len(), 2);
            // assert_eq!(v[0].s.len(), 0);
            // assert_eq!(&v[0].s, "");
            assert_eq!(&v[0].s, "▣");
            assert_eq!(v[0].style, None);
            assert_eq!(&v[1].s, " ");
            assert_ne!(v[1].style, None);
        }

        #[test]
        fn test_cursorify4() {
            let s = StyledString::new("".into(), None);
            let v: Vec<_> = cursorify(s, 0, AnsiColor::White.dark()).collect();
            assert_eq!(v.len(), 2);
            assert_eq!(v[0].style, None);
            assert_ne!(v[1].style, None);
        }
    }
}

