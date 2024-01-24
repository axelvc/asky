use crate::utils::renderer::{Printable, Renderer};

use crate::Typeable;
use crate::{DrawTime, NumLike};
use bevy::{
    ecs::{
        component::Tick,
        system::{SystemMeta, SystemParam},
        world::unsafe_world_cell::UnsafeWorldCell,
    },
    input::keyboard::{KeyCode, KeyboardInput},
    utils::Duration,
};
use promise_out::{
    pair::{Consumer, Producer},
    Promise,
};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use std::future::Future;
use std::io;

use std::ops::{Deref, DerefMut};

use crate::{
    Confirm, Error, Message, MultiSelect, Number, Password, Select, Toggle,
    Valuable,
};
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use futures_lite::future;
use itertools::Itertools;
use text_style::{self, bevy::TextStyleParams, AnsiColor, AnsiMode, StyledString};
use crate::text_style_adapter::{StyledStringWriter};

#[derive(Component, Debug)]
pub struct AskyNode<T: Typeable<KeyEvent> + Valuable>(pub T, pub AskyState<T::Output>);

#[derive(Component, Debug)]
struct AskyDelay(Timer, Option<Producer<(), Error>>);

#[derive(Debug, Default)]
pub enum AskyState<T> {
    #[default]
    Reading,
    Waiting(Producer<T, Error>),
    Complete,
    Hidden,
}

fn run_timers(mut commands: Commands, mut query: Query<(Entity, &mut AskyDelay)>, time: Res<Time>) {
    for (id, mut asky_delay) in query.iter_mut() {
        asky_delay.0.tick(time.delta());
        if asky_delay.0.finished() {
            asky_delay.1.take().expect("Promise not there").resolve(());
            commands.entity(id).despawn();
        }
    }
}

pub struct Asky {
    config: AskyParamConfig,
}

#[derive(Resource, Clone)]
pub struct AskyParamConfig {
    pub(crate) state: Arc<Mutex<AskyParamState>>,
}

type Closure = dyn FnOnce(&mut Commands, Option<Entity>, Option<&Children>) -> Result<(), Error>
    + 'static
    + Send
    + Sync;

// #[derive(Debug)]
pub struct AskyParamState {
    pub(crate) closures: Vec<(Box<Closure>, Option<Entity>)>,
}

impl Asky {
    fn new(config: AskyParamConfig) -> Self {
        Self { config }
    }

    pub fn prompt<T: Typeable<KeyEvent> + Valuable + Send + Sync + 'static>(
        &mut self,
        prompt: T,
        dest: Entity,
    ) -> Consumer<T::Output, Error> {
        let (promise, waiter) = Producer::<T::Output, Error>::new();
        self.config.state.lock().unwrap().closures.push((
            Box::new(
                move |commands: &mut Commands,
                      entity: Option<Entity>,
                      _children: Option<&Children>| {
                    let node = NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        ..default()
                    };
                    let id = commands
                        .spawn((node, AskyNode(prompt, AskyState::Waiting(promise))))
                        .id();
                    commands.entity(entity.unwrap()).push_children(&[id]);
                    Ok(())
                },
            ),
            Some(dest),
        ));
        waiter
    }

    pub fn clear(&mut self, dest: Entity) -> Consumer<(), Error> {
        let (promise, waiter) = Producer::<(), Error>::new();
        self.config.state.lock().unwrap().closures.push((
            Box::new(
                move |commands: &mut Commands,
                      entity: Option<Entity>,
                      children_maybe: Option<&Children>| {
                    commands.entity(entity.unwrap()).clear_children();
                    if let Some(children) = children_maybe {
                        for child in children.iter() {
                            commands.entity(*child).despawn_recursive();
                        }
                    }
                    promise.resolve(());
                    Ok(())
                },
            ),
            Some(dest),
        ));
        waiter
    }

    pub fn delay(&mut self, duration: Duration) -> Consumer<(), Error> {
        let (promise, waiter) = Producer::<(), Error>::new();
        self.config.state.lock().unwrap().closures.push((
            Box::new(
                move |commands: &mut Commands,
                      _entity: Option<Entity>,
                      _children_maybe: Option<&Children>| {
                    commands.spawn(AskyDelay(
                        Timer::new(duration, TimerMode::Once),
                        Some(promise),
                    ));
                    Ok(())
                },
            ),
            None,
        ));
        waiter
    }
}

fn run_closures(
    config: ResMut<AskyParamConfig>,
    mut commands: Commands,
    query: Query<Option<&Children>>,
) {
    for (closure, id_maybe) in config
        .state
        .lock()
        .expect("Unable to lock mutex")
        .closures
        .drain(0..)
    {
        let children = id_maybe
            .map(|id| query.get(id).expect("Unable to get children"))
            .unwrap_or(None);
        let _ = closure(&mut commands, id_maybe, children);
    }
}

unsafe impl SystemParam for Asky {
    type State = AskyParamConfig;
    type Item<'w, 's> = Asky;

    fn init_state(world: &mut World, _system_meta: &mut SystemMeta) -> Self::State {
        world
            .get_resource_mut::<AskyParamConfig>()
            .expect("No AskyParamConfig setup.")
            .clone()
    }

    #[inline]
    unsafe fn get_param<'w, 's>(
        state: &'s mut Self::State,
        _system_meta: &SystemMeta,
        _world: UnsafeWorldCell<'w>,
        _change_tick: Tick,
    ) -> Self::Item<'w, 's> {
        Asky::new(state.clone())
    }
}

#[derive(Component)]
pub struct TaskSink<T>(pub Task<T>);

impl<T: Send + 'static> TaskSink<T> {
    pub fn new(future: impl Future<Output = T> + Send + 'static) -> Self {
        let thread_pool = AsyncComputeTaskPool::get();
        let task = thread_pool.spawn(future);
        Self(task)
    }
}

pub fn future_sink<T: Send + 'static, F: Future<Output = T> + Send + 'static>(
    In(future): In<F>,
    mut commands: Commands,
) {
    commands.spawn(TaskSink::new(future));
}

// pub fn future_result_sink<T: Send + 'static, F: Future<Output = Result<T, Error>> + Send + 'static>(
//     In(future): In<F>,
//     mut commands: Commands,
// ) {
//     commands.spawn(TaskSink::new(future));
// }

pub fn option_future_sink<T: Send + 'static, F: Future<Output = T> + Send + 'static>(
    In(future_maybe): In<Option<F>>,
    mut commands: Commands,
) {
    if let Some(future) = future_maybe {
        commands.spawn(TaskSink::new(future));
    }
}

pub fn poll_tasks<T: Send + Sync + 'static>(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut TaskSink<T>)>,
) {
    for (entity, mut task) in &mut tasks {
        if block_on(future::poll_once(&mut task.0)).is_some() {
            // Once
            commands.entity(entity).despawn();
        }
    }
}

pub fn poll_tasks_err<T: Send + Sync + 'static>(
    mut commands: Commands,
    _asky: Asky,
    mut tasks: Query<(Entity, &mut TaskSink<Result<T, Error>>)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.0)) {
            // Once
            if let Err(error) = result {
                eprintln!("Got here {:?}.", error);
                // FIXME: I need the right entity to make this work.
                // let _ = asky.prompt(Message::new(format!("{:?}", error)), entity);
                commands.entity(entity).despawn();
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}

impl<T: Typeable<KeyEvent> + Valuable> Deref for AskyNode<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: Typeable<KeyEvent> + Valuable> DerefMut for AskyNode<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

pub struct KeyEvent {
    pub chars: Vec<char>,
    pub codes: Vec<KeyCode>,
}

impl KeyEvent {
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty() && self.codes.is_empty()
    }
}

impl<T: Typeable<KeyCode>> Typeable<KeyEvent> for T {
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        let mut result = false;
        for code in &key.codes {
            result |= self.handle_key(code);
        }
        result
    }

    fn will_handle_key(&self, key: &KeyEvent) -> bool {
        for code in &key.codes {
            if self.will_handle_key(code) {
                return true;
            }
        }
        false
    }
}

impl KeyEvent {
    pub fn new(
        mut char_evr: EventReader<ReceivedCharacter>,
        // keys: &'w Res<'w, Input<KeyCode>>,
        mut key_evr: EventReader<KeyboardInput>,
    ) -> Self {
        Self {
            chars: char_evr.read().map(|e| e.char).collect(),
            // keys,
            codes: key_evr
                .read()
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
struct BevyRenderer<'a, 'w, 's> {
    state: &'a mut BevyRendererState,
    settings: &'a BevyAskySettings,
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
            commands,
            column,
        }
    }

    // pub fn build_text_bundle(s: ColoredString, mut style: TextStyle) -> TextBundle {
    //     if let Some(fg) = s.fgcolor() {
    //         style.color = convert(fg);
    //     }
    //     // return <str as fmt::Display>::fmt(&s.input, f);
    //     // Don't use format!("{}", s) or you could get ANSI escape sequences.
    //     let mut bundle = TextBundle::from_section(s.input.to_owned(), style);
    //     if let Some(bg) = s.bgcolor() {
    //         bundle.background_color = BackgroundColor(convert(bg));
    //     }
    //     bundle
    // }
}
fn cursorify(
    cs: StyledString,
    i: usize,
    cursor_color: text_style::Color,
) -> impl Iterator<Item = StyledString> {
    let to_colored_string = |s: String| -> StyledString { StyledString { s, ..cs.clone() } };
    let mut input = cs.s.to_string();
    let mut right = None;
    if let Some((byte_index, _)) = input.char_indices().nth(i + 1) {
        let (l, r) = input.split_at(byte_index);
        right = Some(to_colored_string(r.to_owned()));
        input = l.to_owned();
    }
    let cursor = Some(
        to_colored_string(
            input
                .pop()
                // Newline is not printed. So use a space if necessary.
                .map(|c| if c == '\n' { ' ' } else { c })
                .expect("Could not get cursor")
                .to_string(),
        )
        .on(cursor_color),
    );
    let left = Some(to_colored_string(input));
    left.into_iter()
        .chain(cursor.into_iter().chain(right.into_iter()))
}

fn cursorify_iter(
    iter: impl Iterator<Item = StyledString>,
    cursor_pos: usize,
    cursor_color: text_style::Color,
) -> impl Iterator<Item = StyledString> {
    let mut count = 0;
    iter.flat_map(move |ss| {
        let l = ss.s.chars().count();
        let has_index = cursor_pos < count + l && cursor_pos >= count;

        let mut a = None;
        let mut b = None;
        if has_index {
            a = Some(cursorify(ss, cursor_pos - count, cursor_color));
        } else {
            b = Some(ss);
        }

        count += l;
        a.into_iter().flatten().chain(b.into_iter())
    })
}

impl<'a, 'w, 's> Renderer for BevyRenderer<'a, 'w, 's> {
    type Writer = StyledStringWriter;
    fn draw_time(&self) -> DrawTime {
        self.state.draw_time
    }

    fn update_draw_time(&mut self) {
        self.state.draw_time = match self.state.draw_time {
            DrawTime::First => DrawTime::Update,
            _ => DrawTime::Last,
        }
    }

    fn print2<F>(&mut self, draw_text: F) -> io::Result<()>
    where
        F: FnOnce(&mut Self::Writer) -> io::Result<u16> {
        todo!();
    }

    // fn print(&mut self, strings: ColoredStrings) -> io::Result<()> {
    //     let white = text_style::Color::Ansi {
    //         color: AnsiColor::White,
    //         mode: AnsiMode::Dark,
    //     };

    //     self.commands.entity(self.column).with_children(|column| {
    //         let mut next_line_count: Option<usize> = None;
    //         let mut line_count: usize = 0;
    //         let lines = strings
    //             .0
    //             .into_iter()
    //             .map(StyledString::from)
    //             .flat_map(|mut s| {
    //                 let mut a = vec![];
    //                 let mut b = None;
    //                 if s.s.contains('\n') {
    //                     let str = std::mem::take(&mut s.s);
    //                     a.extend(str.split_inclusive('\n').map(move |line| StyledString {
    //                         s: line.to_string(),
    //                         ..s.clone()
    //                     }));
    //                 } else {
    //                     b = Some(s);
    //                 }
    //                 a.into_iter().chain(b.into_iter())
    //             })
    //             .group_by(|x| {
    //                 if let Some(x) = next_line_count.take() {
    //                     line_count = x;
    //                 }
    //                 if x.s.chars().last().map(|c| c == '\n').unwrap_or(false) {
    //                     next_line_count = Some(line_count + 1);
    //                 }
    //                 line_count
    //             });

    //         let mut line_num = 0;
    //         for (_key, line) in &lines {
    //             let style: TextStyleParams = self.settings.style.clone().into();
    //             column
    //                 .spawn(NodeBundle {
    //                     style: Style {
    //                         flex_direction: FlexDirection::Row,
    //                         ..default()
    //                     },
    //                     ..default()
    //                 })
    //                 .with_children(|parent| {
    //                     if self.state.cursor_visible && line_num == self.state.cursor_pos[1] {
    //                         text_style::bevy::render_iter(
    //                             parent,
    //                             &style,
    //                             cursorify_iter(line, self.state.cursor_pos[0], white),
    //                         );
    //                     } else {
    //                         text_style::bevy::render_iter(parent, &style, line);
    //                     }
    //                 });
    //             line_num += 1;
    //         }
    //     });
    //     Ok(())
    // }

    /// Utility function for line input.
    /// Set initial position based on the position after drawing.
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

pub fn asky_system<T>(
    mut commands: Commands,
    char_evr: EventReader<ReceivedCharacter>,
    key_evr: EventReader<KeyboardInput>,
    asky_settings: Res<BevyAskySettings>,
    mut render_state: Local<BevyRendererState>,
    mut query: Query<(Entity, &mut AskyNode<T>, Option<&Children>)>,
) where
    T: Typeable<KeyEvent> + Valuable + Send + Sync + 'static,
    AskyNode<T>: Printable,
{
    let key_event = KeyEvent::new(char_evr, key_evr);
    for (entity, mut prompt, children) in query.iter_mut() {
        match prompt.1 {
            AskyState::Complete => {
                continue;
            }
            AskyState::Hidden => {
                if let Some(children) = children {
                    commands.entity(entity).remove_children(children);
                    for child in children {
                        commands.entity(*child).despawn_recursive();
                    }
                }
            }
            AskyState::Waiting(_) | AskyState::Reading => {
                if !is_abort_key(&key_event)
                    && !prompt.will_handle_key(&key_event)
                    && render_state.draw_time != DrawTime::First
                {
                    continue;
                }
                // For the terminal it had an abort key handling happen here.
                if is_abort_key(&key_event) {
                    let waiting_maybe = std::mem::replace(&mut prompt.1, AskyState::Complete);
                    if let AskyState::Waiting(promise) = waiting_maybe {
                        promise.reject(Error::Cancel);
                    }
                    render_state.draw_time = DrawTime::Last;
                } else if prompt.handle_key(&key_event) {
                    // It's done.
                    let waiting_maybe = std::mem::replace(&mut prompt.1, AskyState::Complete);
                    if let AskyState::Waiting(promise) = waiting_maybe {
                        match prompt.0.value() {
                            Ok(v) => promise.resolve(v),
                            Err(e) => promise.reject(e),
                        }
                    }
                    render_state.draw_time = DrawTime::Last;
                }
                if let Some(children) = children {
                    commands.entity(entity).remove_children(children);
                    for child in children {
                        commands.entity(*child).despawn_recursive();
                    }
                }
                let mut renderer =
                    BevyRenderer::new(&asky_settings, &mut render_state, &mut commands, entity);
                let draw_time = renderer.draw_time();
                let _ = prompt.draw(&mut renderer);
                // This is just to affirm that we're not recreating the nodes unless we need to.
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

fn is_abort_key(key: &KeyEvent) -> bool {
    for code in &key.codes {
        if code == &KeyCode::Escape {
            return true;
        }
    }
    false
}

pub struct AskyPlugin;

impl Plugin for AskyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AskyParamConfig {
            state: Arc::new(Mutex::new(AskyParamState {
                closures: Vec::new(),
            })),
        })
        // .add_systems(Update, asky_system::<Confirm>)
        // .add_systems(Update, asky_system::<Toggle>)
        // .add_systems(Update, asky_system::<crate::Text>)
        // .add_systems(Update, asky_system::<Number<u8>>)
        // .add_systems(Update, asky_system::<Number<u16>>)
        // .add_systems(Update, asky_system::<Number<u32>>)
        // .add_systems(Update, asky_system::<Number<u64>>)
        // .add_systems(Update, asky_system::<Number<u128>>)
        // .add_systems(Update, asky_system::<Number<i8>>)
        // .add_systems(Update, asky_system::<Number<i16>>)
        // .add_systems(Update, asky_system::<Number<i32>>)
        // .add_systems(Update, asky_system::<Number<i64>>)
        // .add_systems(Update, asky_system::<Number<i128>>)
        // .add_systems(Update, asky_system::<Number<f32>>)
        // .add_systems(Update, asky_system::<Number<f64>>)
        // .add_systems(Update, asky_system::<Select<'_, Cow<'static, str>>>)
        // .add_systems(Update, asky_system::<Select<'_, &'static str>>)
        // .add_systems(Update, asky_system::<Password>)
        // .add_systems(Update, asky_system::<Message>)
        // .add_systems(Update, asky_system::<MultiSelect<'static, &'static str>>)
        // .add_systems(Update, asky_system::<MultiSelect<'_, Cow<'static, str>>>)
        .add_systems(Update, poll_tasks::<()>)
        .add_systems(Update, poll_tasks_err::<()>)
        .add_systems(Update, run_closures)
        .add_systems(Update, run_timers);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_scan() {
        let a = [1, 2, 3, 4];

        let mut iter = a.iter().scan(0, |state, &x| {
            // each iteration, we'll multiply the state by the element ...
            *state += 1;

            // ... and terminate if the state exceeds 6
            if *state % 2 == 0 {
                return None;
            }
            // ... else yield the negation of the state
            Some(-x)
        });

        assert_eq!(iter.next(), Some(-1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), Some(-3));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }
}

// Confirm

impl Typeable<KeyCode> for Confirm<'_> {
    fn will_handle_key(&self, key: &KeyCode) -> bool {
        match key {
            KeyCode::Left | KeyCode::H => true,
            KeyCode::Right | KeyCode::L => true,
            KeyCode::Y => true,
            KeyCode::N => true,
            KeyCode::Return | KeyCode::Back => true,
            _ => false,
        }
    }

    fn handle_key(&mut self, key: &KeyCode) -> bool {
        let mut submit = false;

        match key {
            // update value
            KeyCode::Left | KeyCode::H => self.active = false,
            KeyCode::Right | KeyCode::L => self.active = true,
            // update value and submit
            KeyCode::Y => submit = self.update_and_submit(true),
            KeyCode::N => submit = self.update_and_submit(false),
            // submit current/initial value
            KeyCode::Return | KeyCode::Back => submit = true,
            _ => (),
        }

        submit
    }
}

// impl Printable for AskyNode<Confirm<'_>> {
//     fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
//         let mut out = ColoredStrings::default();
//         (self.formatter)(self, renderer.draw_time(), &mut out);
//         renderer.hide_cursor()?;
//         renderer.print(out)
//     }
// }

// MultiSelect

impl<T> Typeable<KeyCode> for MultiSelect<'_, T> {
    fn handle_key(&mut self, key: &KeyCode) -> bool {
        use crate::prompts::select::Direction;
        let mut submit = false;

        match key {
            // submit
            KeyCode::Return | KeyCode::Back => submit = self.validate_to_submit(),
            KeyCode::Space => self.toggle_focused(),
            // update value
            KeyCode::Up | KeyCode::K => self.input.move_cursor(Direction::Up),
            KeyCode::Down | KeyCode::J => self.input.move_cursor(Direction::Down),
            KeyCode::Left | KeyCode::H => self.input.move_cursor(Direction::Left),
            KeyCode::Right | KeyCode::L => self.input.move_cursor(Direction::Right),
            _ => (),
        }

        submit
    }
}

// impl<T> Printable for AskyNode<MultiSelect<'_, T>> {
//     fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
//         let mut out = ColoredStrings::default();
//         (self.formatter)(self, renderer.draw_time(), &mut out);
//         renderer.print(out)
//     }
// }

// Number

impl<T: NumLike> Typeable<KeyEvent> for Number<'_, T> {
    fn will_handle_key(&self, key: &KeyEvent) -> bool {
        for c in key.chars.iter() {
            if !c.is_control() {
                return true;
            }
        }

        for code in &key.codes {
            if match code {
                // submit
                KeyCode::Return => true,
                // remove delete
                KeyCode::Back => true,
                KeyCode::Delete => true,
                // move cursor
                KeyCode::Left => true,
                KeyCode::Right => true,
                _ => false,
            } {
                return true;
            }
        }

        false
    }
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        use crate::prompts::text::Direction;
        let mut submit = false;

        for c in key.chars.iter() {
            if !c.is_control() {
                self.insert(*c);
            }
        }

        for code in &key.codes {
            match code {
                // submit
                KeyCode::Return => submit = self.validate_to_submit(),
                // remove delete
                KeyCode::Back => self.input.backspace(),
                KeyCode::Delete => self.input.delete(),
                // move cursor
                KeyCode::Left => self.input.move_cursor(Direction::Left),
                KeyCode::Right => self.input.move_cursor(Direction::Right),
                _ => (),
            };
        }

        submit
    }
}

// impl<T: NumLike> Printable for AskyNode<Number<'_, T>> {
//     fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
//         let mut out = ColoredStrings::default();
//         let cursor = (self.formatter)(self, renderer.draw_time(), &mut out);
//         renderer.show_cursor()?;
//         renderer.set_cursor(cursor)?;
//         renderer.print(out)
//     }
// }

impl<'a, T: NumLike + 'a> Default for Number<'a, T> {
    fn default() -> Self {
        Self::new("")
    }
}

// Password

impl Typeable<KeyEvent> for Password<'_> {
    fn will_handle_key(&self, key: &KeyEvent) -> bool {
        for c in key.chars.iter() {
            if !c.is_control() {
                return true;
            }
        }

        for code in &key.codes {
            if match code {
                // submit
                KeyCode::Return => true,
                // type
                // KeyCode::Char(c) => self.input.insert(c),
                // remove delete
                KeyCode::Back => true,
                KeyCode::Delete => true,
                // move cursor
                KeyCode::Left => true,
                KeyCode::Right => true,
                _ => false,
            } {
                return true;
            }
        }

        false
    }
    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        use crate::prompts::text::Direction;
        let mut submit = false;

        for c in key.chars.iter() {
            if !c.is_control() {
                self.input.insert(*c);
            }
        }

        for code in &key.codes {
            match code {
                // submit
                KeyCode::Return => submit = self.validate_to_submit(),
                // remove delete
                KeyCode::Back => self.input.backspace(),
                KeyCode::Delete => self.input.delete(),
                // move cursor
                KeyCode::Left => self.input.move_cursor(Direction::Left),
                KeyCode::Right => self.input.move_cursor(Direction::Right),
                _ => (),
            };
        }

        submit
    }
}

// impl Printable for crate::bevy::AskyNode<Password<'_>> {
//     fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
//         let mut out = ColoredStrings::default();
//         let cursor = (self.formatter)(self, renderer.draw_time(), &mut out);
//         renderer.show_cursor()?;
//         renderer.set_cursor(cursor)?;
//         renderer.print(out)
//     }
// }

// Select

impl<T> Typeable<KeyCode> for Select<'_, T> {
    fn handle_key(&mut self, key: &KeyCode) -> bool {
        use crate::prompts::select::Direction;
        let mut submit = false;

        match key {
            // submit
            KeyCode::Return | KeyCode::Back => submit = self.validate_to_submit(),
            // update value
            KeyCode::Up | KeyCode::K => self.input.move_cursor(Direction::Up),
            KeyCode::Down | KeyCode::J => self.input.move_cursor(Direction::Down),
            KeyCode::Left | KeyCode::H => self.input.move_cursor(Direction::Left),
            KeyCode::Right | KeyCode::L => self.input.move_cursor(Direction::Right),
            _ => (),
        }

        submit
    }
}

// impl<T: Send> Printable for crate::bevy::AskyNode<Select<'_, T>> {
//     fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
//         let mut out = ColoredStrings::default();
//         (self.formatter)(self, renderer.draw_time(), &mut out);
//         renderer.print(out)
//     }
// }

impl Typeable<KeyEvent> for crate::Text<'_> {
    fn will_handle_key(&self, key: &KeyEvent) -> bool {
        for c in key.chars.iter() {
            if !c.is_control() {
                return true;
            }
        }

        for code in &key.codes {
            use KeyCode::*;
            match code {
                Return | Back | Delete | Left | Right => return true,
                _ => (),
            }
        }
        false
    }

    fn handle_key(&mut self, key: &KeyEvent) -> bool {
        use crate::prompts::text::Direction;
        let mut submit = false;

        for c in key.chars.iter() {
            if !c.is_control() {
                self.input.insert(*c);
            }
        }

        for code in &key.codes {
            match code {
                // submit
                KeyCode::Return => submit = self.validate_to_submit(),
                // remove delete
                KeyCode::Back => self.input.backspace(),
                KeyCode::Delete => self.input.delete(),
                // move cursor
                KeyCode::Left => self.input.move_cursor(Direction::Left),
                KeyCode::Right => self.input.move_cursor(Direction::Right),
                _ => (),
            };
        }

        submit
    }
}

// impl Printable for AskyNode<crate::Text<'_>> {
//     fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
//         let mut out = ColoredStrings::default();
//         let cursor = (self.formatter)(self, renderer.draw_time(), &mut out);
//         renderer.show_cursor()?;
//         renderer.set_cursor(cursor)?;
//         renderer.print(out)
//     }
// }

impl Typeable<KeyCode> for Toggle<'_> {
    fn handle_key(&mut self, key: &KeyCode) -> bool {
        let mut submit = false;

        match key {
            // update value
            KeyCode::Left | KeyCode::H => self.active = false,
            KeyCode::Right | KeyCode::L => self.active = true,
            // submit current/initial value
            KeyCode::Return | KeyCode::Back => submit = true,
            _ => (),
        }

        submit
    }
}

// impl Printable for AskyNode<Toggle<'_>> {
//     fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
//         let mut out = ColoredStrings::default();
//         (self.formatter)(self, renderer.draw_time(), &mut out);
//         renderer.print(out)
//     }
// }

impl Typeable<KeyCode> for Message<'_> {
    fn will_handle_key(&self, _key: &KeyCode) -> bool {
        true
    }

    fn handle_key(&mut self, _key: &KeyCode) -> bool {
        true
    }
}

// impl Printable for AskyNode<Message<'_>> {
//     fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
//         let mut out = ColoredStrings::default();
//         (self.formatter)(self, renderer.draw_time(), &mut out);
//         renderer.hide_cursor()?;
//         renderer.print(out)
//     }
// }
