use crate::style::{DefaultStyle, Style, WithFormat, WithStyle};
use std::io;
use text_style::Color;

pub trait Printable {
    fn draw_with_style<R: Renderer>(&self, renderer: &mut R, style: &dyn Style)
        -> io::Result<()>;

    fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
        let style = DefaultStyle::default();
        self.draw_with_style(renderer, &style)
    }

    fn style<S: Style>(self, style: S) -> WithStyle<Self, S>
    where
        Self: Sized,
    {
        WithStyle(self, style)
    }

    fn format<F: Fn(&Self, &mut dyn Renderer) -> io::Result<()>>(
        self,
        format: F,
    ) -> WithFormat<Self, F>
    where
        Self: Sized,
    {
        WithFormat(self, format)
    }
}

impl<T, F> Printable for WithFormat<T, F>
where
    F: Fn(&T, &mut dyn Renderer) -> io::Result<()>,
    T: Printable,
{
    fn draw_with_style<R: Renderer>(
        &self,
        renderer: &mut R,
        _style: &dyn Style,
    ) -> io::Result<()> {
        (self.1)(&self.0, renderer)
    }
}

impl<T, S> Printable for WithStyle<T, S>
where
    T: Printable,
    S: Style,
{

    fn draw_with_style<R: Renderer>(
        &self,
        renderer: &mut R,
        _style: &dyn Style,
    ) -> io::Result<()> {
        self.0.draw_with_style(renderer, &self.1)
    }
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

pub trait Renderer: io::Write {
    fn newline_count(&mut self) -> &mut u16;
    fn draw_time(&self) -> DrawTime;
    fn update_draw_time(&mut self);
    fn set_foreground(&mut self, color: Color) -> io::Result<()>;
    fn set_background(&mut self, color: Color) -> io::Result<()>;
    fn reset_color(&mut self) -> io::Result<()>;
    fn pre_prompt(&mut self) -> io::Result<()>;
    fn post_prompt(&mut self) -> io::Result<()>;

    // fn print2<F>(&mut self, draw_prompt: F) -> io::Result<()>
    // where
    //     F: FnOnce(&mut Self::Writer) -> io::Result<u16>;
    // fn print_prompt<F>(&mut self, draw_prompt: F) -> io::Result<()>
    // where
    //     F: FnOnce(&mut Self) -> io::Result<u16>;
    // fn print(&mut self, text: ColoredStrings) -> io::Result<()>;
    fn move_cursor(&mut self, directions: [usize; 2]) -> io::Result<()>;
    // fn move_cursor(&mut self, direction: [usize; 2]) -> io::Result<()> { Ok(()) }
    fn save_cursor(&mut self) -> io::Result<()>;
    fn restore_cursor(&mut self) -> io::Result<()>;
    fn hide_cursor(&mut self) -> io::Result<()>;
    fn show_cursor(&mut self) -> io::Result<()>;
}

#[derive(Clone, Default)]
pub struct StringRenderer {
    pub string: String,
    pub draw_time: DrawTime,
    pub line_count: u16,
}

pub(crate) fn count_newlines(input: &str) -> u16 {
    input.chars().filter(|&c| c == '\n').count() as u16
}

impl std::io::Write for StringRenderer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = std::str::from_utf8(buf).expect("Not a utf8 string");
        *self.newline_count() += count_newlines(s);
        self.string.push_str(s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl std::fmt::Write for StringRenderer {
    fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
        *self.newline_count() += count_newlines(s);
        self.string.push_str(s);
        Ok(())
    }
}

impl Renderer for StringRenderer {
    fn newline_count(&mut self) -> &mut u16 {
        &mut self.line_count
    }

    fn draw_time(&self) -> DrawTime {
        self.draw_time
    }

    fn update_draw_time(&mut self) {
        self.draw_time = match self.draw_time {
            DrawTime::First => DrawTime::Update,
            _ => DrawTime::Last,
        }
    }

    fn save_cursor(&mut self) -> io::Result<()> { Ok(()) }

    fn restore_cursor(&mut self) -> io::Result<()> { Ok(()) }

    fn set_foreground(&mut self, _color: Color) -> io::Result<()> {
        Ok(())
    }

    fn set_background(&mut self, _color: Color) -> io::Result<()> {
        Ok(())
    }

    fn reset_color(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn pre_prompt(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn post_prompt(&mut self) -> io::Result<()> {
        Ok(())
    }

    // fn print_prompt<F>(&mut self, draw_prompt: F) -> io::Result<()>
    // where
    //     F: FnOnce(&mut Self) -> io::Result<u16> {
    //     let _text_lines = draw_prompt(self)? - 1;
    //     Ok(())
    // }

    /// Utility function for line input.
    /// Set initial position based on the position after drawing.
    fn move_cursor(&mut self, _pos: [usize; 2]) -> io::Result<()> {
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        Ok(())
    }
}
