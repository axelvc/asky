use crate::utils::num_like::NumLike;
use crate::{
    utils::renderer::{Printable, Renderer},
    ColoredStrings, Confirm, DrawTime, Message, MultiSelect, Number, Password, Select, Text,
    Toggle, Valuable,
};
use std::io::{self, Write};

use crate::prompts::text::Direction;
use crate::utils::key_listener::Typeable;
use crossterm::event::{KeyCode, KeyEvent};
use crossterm::{cursor, execute, queue, style::Print, terminal};
use crate::style::{DefaultStyle, Style, Section, Region};

pub struct TermRenderer {
    pub draw_time: DrawTime,
    out: io::Stdout,
}

impl TermRenderer {
    pub fn new() -> Self {
        TermRenderer {
            draw_time: DrawTime::First,
            out: io::stdout(),
        }
    }
}

impl Renderer for TermRenderer {
    type Writer = io::Stdout;
    fn draw_time(&self) -> DrawTime {
        self.draw_time
    }

    fn update_draw_time(&mut self) {
        self.draw_time = match self.draw_time {
            DrawTime::First => DrawTime::Update,
            _ => DrawTime::Last,
        }
    }

    fn print2<F>(&mut self, draw_text: F) -> io::Result<()> where F : FnOnce(&mut Self::Writer) -> io::Result<u16> {
        if self.draw_time != DrawTime::First {
            queue!(
                self.out,
                cursor::RestorePosition,
                terminal::Clear(terminal::ClearType::FromCursorDown),
            )?;
        }
        // XXX: try not to flush twice later.
        // self.out.flush()?;
        let text_lines = draw_text(&mut self.out)? - 1;
        // let mut text = format!("{}", text);

        // if !text.ends_with('\n') {
        //     text.push('\n')
        // }

        // queue!(self.out, Print(&text))?;

        // Saved position is updated each draw because the text lines could be different
        // between draws. The last draw is ignored to always set the cursor at the end
        //
        // The position is saved this way to ensure the correct position when the cursor is at
        // the bottom of the terminal. Otherwise, the saved position will be the last row
        // and when trying to restore, the next draw will be below the last row.
        if self.draw_time != DrawTime::Last {
            let (col, row) = cursor::position()?;
            // let text_lines = text.lines().count() as u16;

            queue!(
                self.out,
                cursor::MoveToPreviousLine(text_lines),
                cursor::SavePosition,
                cursor::MoveTo(col, row)
            )?;
        }

        self.out.flush()
    }

    fn print(&mut self, text: ColoredStrings) -> io::Result<()> {
        if self.draw_time != DrawTime::First {
            queue!(
                self.out,
                cursor::RestorePosition,
                terminal::Clear(terminal::ClearType::FromCursorDown),
            )?;
        }
        let mut text = format!("{}", text);

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
    fn set_cursor(&mut self, [x, y]: [usize; 2]) -> io::Result<()> {
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

    fn hide_cursor(&mut self) -> io::Result<()> {
        execute!(self.out, cursor::Hide)
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        execute!(self.out, cursor::Show)
    }
}

impl Default for TermRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> crate::Promptable for T
where
    T: Printable + Typeable<KeyEvent> + Valuable,
{
    type Output = T::Output;

    fn prompt(&mut self) -> Result<Self::Output, crate::Error> {
        crate::utils::key_listener::listen(self, true)?;
        self.value()
    }
}

// Text
impl Typeable<KeyEvent> for Text<'_> {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        use crate::prompts::text::Direction::*;
        let mut submit = false;

        match key.code {
            // submit
            KeyCode::Enter => submit = self.validate_to_submit(),
            // type
            KeyCode::Char(c) => self.input.insert(c),
            // remove delete
            KeyCode::Backspace => self.input.backspace(),
            KeyCode::Delete => self.input.delete(),
            // move cursor
            KeyCode::Left => self.input.move_cursor(Left),
            KeyCode::Right => self.input.move_cursor(Right),
            _ => (),
        };

        submit
    }
}

impl Printable for Text<'_> {
    fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
        let mut out = ColoredStrings::default();
        let cursor = (self.formatter)(self, renderer.draw_time(), &mut out);
        renderer.print(out)?;
        renderer.set_cursor(cursor)
    }
}

// Confirm
impl Typeable<KeyEvent> for Confirm<'_> {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        let mut submit = false;

        match key.code {
            // update value
            KeyCode::Left | KeyCode::Char('h' | 'H') => self.active = false,
            KeyCode::Right | KeyCode::Char('l' | 'L') => self.active = true,
            // update value and submit
            KeyCode::Char('y' | 'Y') => submit = self.update_and_submit(true),
            KeyCode::Char('n' | 'N') => submit = self.update_and_submit(false),
            // submit current/initial value
            KeyCode::Enter | KeyCode::Backspace => submit = true,
            _ => (),
        }

        submit
    }
}

// Number
impl<T: NumLike> Typeable<KeyEvent> for Number<'_, T> {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        let mut submit = false;

        match key.code {
            // submit
            KeyCode::Enter => submit = self.validate_to_submit(),
            // type
            KeyCode::Char(c) => self.insert(c),
            // remove delete
            KeyCode::Backspace => self.input.backspace(),
            KeyCode::Delete => self.input.delete(),
            // move cursor
            KeyCode::Left => self.input.move_cursor(Direction::Left),
            KeyCode::Right => self.input.move_cursor(Direction::Right),
            _ => (),
        }

        submit
    }
}

impl<T: NumLike> Printable for Number<'_, T> {
    fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
        // let mut out = ColoredStrings::default();
        // let cursor = (self.formatter)(self, renderer.draw_time(), &mut out);
        // renderer.print(out)?;
        // renderer.set_cursor(cursor)
        //
        use crate::style::Section::*;
        let style = DefaultStyle { ascii: true };
        let draw_time = renderer.draw_time();

        renderer.print2(|writer| {
            if draw_time == DrawTime::Last {

                queue!(writer,
                    style.begin(Query(true)),
                    Print(self.message.to_string()),
                    style.end(Query(true)),
                    style.begin(Answer),
                    Print(self.input.value.to_string()),
                    style.end(Answer),
                )?;
                Ok(1)
            } else {
                queue!(writer,
                    style.begin(Query(false)),
                    Print(self.message.to_string()),
                    style.end(Query(false)),
                )?;
                if let Some(x) = self.default_value {
                    queue!(writer,
                        style.begin(DefaultAnswer),
                        Print(self.message.to_string()),
                        style.end(DefaultAnswer),
                    )?;
                }
                let is_valid = self.validator_result.is_ok();
                queue!(writer,
                       style.begin(Validator(is_valid)),
                       style.end(Validator(is_valid)),
                       )?;
                if self.input.value.is_empty() {
                    if let Some(placeholder) = self.placeholder {
                        queue!(writer,
                                style.begin(Placeholder),
                                Print(placeholder),
                                style.end(Placeholder),
                            )?;
                    }
                } else {
                    queue!(writer,
                            style.begin(Input),
                            Print(self.input.value.to_string()),
                            style.end(Input),
                           )?;
                }
                Ok(1)
            }
        })
    }
}

// Select
impl<T> Typeable<KeyEvent> for Select<'_, T> {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        use crate::prompts::select::Direction;
        let mut submit = false;

        match key.code {
            // submit
            KeyCode::Enter | KeyCode::Backspace => submit = self.validate_to_submit(),
            // update value
            KeyCode::Up | KeyCode::Char('k' | 'K') => self.input.move_cursor(Direction::Up),
            KeyCode::Down | KeyCode::Char('j' | 'J') => self.input.move_cursor(Direction::Down),
            KeyCode::Left | KeyCode::Char('h' | 'H') => self.input.move_cursor(Direction::Left),
            KeyCode::Right | KeyCode::Char('l' | 'L') => self.input.move_cursor(Direction::Right),
            _ => (),
        }

        submit
    }
}

// MultiSelect
impl<T> Typeable<KeyEvent> for MultiSelect<'_, T> {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        use crate::prompts::select::Direction;
        let mut submit = false;

        match key.code {
            // submit
            KeyCode::Enter | KeyCode::Backspace => submit = self.validate_to_submit(),
            // select/unselect
            KeyCode::Char(' ') => self.toggle_focused(),
            // update focus
            KeyCode::Up | KeyCode::Char('k' | 'K') => self.input.move_cursor(Direction::Up),
            KeyCode::Down | KeyCode::Char('j' | 'J') => self.input.move_cursor(Direction::Down),
            KeyCode::Left | KeyCode::Char('h' | 'H') => self.input.move_cursor(Direction::Left),
            KeyCode::Right | KeyCode::Char('l' | 'L') => self.input.move_cursor(Direction::Right),
            _ => (),
        }

        submit
    }
}

// Password
impl Typeable<KeyEvent> for Password<'_> {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        let mut submit = false;

        match key.code {
            // submit
            KeyCode::Enter => submit = self.validate_to_submit(),
            // type
            KeyCode::Char(c) => self.input.insert(c),
            // remove delete
            KeyCode::Backspace => self.input.backspace(),
            KeyCode::Delete => self.input.delete(),
            // move cursor
            KeyCode::Left => self.input.move_cursor(Direction::Left),
            KeyCode::Right => self.input.move_cursor(Direction::Right),
            _ => (),
        };

        submit
    }
}

impl Printable for Password<'_> {
    fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
        let mut out = ColoredStrings::new();
        let cursor = (self.formatter)(self, renderer.draw_time(), &mut out);
        renderer.print(out)?;
        renderer.set_cursor(cursor)
    }
}

// Toggle
impl Typeable<KeyEvent> for Toggle<'_> {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        let mut submit = false;

        match key.code {
            // submit focused/initial option
            KeyCode::Enter | KeyCode::Backspace => submit = true,
            // update focus option
            KeyCode::Left | KeyCode::Char('h' | 'H') => self.active = false,
            KeyCode::Right | KeyCode::Char('l' | 'L') => self.active = true,
            _ => (),
        }

        submit
    }
}

impl Typeable<KeyEvent> for Message<'_> {
    fn handle_key(&mut self, _key: &KeyEvent) -> bool {
        true
    }
}
