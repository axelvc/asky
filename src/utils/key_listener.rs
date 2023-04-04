use std::io;

use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

use super::renderer::{Printable, Renderer};

/// Trait used for the prompts to handle key events
pub trait Typeable {
    /// Returns `true` if it should end to listen for more key events
    fn handle_key(&mut self, key: KeyEvent) -> bool;
}

/// Helper function to listen for key events and draw the prompt
pub fn listen(prompt: &mut (impl Printable + Typeable)) -> io::Result<()> {
    let mut renderer = Renderer::new();

    prompt.draw(&mut renderer)?;
    renderer.update_draw_time();

    let mut submit = false;

    while !submit {
        // raw mode to listen each key
        terminal::enable_raw_mode()?;
        let key = read()?;
        terminal::disable_raw_mode()?;

        if let Event::Key(key) = key {
            handle_abort(key);
            submit = prompt.handle_key(key);
            prompt.draw(&mut renderer)?;
        }
    }

    renderer.update_draw_time();
    prompt.draw(&mut renderer)
}

fn handle_abort(ev: KeyEvent) {
    let is_abort = matches!(
        ev,
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } | KeyEvent {
            code: KeyCode::Char('c' | 'd'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    );

    if is_abort {
        std::process::exit(1)
    }
}
