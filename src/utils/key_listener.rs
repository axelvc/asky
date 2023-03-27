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
            if is_abort(key) {
                renderer.update_draw_time();
                handler.draw(&mut renderer)?;
                abort()?;
            }

            submit = handler.handle_key(key);
            handler.draw(&mut renderer)?;
        }
    }

    renderer.update_draw_time();
    handler.draw(&mut renderer)?;

    Ok(())
}

fn is_abort(ev: KeyEvent) -> bool {
    matches!(
        ev,
        KeyEvent {
            code: KeyCode::Esc,
            ..
        } | KeyEvent {
            code: KeyCode::Char('c' | 'd'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    )
}

fn abort() -> io::Result<()> {
    terminal::disable_raw_mode()?;
    std::process::exit(0)
}
