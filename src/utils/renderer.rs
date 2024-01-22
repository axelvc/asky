// #[cfg(feature="terminal")]
use std::io::{self, Write, Stdout};

use crate::ColoredStrings;
pub trait Printable {
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

pub trait Renderer {
    type Writer: Write;
    fn draw_time(&self) -> DrawTime;
    fn update_draw_time(&mut self);

    fn print2<F>(&mut self, draw_text: F) -> io::Result<()> where F : FnOnce(&mut Self::Writer) -> io::Result<u16>;
    // fn print(&mut self, text: ColoredStrings) -> io::Result<()>;
    fn print(&mut self, text: ColoredStrings) -> io::Result<()>;
    fn set_cursor(&mut self, position: [usize; 2]) -> io::Result<()>;
    fn hide_cursor(&mut self) -> io::Result<()>;
    fn show_cursor(&mut self) -> io::Result<()>;
}
