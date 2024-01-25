use std::io;
use text_style::Color;

pub trait Printable {
    fn hide_cursor(&self) -> bool {
        true
    }
    fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()>;
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

pub trait Renderer : io::Write{
    type Writer: std::io::Write;
    fn draw_time(&self) -> DrawTime;
    fn update_draw_time(&mut self);
    fn set_foreground(&mut self, color: Color) -> io::Result<()>;
    fn set_background(&mut self, color: Color) -> io::Result<()>;
    fn reset_color(&mut self) -> io::Result<()>;

    fn print2<F>(&mut self, draw_prompt: F) -> io::Result<()>
    where
        F: FnOnce(&mut Self::Writer) -> io::Result<u16>;
    fn print_prompt<F>(&mut self, draw_prompt: F) -> io::Result<()>
    where
        F: FnOnce(&mut Self) -> io::Result<u16>;
    // fn print(&mut self, text: ColoredStrings) -> io::Result<()>;
    fn set_cursor(&mut self, position: [usize; 2]) -> io::Result<()>;
    fn hide_cursor(&mut self) -> io::Result<()>;
    fn show_cursor(&mut self) -> io::Result<()>;
}
