use bevy::prelude::*;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;

pub(crate) struct KeyEvent<'w, 's> {
    pub(crate) char_evr: EventReader<'w, 's, ReceivedCharacter>,
    pub(crate) keys: Res<'w, Input<KeyCode>>,
    pub(crate) key_evr: EventReader<'w, 's, KeyboardInput>
}

impl<'w, 's> KeyEvent<'w, 's> {

    pub(crate) fn codes(&mut self) -> impl Iterator<Item = KeyCode> + '_ {
        self.key_evr.iter().filter_map(|e| if e.state == ButtonState::Pressed {
            e.key_code
        } else {
            None
        })

    }
}
