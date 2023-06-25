use crate::utils::renderer::{Printable, Renderer};
use crate::DrawTime;
use crate::Typeable;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
use colored::{Color as Colored, Color::TrueColor, ColoredString, ColoredStrings, Colorize};
use std::io;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::{Confirm, MultiSelect, Number, Password, Select, Toggle};

#[derive(Component, Debug)]
// pub struct Asky<T: Printable + for<'a> Typeable<KeyEvent<'a>>>(pub T);
pub struct Asky<T: Printable + Typeable<KeyEvent>>(pub T, pub AskyState);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AskyState {
    #[default]
    Reading,
    Complete,
    Hidden,
}

// impl<'a, T: Printable + Typeable<KeyEvent<'a>>> Asky<'a,T> {
//     fn new(x: T) -> Self {
//         Asky(x, PhantomData)
//     }
// }

impl<T: Printable + Typeable<KeyEvent>> Deref for Asky<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Printable + Typeable<KeyEvent>> DerefMut for Asky<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

pub struct KeyEvent {
    pub chars: Vec<char>,
    pub codes: Vec<KeyCode>,
}

impl<T: Typeable<KeyCode>> Typeable<KeyEvent> for T {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        let mut result = false;
        for code in &key.codes {
            result |= self.handle_key(code);
        }
        return result;
    }

    fn will_handle_key(&self, key: &KeyEvent) -> bool {
        let mut result = false;
        for code in &key.codes {
            result |= self.will_handle_key(code);
        }
        return result;
    }
}

impl KeyEvent {
    pub fn new(
        mut char_evr: EventReader<ReceivedCharacter>,
        // keys: &'w Res<'w, Input<KeyCode>>,
        mut key_evr: EventReader<KeyboardInput>,
    ) -> Self {
        Self {
            chars: char_evr.iter().map(|e| e.char).collect(),
            // keys,
            codes: key_evr
                .iter()
                .filter_map(|e| {
                    if e.state == bevy::input::ButtonState::Pressed {
                        e.key_code
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

#[derive(Resource, Debug)]
pub struct BevyAskySettings {
    pub style: TextStyle,
}

fn convert(c: Colored) -> Color {
    match c {
        Colored::Black => Color::BLACK,
        Colored::Red => Color::rgb_u8(204, 0, 0),
        Colored::Green => Color::rgb_u8(78, 154, 6),
        Colored::Yellow => Color::rgb_u8(196, 160, 0),
        Colored::Blue => Color::rgb_u8(114, 159, 207),
        Colored::Magenta => Color::rgb_u8(117, 80, 123),
        Colored::Cyan => Color::rgb_u8(6, 152, 154),
        Colored::White => Color::rgb_u8(211, 215, 207),
        Colored::BrightBlack => Color::rgb_u8(85, 87, 83),
        Colored::BrightRed => Color::rgb_u8(239, 41, 41),
        Colored::BrightGreen => Color::rgb_u8(138, 226, 52),
        Colored::BrightYellow => Color::rgb_u8(252, 233, 79),
        Colored::BrightBlue => Color::rgb_u8(50, 175, 255),
        Colored::BrightMagenta => Color::rgb_u8(173, 127, 168),
        Colored::BrightCyan => Color::rgb_u8(52, 226, 226),
        Colored::BrightWhite => Color::rgb_u8(255, 255, 255),
        Colored::TrueColor { r, g, b } => Color::rgb_u8(r, g, b),
    }
}

#[derive(Debug, Default)]
pub struct BevyRendererState {
    pub(crate) draw_time: DrawTime,
    cursor_visible: bool,
    cursor_pos: [usize; 2],
}

impl BevyRendererState {
    pub fn clear(&mut self) {
        self.draw_time = DrawTime::First;
        self.cursor_visible = true;
        self.cursor_pos[0] = 0;
        self.cursor_pos[1] = 0;
    }
}

// #[derive(Debug)]
pub struct BevyRenderer<'a, 'w, 's> {
    state: &'a mut BevyRendererState,
    settings: &'a BevyAskySettings,
    pub children: Vec<TextBundle>,
    commands: &'a mut Commands<'w, 's>,
    column: Entity,
}

impl<'a, 'w, 's> BevyRenderer<'a, 'w, 's> {
    pub fn new(
        settings: &'a BevyAskySettings,
        state: &'a mut BevyRendererState,
        commands: &'a mut Commands<'w, 's>,
        column: Entity,
    ) -> Self {
        BevyRenderer {
            settings,
            state,
            children: Vec::new(),
            commands,
            column,
        }
    }

    // pub fn to_text(&mut self, strings: ColoredStrings) {
    //     self.text.sections.clear();
    //     for s in strings.0.iter() {
    //         let mut style = self.settings.style.clone();
    //         if let Some(fg) = s.fgcolor() {
    //             style.color = convert(fg);
    //         }
    //         self.text.sections.push(TextSection::new(s.input.to_owned(), style));
    //     }
    // }

    pub fn build_text_bundle(s: ColoredString, mut style: TextStyle) -> TextBundle {
        if let Some(fg) = s.fgcolor() {
            style.color = convert(fg);
        }
        let mut bundle = TextBundle::from_section(format!("{}", s), style);
        if let Some(bg) = s.bgcolor() {
            bundle.background_color = BackgroundColor(convert(bg));
        }
        bundle
    }

    fn cursorify(cs: ColoredString, i: usize, cursor_color: colored::Color) -> impl Iterator<Item = ColoredString> {
        let to_colored_string = |s: String| -> ColoredString {
            let mut c = cs.clone();
            c.input = s.into();
            c
        };
        let mut input = cs.input.to_string();
        let mut left = None;
        let mut cursor = None;
        let mut right = None;
        if let Some((byte_index, _)) = input.char_indices().nth(i + 1) {
            // let (l, r) = input.split_at(i + 1);
            let (l, r) = input.split_at(byte_index);
            right = Some(to_colored_string(r.to_owned()));
            input = l.to_owned();
        }
        cursor = Some(to_colored_string(input.pop().expect("Could not get cursor").to_string()).on_color(cursor_color));
        left = Some(to_colored_string(input));
        left.into_iter().chain(cursor.into_iter().chain(right.into_iter()))
    }

    // fn split(strings: ColoredStrings, pat: char) -> impl Iterator<Item = ColoredStrings> {

    // }
}

impl<'a, 'w, 's> Renderer for BevyRenderer<'a, 'w, 's> {
    fn draw_time(&self) -> DrawTime {
        self.state.draw_time
    }

    fn update_draw_time(&mut self) {
        self.state.draw_time = match self.state.draw_time {
            DrawTime::First => DrawTime::Update,
            _ => DrawTime::Last,
        }
    }

    fn print(&mut self, strings: ColoredStrings) -> io::Result<()> {
        let style = self.settings.style.clone();
        self.commands.entity(self.column).with_children(|column| {
            for lines in strings.split('\n').into_iter().enumerate().map(|(i, mut colored_line)| {
                if self.state.cursor_visible && i == self.state.cursor_pos[1] {
                    let mut length = 0;
                    let mut inserted = false;
                    for i in 0..colored_line.len() {
                        if self.state.cursor_pos[0] < length + colored_line[i].input.chars().count() {
                            // The cursor is in this one.
                            let part = colored_line.remove(i);
                            for (j, new_part) in BevyRenderer::cursorify(part, self.state.cursor_pos[0] - length, colored::Color::White).enumerate() {
                                colored_line.insert(i + j, new_part)
                            }
                            inserted = true;
                            break;

                        }
                        length += colored_line[i].input.chars().count();
                    }
                    if !inserted && self.state.cursor_pos[0] >= length {
                        // Cursor is actually one character past string.
                        colored_line.push(" ".on_color(colored::Color::White));
                    }
                }
                colored_line
                    .0
                    .into_iter()
                    .map(|cs| BevyRenderer::build_text_bundle(cs, style.clone()))
            }) {
                column
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        for line in lines {
                            parent.spawn(line);
                        }
                    });
            }
        });
        Ok(())
    }

    /// Utility function for line input
    /// Set initial position based on the position after drawing
    fn set_cursor(&mut self, [x, y]: [usize; 2]) -> io::Result<()> {
        if self.state.draw_time == DrawTime::Last {
            return Ok(());
        }
        self.state.cursor_pos[0] = x;
        self.state.cursor_pos[1] = y;
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.state.cursor_visible = false;
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.state.cursor_visible = true;
        Ok(())
    }
}

pub fn asky_system<T: Printable + Typeable<KeyEvent> + Send + Sync + 'static>(
    mut commands: Commands,
    char_evr: EventReader<ReceivedCharacter>,
    mut key_evr: EventReader<KeyboardInput>,
    asky_settings: Res<BevyAskySettings>,
    mut render_state: Local<BevyRendererState>,
    // mut query: Query<&mut Text, With<Confirm>>) { // Compiler goes broke on this line.
    mut query: Query<(Entity, &mut Asky<T>, Option<&Children>)>,
) {
    let key_event = KeyEvent::new(char_evr, key_evr);

    // let must_mutate = query.iter().filter(|(e, c, _)| c.1 != AskyState::Complete && c.will_handle_key(&key));

    for (entity, mut confirm, children) in query.iter_mut() {
        match confirm.1 {
            AskyState::Complete => {
                continue;
            },
            AskyState::Hidden => {
                if let Some(children) = children {
                    let children: Vec<Entity> = children.to_vec();
                    commands.entity(entity).remove_children(&children);
                    for child in children {
                        commands.entity(child).despawn_recursive();
                    }
                }
            },
            AskyState::Reading => {
                if ! confirm.will_handle_key(&key_event)
                    && render_state.draw_time != DrawTime::First {
                    continue;
                }
                if confirm.handle_key(&key_event) {
                    // It's done.
                    confirm.1 = AskyState::Complete;
                    render_state.draw_time = DrawTime::Last;
                }
                if let Some(children) = children {
                    let children: Vec<Entity> = children.to_vec();
                    commands.entity(entity).remove_children(&children);
                    for child in children {
                        commands.entity(child).despawn_recursive();
                    }
                }
                let mut renderer =
                    BevyRenderer::new(&asky_settings, &mut render_state, &mut commands, entity);
                let _ = renderer.show_cursor();
                let draw_time = renderer.draw_time();
                confirm.draw(&mut renderer);
                eprint!(".");
                if draw_time == DrawTime::First {
                    renderer.update_draw_time();
                } else if draw_time == DrawTime::Last {
                    render_state.clear();
                }
            }
        }
    }
}

pub struct AskyPlugin;

impl Plugin for AskyPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_system(asky_system::<Confirm>)
        .add_system(asky_system::<Toggle>)
        .add_system(asky_system::<crate::Text>)
        .add_system(asky_system::<Number<u8>>)
        .add_system(asky_system::<Number<f32>>)
        .add_system(asky_system::<Select<'static, &'static str>>)
        .add_system(asky_system::<Password>)
        .add_system(asky_system::<MultiSelect<'static, &'static str>>);
    }
}
