#[cfg(feature = "terminal")]
use std::io;

#[cfg(feature = "terminal")]
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

#[cfg(feature = "terminal")]
use super::renderer::{Printable, Renderer};

/// Trait used for the prompts to handle key events
pub trait Typeable<Key> {
    /// Returns `true` if it should end to listen for more key events
    fn handle_key(&mut self, key: &Key) -> bool;

    /// Returns `true` if this will handle a key. (Useful for avoiding mutable
    /// access to allow for change detection in some cases.)
    fn will_handle_key(&self, _key: &Key) -> bool {
        true
    }
}

#[cfg(feature = "terminal")]
/// Helper function to listen for key events and draw the prompt
pub fn listen(
    prompt: &mut (impl Printable + Typeable<KeyEvent>),
    hide_cursor: bool,
) -> io::Result<()> {
    let mut renderer = crate::terminal::TermRenderer::new();

    prompt.draw(&mut renderer)?;

    if prompt.hide_cursor() {
        renderer.hide_cursor()?;
    }

    renderer.update_draw_time();

    let mut submit = false;

    while !submit {
        // raw mode to listen each key
        terminal::enable_raw_mode()?;
        let key = read()?;
        terminal::disable_raw_mode()?;

        if let Event::Key(key) = key {
            handle_abort(key, &mut renderer);
            submit = prompt.handle_key(&key);
            // *renderer.newline_count() = 0;
            prompt.draw(&mut renderer)?;
        }
    }

    renderer.update_draw_time();

    if hide_cursor {
        renderer.show_cursor()?;
    }

    // *renderer.newline_count() = 0;
    prompt.draw(&mut renderer)
}

#[cfg(feature = "terminal")]
fn handle_abort<R: Renderer>(ev: KeyEvent, renderer: &mut R) {
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
        renderer.show_cursor().ok();
        std::process::exit(1)
    }
}
