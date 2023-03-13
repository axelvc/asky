use std::io;

use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

use super::renderer::Renderer;

pub trait KeyHandler {
    fn submit(&self) -> bool;
    fn draw<W: io::Write>(&self, renderer: &mut Renderer<W>) -> io::Result<()>;
    fn handle_key(&mut self, key: KeyEvent);
}

pub fn listen(handler: &mut impl KeyHandler) -> io::Result<()> {
    let mut renderer = Renderer::new();

    handler.draw(&mut renderer)?;
    renderer.update_draw_time();

    while !handler.submit() {
        terminal::enable_raw_mode()?;
        let k = read()?;
        terminal::disable_raw_mode()?;

        if let Event::Key(key) = k {
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
