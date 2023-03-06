use std::io;

use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

use crate::renderer::Renderer;

pub trait KeyHandler {
    fn submit(&self) -> bool;
    fn draw<W: io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()>;
    fn handle_key(&mut self, key: KeyEvent);
}

pub fn listen(message: &str, handler: &mut impl KeyHandler) -> io::Result<()> {
    let mut renderer = Renderer::new(message);

    terminal::enable_raw_mode()?;
    handler.draw(&mut renderer)?;

    while !handler.submit() {
        if let Event::Key(key) = read()? {
            if is_abort(key) {
                renderer.update_draw_time();
                handler.draw(&mut renderer)?;
                abort()?;
            }

            handler.handle_key(key);
            handler.draw(&mut renderer)?;
        }
    }

    renderer.update_draw_time();
    handler.draw(&mut renderer)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn is_abort(ev: KeyEvent) -> bool {
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

fn abort() -> io::Result<()> {
    terminal::disable_raw_mode()?;
    std::process::exit(0)
}
