use std::io;

use colored::Colorize;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::Print,
    terminal,
};

enum SelectedOption {
    Yes,
    No,
}

impl SelectedOption {
    fn to_string(&self) -> String {
        const YES: &str = " Yes ";
        const NO: &str = " No ";



        let (yes, no) = match self {
            SelectedOption::Yes => (
                SelectedOption::focused_colors(YES),
                SelectedOption::unfocused_colors(NO),
            ),
            SelectedOption::No => (
                SelectedOption::unfocused_colors(YES),
                SelectedOption::focused_colors(NO),
            ),
        };

        format!("{yes}  {no}")
    }

    fn focused_colors(s: &str) -> String {
        s.black().on_blue().to_string()
    }

    fn unfocused_colors(s: &str) -> String {
        s.white().on_bright_black().to_string()
    }
}

impl From<bool> for SelectedOption {
    fn from(value: bool) -> Self {
        match value {
            true => SelectedOption::Yes,
            false => SelectedOption::No,
        }
    }
}

pub struct Confirm<'a> {
    message: &'a str,
    value: bool,
    first_draw: bool,
}

impl Confirm<'_> {
    pub fn new(message: &str) -> Confirm {
        Confirm {
            message,
            value: false,
            first_draw: true,
        }
    }

    pub fn initial(&mut self, value: bool) -> &mut Self {
        self.value = value;
        self
    }

    pub fn prompt(&mut self) -> io::Result<bool> {
        self.draw(&mut io::stdout())?;

        match self.listen() {
            Ok(_) => Ok(self.value),
            Err(e) => {
                terminal::disable_raw_mode()?;
                Err(e)
            },
        }
    }

    fn draw<W: io::Write>(&mut self, out: &mut W) -> io::Result<()> {
        let options = SelectedOption::from(self.value);

        if !self.first_draw {
            queue!(out, cursor::MoveToPreviousLine(2))?;
        } else {
            self.first_draw = false;
        }

        queue!(
            out,
            Print(self.message),
            cursor::MoveToNextLine(1),
            Print(options.to_string()),
            cursor::MoveToNextLine(1),
        )?;

        out.flush()
    }

    fn listen(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;

        loop {
            if let Event::Key(key) = read()? {
                match key {
                    // confirm focused/initial
                    KeyEvent {
                        code: KeyCode::Enter | KeyCode::Backspace,
                        ..
                    } => break,
                    // yes
                    KeyEvent {
                        code: KeyCode::Char('y' | 'Y'),
                        ..
                    } => {
                        self.update_value(true)?;
                        break;
                    }
                    // no
                    KeyEvent {
                        code: KeyCode::Char('n' | 'N'),
                        ..
                    } => {
                        self.update_value(false)?;
                        break;
                    }
                    // focus yes
                    KeyEvent {
                        code: KeyCode::Left | KeyCode::Char('h' | 'H'),
                        ..
                    } => self.update_value(true)?,
                    // focus no
                    KeyEvent {
                        code: KeyCode::Right | KeyCode::Char('l' | 'L'),
                        ..
                    } => self.update_value(false)?,
                    // abort
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    }
                    | KeyEvent {
                        code: KeyCode::Char('c' | 'd'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    } => self.abort()?,
                    _ => (),
                }
            }
        }

        terminal::disable_raw_mode()
    }

    fn update_value(&mut self, value: bool) -> io::Result<()> {
        self.value = value;
        self.draw(&mut io::stdout())?;
        Ok(())
    }

    fn abort(&self) -> io::Result<()> {
        terminal::disable_raw_mode()?;
        std::process::exit(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_initial_value() {
        let mut confirm = Confirm::new("");

        assert!(!confirm.value);
        confirm.initial(true);
        assert!(confirm.value);
    }

    #[test]
    fn draw_right_colors() {
        let msg = "Do you like pizza?";

        let mut confirim = Confirm::new(msg);

        // first render
        let mut out = Vec::new();
        confirim.draw(&mut out).unwrap();

        let out = String::from_utf8(out).unwrap();
        assert!(out.contains(msg));
        assert!(out.contains(&SelectedOption::No.to_string()));

        // simulate update value
        let mut out = Vec::new();

        confirim.value = true;
        confirim.draw(&mut out).unwrap();

        let out = String::from_utf8(out).unwrap();
        assert!(out.contains(msg));
        assert!(out.contains(&SelectedOption::Yes.to_string()));
    }
}
