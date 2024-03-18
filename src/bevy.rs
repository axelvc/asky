use crate::utils::renderer::{Printable, Renderer};

use crate::Typeable;
use crate::{DrawTime, NumLike};
use crate::style;
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
use std::borrow::Cow;
use std::sync::{Arc, Mutex};
// use std::rc::Rc;

use bevy::prelude::*;
use std::future::Future;
use futures_lite::future;

use std::ops::{Deref, DerefMut};

use crate::text_style_adapter::StyledStringWriter;
use crate::{Confirm, Error, Message, MultiSelect, Number, Password, Select, Toggle, Valuable};
use bevy::tasks::{block_on, AsyncComputeTaskPool, Task};
use bevy::window::RequestRedraw;
use itertools::Itertools;
use text_style::{self, bevy::TextStyleParams, AnsiColor, StyledString};

#[derive(Component, Debug)]
pub struct AskyNode<T: Typeable<KeyEvent> + Valuable> {
    prompt: T,
    promise: Option<Producer<T::Output, Error>>,
}

#[derive(Component, Debug)]
struct AskyDelay(Timer, Option<Producer<(), Error>>);

#[derive(Debug, Default, Component)]
pub enum AskyState {
    #[default]
    Waiting,
    Complete,
    Hidden,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AskyPrompt {
    #[default]
    Inactive,
    Active,
}

fn run_timers(mut commands: Commands, mut query: Query<(Entity, &mut AskyDelay)>, time: Res<Time>,
    mut redraw: EventWriter<RequestRedraw>,
) {
    for (id, mut asky_delay) in query.iter_mut() {
        asky_delay.0.tick(time.delta());
        if asky_delay.0.finished() {
            asky_delay.1.take().expect("Promise not there").resolve(());
            commands.entity(id).remove::<AskyDelay>();
        }
        // I would RequestTick to just run the systems once, but this seems to
        // be the way.
        redraw.send(RequestRedraw);
    }
}

#[derive(Clone)]
pub struct Asky {
    config: AskyParamConfig,
}

#[derive(Resource, Clone)]
pub struct AskyParamConfig {
    pub(crate) state: Arc<Mutex<AskyParamState>>,
}

type Closure = dyn FnOnce(&mut Commands, Option<Entity>, Option<&Children>)
                          -> Result<(), Error> + 'static + Send + Sync;

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
                              .spawn((node, AskyNode { prompt, promise: Some(promise) },  AskyState::Waiting))
                              .id();
                    commands.entity(entity.unwrap()).push_children(&[id]);
                    Ok(())
                },
            ),
            Some(dest),
        ));
        waiter
    }

    pub fn prompt_styled<T: Typeable<KeyEvent> + Valuable + Send + Sync + 'static, S>(
        &mut self,
        prompt: T,
        dest: Entity,
        style: S
    ) -> Consumer<T::Output, Error>
    where S: style::Style + Send + Sync + 'static{
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
                              .spawn((node, AskyNode { prompt, promise: Some(promise) },  AskyState::Waiting,
                              AskyStyle(Box::new(style))))
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
    mut redraw: EventWriter<RequestRedraw>,
    query: Query<Option<&Children>>,
) {
    let mut ran_closure = false;
    for (closure, id_maybe) in config
        .state
        .lock()
        .expect("Unable to lock mutex")
        .closures
        .drain(0..)
    {
        let children = id_maybe
            .and_then(|id| query.get(id).expect("Unable to get children"));
        eprintln!("run closure");
        // TODO: Handle error
        let _ = closure(&mut commands, id_maybe, children);
        ran_closure = true;
    }
    if ran_closure {
        redraw.send(RequestRedraw);
    }
}

fn check_prompt_state(
    query: Query<&AskyState>,
    delays: Query<&AskyDelay>,
    asky_prompt: Res<State<AskyPrompt>>,
    mut next_asky_prompt: ResMut<NextState<AskyPrompt>>,
    mut redraw: EventWriter<RequestRedraw>,
) {
    let was_active = matches!(**asky_prompt, AskyPrompt::Active);
    let is_active = query.iter().filter(|x| matches!(*x, AskyState::Waiting)).next().is_some()
        || delays.iter().next().is_some();
    if was_active ^ is_active {
        next_asky_prompt.set(if is_active { AskyPrompt::Active } else { AskyPrompt::Inactive });
        redraw.send(RequestRedraw);
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
                eprintln!("Got error here {:?}.", error);
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
        &self.prompt
    }
}

impl<T: Typeable<KeyEvent> + Valuable> DerefMut for AskyNode<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.prompt
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
            chars: char_evr.read().flat_map(|e| e.char.chars()).collect(),
            // keys,
            codes: key_evr
                .read()
                .filter_map(|e| {
                    if e.state == bevy::input::ButtonState::Pressed {
                        Some(e.key_code)
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct BevyAskySettings {
    pub style: TextStyle,
}

pub fn asky_system<T>(
    mut commands: Commands,
    char_evr: EventReader<ReceivedCharacter>,
    key_evr: EventReader<KeyboardInput>,
    asky_settings: Res<BevyAskySettings>,
    // mut render_state: Local<BevyRendererState>,
    mut renderer: Local<StyledStringWriter>,
    mut query: Query<(Entity, &mut AskyNode<T>, &mut AskyState, Option<&Children>, Option<&AskyStyle>)>,
) where
    T: Printable + Typeable<KeyEvent> + Valuable + Send + Sync + 'static,
    // AskyNode<T>: Printable,
{
    // eprint!("1");
    let key_event = KeyEvent::new(char_evr, key_evr);
    for (entity, mut node, mut state, children, style_maybe) in query.iter_mut() {
        match *state {
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
            AskyState::Waiting => {
                eprint!("2");
                if !is_abort_key(&key_event)
                    && !node.will_handle_key(&key_event)
                    && renderer.state.draw_time != DrawTime::First
                {
                    continue;
                }
                // For the terminal it had an abort key handling happen here.
                if is_abort_key(&key_event) {
                    *state = AskyState::Complete;

                    let waiting_maybe = node.promise.take();//std::mem::replace(&mut state, AskyState::Complete);
                    if let Some(promise) = waiting_maybe {
                        promise.reject(Error::Cancel);
                    }
                    renderer.state.draw_time = DrawTime::Last;
                } else if node.handle_key(&key_event) {
                    // It's done.
                    *state = AskyState::Complete;
                    let waiting_maybe = node.promise.take();//std::mem::replace(&mut state, AskyState::Complete);
                    // let waiting_maybe = std::mem::replace(&mut state, AskyState::Complete);
                    if let Some(promise) = waiting_maybe {
                        match node.prompt.value() {
                            Ok(v) => promise.resolve(v),
                            Err(e) => promise.reject(e),
                        }
                    }
                    renderer.state.draw_time = DrawTime::Last;
                }
                if let Some(children) = children {
                    commands.entity(entity).remove_children(children);
                    for child in children {
                        commands.entity(*child).despawn_recursive();
                    }
                }
                eprint!("3");
                // let mut renderer =
                //     BevyRenderer::new(&asky_settings, &mut render_state, &mut commands, entity);
                // let draw_time = renderer.draw_time();
                renderer.cursor_pos = None;
                renderer.cursor_pos_save = None;
                match style_maybe {
                    Some(style) => {
                        let _ = node.draw_with_style(&mut *renderer, &*style.0);
                    }
                    None => {
                        let _ = node.draw(&mut *renderer);
                    }
                }
                bevy_render(&mut commands, &asky_settings, &mut renderer, entity);
                // This is just to affirm that we're not recreating the nodes unless we need to.
                let draw_time = renderer.draw_time();
                eprint!(".");
                if draw_time == DrawTime::First {
                    renderer.update_draw_time();
                } else if draw_time == DrawTime::Last {
                    renderer.clear();
                    let waiting_maybe = node.promise.take();
                    *state = AskyState::Complete;
                    // let waiting_maybe = std::mem::replace(&mut node.1, AskyState::Complete);
                    if let Some(promise) = waiting_maybe {
                        match node.prompt.value() {
                            Ok(v) => promise.resolve(v),
                            Err(e) => promise.reject(e),
                        }
                    }
                }
            }
        }
    }
}

fn bevy_render(
    commands: &mut Commands,
    settings: &BevyAskySettings,
    out: &mut StyledStringWriter,
    column: Entity,
) {
    // -> io::Result<()>
    let white = AnsiColor::White.dark();

    let strings = if out.state.cursor_visible {
        out.drain_with_styled_cursor(white)
    } else {
        std::mem::take(&mut out.strings)
    };

    commands.entity(column).with_children(|column| {
        let mut next_line_count: Option<usize> = None;
        let mut line_count: usize = 0;
        let lines = strings
            .into_iter()
            .flat_map(|mut s| {
                let mut a = vec![];
                let mut b = None;
                if s.s.contains('\n') {
                    let str = std::mem::take(&mut s.s);
                    a.extend(str.split_inclusive('\n').map(move |line| StyledString {
                        s: line.to_string(),
                        ..s.clone()
                    }));
                } else {
                    b = Some(s);
                }
                a.into_iter().chain(b)
            })
            .group_by(|x| {
                if let Some(x) = next_line_count.take() {
                    line_count = x;
                }
                if x.s.chars().last().map(|c| c == '\n').unwrap_or(false) {
                    next_line_count = Some(line_count + 1);
                }
                line_count
            });

        // let mut line_num = 0;
        for (_key, line) in &lines {
            let style: TextStyleParams = settings.style.clone().into();
            column
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // if out.state.cursor_visible && line_num == out.state.cursor_pos[1] {
                    //     text_style::bevy::render_iter(
                    //         parent,
                    //         &style,
                    //         cursorify_iter(line, out.state.cursor_pos[0], white),
                    //     );
                    // } else {
                        text_style::bevy::render_iter(parent, &style, line);
                    // }
                });
            // line_num += 1;
        }
    });
}

fn is_abort_key(key: &KeyEvent) -> bool {
    for code in &key.codes {
        if code == &KeyCode::Escape {
            return true;
        }
    }
    false
}

#[derive(Component)]
pub struct AskyStyle(pub Box<dyn style::Style + 'static + Send + Sync>);

pub struct AskyPlugin;

impl Plugin for AskyPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(AskyParamConfig {
                state: Arc::new(Mutex::new(AskyParamState {
                    closures: Vec::new(),
                })),
            })
            .init_resource::<BevyAskySettings>()
            .init_state::<AskyPrompt>()
            .add_systems(Update, asky_system::<Confirm>)
            .add_systems(Update, asky_system::<Toggle>)
            .add_systems(Update, asky_system::<crate::Text>)
            .add_systems(Update, asky_system::<Number<u8>>)
            .add_systems(Update, asky_system::<Number<u16>>)
            .add_systems(Update, asky_system::<Number<u32>>)
            .add_systems(Update, asky_system::<Number<u64>>)
            .add_systems(Update, asky_system::<Number<u128>>)
            .add_systems(Update, asky_system::<Number<i8>>)
            .add_systems(Update, asky_system::<Number<i16>>)
            .add_systems(Update, asky_system::<Number<i32>>)
            .add_systems(Update, asky_system::<Number<i64>>)
            .add_systems(Update, asky_system::<Number<i128>>)
            .add_systems(Update, asky_system::<Number<f32>>)
            .add_systems(Update, asky_system::<Number<f64>>)
            .add_systems(Update, asky_system::<Select<'_, Cow<'static, str>>>)
            .add_systems(Update, asky_system::<Select<'_, &'static str>>)
            .add_systems(Update, asky_system::<Password>)
            .add_systems(Update, asky_system::<Message>)
            .add_systems(Update, asky_system::<MultiSelect<'static, &'static str>>)
            .add_systems(Update, asky_system::<MultiSelect<'_, Cow<'static, str>>>)
            .add_systems(Update, poll_tasks::<()>)
            .add_systems(Update, poll_tasks_err::<()>)
            .add_systems(Update, check_prompt_state)
            .add_systems(Update, run_closures)
            .add_systems(Update, run_timers);
    }
}

// Confirm
impl Typeable<KeyCode> for Confirm<'_> {
    fn will_handle_key(&self, key: &KeyCode) -> bool {
        match key {
            KeyCode::ArrowLeft | KeyCode::KeyH => true,
            KeyCode::ArrowRight | KeyCode::KeyL => true,
            KeyCode::KeyY => true,
            KeyCode::KeyN => true,
            KeyCode::Enter | KeyCode::Backspace => true,
            _ => false,
        }
    }

    fn handle_key(&mut self, key: &KeyCode) -> bool {
        let mut submit = false;

        match key {
            // update value
            KeyCode::ArrowLeft | KeyCode::KeyH => self.active = false,
            KeyCode::ArrowRight | KeyCode::KeyL => self.active = true,
            // update value and submit
            KeyCode::KeyY => submit = self.update_and_submit(true),
            KeyCode::KeyN => submit = self.update_and_submit(false),
            // submit current/initial value
            KeyCode::Enter | KeyCode::Backspace => submit = true,
            _ => (),
        }

        submit
    }
}

// MultiSelect

impl<T> Typeable<KeyCode> for MultiSelect<'_, T> {
    fn handle_key(&mut self, key: &KeyCode) -> bool {
        use crate::prompts::select::Direction;
        let mut submit = false;

        match key {
            // submit
            KeyCode::Enter | KeyCode::Backspace => submit = self.validate_to_submit(),
            KeyCode::Space => self.toggle_focused(),
            // update value
            KeyCode::ArrowUp | KeyCode::KeyK => self.input.move_cursor(Direction::Up),
            KeyCode::ArrowDown | KeyCode::KeyJ => self.input.move_cursor(Direction::Down),
            KeyCode::ArrowLeft | KeyCode::KeyH => self.input.move_cursor(Direction::Left),
            KeyCode::ArrowRight | KeyCode::KeyL => self.input.move_cursor(Direction::Right),
            _ => (),
        }

        submit
    }
}

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
                KeyCode::Enter => true,
                // remove delete
                KeyCode::Backspace => true,
                KeyCode::Delete => true,
                // move cursor
                KeyCode::ArrowLeft => true,
                KeyCode::ArrowRight => true,
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
                KeyCode::Enter => submit = self.validate_to_submit(),
                // remove delete
                KeyCode::Backspace => self.input.backspace(),
                KeyCode::Delete => self.input.delete(),
                // move cursor
                KeyCode::ArrowLeft => self.input.move_cursor(Direction::Left),
                KeyCode::ArrowRight => self.input.move_cursor(Direction::Right),
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
                KeyCode::Enter => true,
                // type
                // KeyCode::Char(c) => self.input.insert(c),
                // remove delete
                KeyCode::Backspace => true,
                KeyCode::Delete => true,
                // move cursor
                KeyCode::ArrowLeft => true,
                KeyCode::ArrowRight => true,
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
                KeyCode::Enter => submit = self.validate_to_submit(),
                // remove delete
                KeyCode::Backspace => self.input.backspace(),
                KeyCode::Delete => self.input.delete(),
                // move cursor
                KeyCode::ArrowLeft => self.input.move_cursor(Direction::Left),
                KeyCode::ArrowRight => self.input.move_cursor(Direction::Right),
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
            KeyCode::Enter | KeyCode::Backspace => submit = self.validate_to_submit(),
            // update value
            KeyCode::ArrowUp | KeyCode::KeyK => self.input.move_cursor(Direction::Up),
            KeyCode::ArrowDown | KeyCode::KeyJ => self.input.move_cursor(Direction::Down),
            KeyCode::ArrowLeft | KeyCode::KeyH => self.input.move_cursor(Direction::Left),
            KeyCode::ArrowRight | KeyCode::KeyL => self.input.move_cursor(Direction::Right),
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
                Enter | Backspace | Delete | ArrowLeft | ArrowRight => return true,
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
                KeyCode::Enter => submit = self.validate_to_submit(),
                // remove delete
                KeyCode::Backspace => self.input.backspace(),
                KeyCode::Delete => self.input.delete(),
                // move cursor
                KeyCode::ArrowLeft => self.input.move_cursor(Direction::Left),
                KeyCode::ArrowRight => self.input.move_cursor(Direction::Right),
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
            KeyCode::ArrowLeft | KeyCode::KeyH => self.active = false,
            KeyCode::ArrowRight | KeyCode::KeyL => self.active = true,
            // submit current/initial value
            KeyCode::Enter | KeyCode::Backspace => submit = true,
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

// impl Printable for AskyNode<Message<'_>> {
//     fn draw<R: Renderer>(&self, renderer: &mut R) -> io::Result<()> {
//         let mut out = ColoredStrings::default();
//         (self.formatter)(self, renderer.draw_time(), &mut out);
//         renderer.hide_cursor()?;
//         renderer.print(out)
//     }
// }
