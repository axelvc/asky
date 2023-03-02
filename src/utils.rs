use std::io;

use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

pub fn is_abort(ev: KeyEvent) -> bool {
    match ev {
        KeyEvent {
            code: KeyCode::Esc, ..
        }
        | KeyEvent {
            code: KeyCode::Char('c' | 'd'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => true,
        _ => false,
    }
}

pub fn abort() -> io::Result<()> {
    terminal::disable_raw_mode()?;
    std::process::exit(0)
}
