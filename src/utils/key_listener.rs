use std::io;

use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

use super::renderer::{Printable, Renderer};

pub trait Typeable {
    fn handle_key(&mut self, key: KeyEvent) -> bool;
}

pub fn listen(handler: &mut (impl Printable + Typeable)) -> io::Result<()> {
    let mut renderer = Renderer::new();

    handler.draw(&mut renderer)?;
    renderer.update_draw_time();
    let mut submit = false;

    while !submit {
        terminal::enable_raw_mode()?;
        let k = read()?;
        terminal::disable_raw_mode()?;

        if let Event::Key(key) = k {
            handle_abort(key);
            submit = handler.handle_key(key);
            handler.draw(&mut renderer)?;
        }
    }

    renderer.update_draw_time();
    handler.draw(&mut renderer)?;

    Ok(())
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
