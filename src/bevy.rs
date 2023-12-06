use crate::utils::renderer::{Printable, Renderer};
use std::sync::{Arc, Mutex};
use promise_out::{pair::{Producer, Consumer}, Promise};
use crate::DrawTime;
use crate::Typeable;
use bevy::{input::keyboard::KeyboardInput, utils::Duration, ecs::{world::unsafe_world_cell::UnsafeWorldCell, component::Tick, system::{SystemParam, SystemMeta}}};

use bevy::prelude::*;
use colored::{Color as Colored, ColoredString, Colorize};
use std::io;
use std::future::Future;

use std::ops::{Deref, DerefMut};

use crate::{Confirm, Message, MultiSelect, Number, Password, Select, Toggle, Valuable, Error, ColoredStrings};
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on};
use futures_lite::future;
use text_style::{self, StyledString, StyledStr, AnsiColor, AnsiMode};

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


// #[derive(SystemParam)]
// pub struct Asky<'w, 's> {
pub struct Asky {
    // commands: Commands<'w, 's>,
    config: AskyParamConfig,
}

#[derive(Resource, Clone)]
pub struct AskyParamConfig {
    pub(crate) state: Arc<Mutex<AskyParamState>>,
}

// type ClosureCommand = dyn FnOnce(&mut Commands) -> Result<(), Error> + 'static + Send + Sync;
type Closure = dyn FnOnce(&mut Commands, Option<Entity>, Option<&Children>) -> Result<(), Error> + 'static + Send + Sync;
// #[derive(Debug)]
pub struct AskyParamState {
    pub(crate) closures: Vec<(Box<Closure>, Option<Entity>)>,
}

// impl<'w, 's> Asky<'w, 's> {
impl Asky {
    fn new(config: AskyParamConfig) -> Self {
        Self { config }
    }

    pub fn listen<T: Typeable<KeyEvent> + Valuable + Send + Sync + 'static>(&mut self, prompt: T, dest: Entity)
                                                                            -> Consumer<T::Output, Error> {
        let (promise, waiter) = Producer::<T::Output, Error>::new();
        self.config.state.lock().unwrap().closures.push((Box::new(move |commands: &mut Commands, entity: Option<Entity>, _children: Option<&Children>| {

            let node = NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            };
            let id = commands.spawn((node, AskyNode(prompt, AskyState::Waiting(promise)))).id();
            commands.entity(entity.unwrap()).push_children(&[id]); 
            Ok(())
        }), Some(dest)));
        waiter
    }

    pub fn clear(&mut self, dest: Entity) -> Consumer<(), Error> {
        let (promise, waiter) = Producer::<(), Error>::new();
        self.config.state.lock().unwrap().closures.push((Box::new(move |commands: &mut Commands, entity: Option<Entity>, children_maybe: Option<&Children>| {
            commands.entity(entity.unwrap()).clear_children();
            if let Some(children) = children_maybe {
                for child in children.iter() {
                    commands.entity(*child).despawn_recursive();
                }
            }
            promise.resolve(());
            Ok(())
        }), Some(dest)));
        waiter
    }

    pub fn delay(&mut self, duration: Duration) -> Consumer<(), Error> {
        let (promise, waiter) = Producer::<(), Error>::new();
        self.config.state.lock().unwrap().closures.push((Box::new(move |commands: &mut Commands, _entity: Option<Entity>, _children_maybe: Option<&Children>| {
            commands.spawn(AskyDelay(Timer::new(duration, TimerMode::Once), Some(promise)));
            Ok(())
        }), None));
        waiter
    }
}

fn run_closures(config: ResMut<AskyParamConfig>, mut commands: Commands, query: Query<Option<&Children>>) {
        for (closure, id_maybe) in config.state.lock().expect("Unable to lock mutex").closures.drain(0..) {
            let children = id_maybe.map(|id| query.get(id).expect("Unable to get children")).unwrap_or(None);
            let _ = closure(&mut commands, id_maybe, children);
        }
}

unsafe impl SystemParam for Asky {
    type State = AskyParamConfig;
    type Item<'w, 's> = Asky;

    fn init_state(world: &mut World, _system_meta: &mut SystemMeta) -> Self::State {
        world.get_resource_mut::<AskyParamConfig>().expect("No AskyParamConfig setup.").clone()
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

pub fn future_sink<T: Send + 'static, F: Future<Output = T>+ Send + 'static>(
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

pub fn poll_tasks<T: Send + Sync + 'static>(mut commands: Commands, mut tasks: Query<(Entity, &mut TaskSink<T>)>) {
    for (entity, mut task) in &mut tasks {
        if block_on(future::poll_once(&mut task.0)).is_some() {
            // Once
            commands.entity(entity).despawn();
        }
    }
}

pub fn poll_tasks_err<T: Send + Sync + 'static>(mut commands: Commands, _asky: Asky, mut tasks: Query<(Entity, &mut TaskSink<Result<T, Error>>)>) {
    for (entity, mut task) in &mut tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.0)) {
            // Once
            if let Err(_error) = result {
                eprintln!("Got here.");
                // FIXME: I need the right entity to make this work.
                // let _ = asky.listen(Message::new(format!("{:?}", error)), entity);
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

    fn cursorify(
        cs: StyledString,
        i: usize,
        cursor_color: text_style::Color,
    ) -> impl Iterator<Item = StyledString> {
        let to_colored_string = |s: String| -> StyledString {
            StyledString {
                s: s,
                ..cs.clone()
            }
        };
        let mut input = cs.s.to_string();
        let mut right = None;
        if let Some((byte_index, _)) = input.char_indices().nth(i + 1) {
            let (l, r) = input.split_at(byte_index);
            right = Some(to_colored_string(r.to_owned()));
            input = l.to_owned();
        }
        let cursor = Some(
            to_colored_string(input.pop().expect("Could not get cursor").to_string())
                .on(cursor_color),
        );
        let left = Some(to_colored_string(input));
        left.into_iter()
            .chain(cursor.into_iter().chain(right.into_iter()))
    }
}

fn split<'a>(ss: &'a StyledStr<'_>, pat: char) -> impl Iterator<Item = StyledStr<'a>> {
    ss.s.split(pat).map(|str| StyledStr { s: str, ..ss.clone() })
}

pub trait DivideAndSeparate : Iterator + Sized {
    /// Given an iterator, separate it with a separator closure into an unfused
    /// iterator that emits None between divisions. The separator is fed
    /// elements from the self iterator.
    ///
    /// When it returns `Ok(x)`, `x` will be emitted by the `DividedIterator` as
    /// the next item. Without any `Err(_)` divide_and_separate degenerates into
    /// [`map`][].
    ///
    /// ```
    /// use asky::bevy::DivideAndSeparate;
    /// assert_eq!([1,2,3].into_iter().divide_and_separate(|x| Ok(x)).collect::<Vec<i32>>(), [1,2,3]);
    /// ```
    ///
    /// When the separator returns `Err(None, None)` `DividedIterator` will not
    /// emit an element and end the current iterator and begin the next
    /// iterator.
    ///
    /// ```
    /// use asky::bevy::DivideAndSeparate;
    /// let mut v: Vec<i32> = Vec::new();
    /// let mut iter = [1,2,3].into_iter().divide_and_separate(|x| {
    ///     if x == 2 { Err((None, None)) } else { Ok(x) }
    /// });
    /// v.extend(iter.by_ref()); // Use by_ref() so the iterator isn't consumed by `extend()`.
    /// assert_eq!(v, [1]);
    /// v.clear();
    /// v.extend(iter.by_ref());
    /// assert_eq!(v, [3]);
    /// ```
    ///
    /// When the separator returns `Err(Some(a), Some(b))`, the
    /// `DividedIterator` will end the current iterator with a and begin the
    /// next iterator with b.
    ///
    /// ```
    /// # use asky::bevy::DivideAndSeparate;
    /// let mut v: Vec<i32> = Vec::new();
    /// let mut iter = [1,2,3].into_iter().divide_and_separate(|x| {
    ///     if x % 2 == 0 { Err((Some(x), Some(x * 2))) } else { Ok(x) }
    /// });
    /// v.extend(iter.by_ref()); // Use by_ref() so the iterator isn't consumed by `extend()`.
    /// assert_eq!(v, [1, 2]);
    /// v.clear();
    /// v.extend(iter.by_ref());
    /// assert_eq!(v, [4, 3]);
    /// ```
    ///
    /// Note: When first designing this trait, I wanted to return an iterator of
    /// iterators. However, I couldn't exercise the second iterator if the first
    /// iterator hasn't been fully evaluated. Still it may be useful to think of this
    /// as being like an iterator of iterators you just need to handle it
    /// carefully. The iterator will signal the end of the sequences with None:
    /// `a1, a2, ..., None, b1, b2, ..., None, c1, c2, ..., None, None`.
    ///
    /// We can use [`peekable()`][] to find out if our iterator is finished.
    ///
    /// ```
    /// # use asky::bevy::DivideAndSeparate;
    /// let mut iter = [1,2,3,4].into_iter().divide_and_separate(|x| {
    ///     if x % 2 == 1 { Ok(x) } else { Err((None, Some(x))) }
    /// })
    /// .peekable();
    /// let mut v: Vec<u32> = Vec::new();
    /// while iter.peek().is_some() {
    ///     v.extend(iter.by_ref());
    ///     v.push(0);
    /// }
    /// assert_eq!(v, [1,
    /// 0,
    ///                2, 3,
    /// 0,
    ///                4,
    /// 0]);
    /// ```
    ///
    fn divide_and_separate<U, F>(self, separator: F) -> DividedIterator<Self, Self::Item, U, F>
    where
        F: Fn(Self::Item) -> Result<U, (Option<U>, Option<U>)>;
}

pub struct DividedIterator<I, T, U, F>
    where
    I: Iterator<Item = T>,
    F: Fn(T) -> Result<U, (Option<U>, Option<U>)>,
    Self: Sized
{
    source_iter: I,
    predicate: F,
    buffer: Vec<Option<U>>,
}

impl<I, T, U, F> Iterator for DividedIterator<I, T, U, F>
where
    I: Iterator<Item = T>,
    F: Fn(T) -> Result<U, (Option<U>, Option<U>)>,
{
    type Item = U;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.len() > 0 {
            self.buffer.pop().unwrap()
        } else {
            match self.source_iter.next() {
                Some(x) =>
                    match (self.predicate)(x) {
                        Ok(z) => Some(z),
                        Err(a) => {
                            if a.0.is_some() && a.1.is_some() {
                                self.buffer.push(a.1);
                                self.buffer.push(None);
                                a.0
                            } else if a.0.is_some() && a.1.is_none() {
                                self.buffer.push(None);
                                a.0
                            } else if a.0.is_none() && a.1.is_some() {
                                self.buffer.push(a.1);
                                None
                            } else {
                                None
                            }
                        }
                    },
                None => None
            }
        }
    }
}

impl<I,T> DivideAndSeparate for I where I: Iterator<Item = T> {
    fn divide_and_separate<U, F>(self, f: F) -> DividedIterator<I, I::Item, U, F>
        where
        F: Fn(T) -> Result<U, (Option<U>, Option<U>)>,
    {
        DividedIterator {
            source_iter: self,
            predicate: f,
            buffer: Vec::new()
        }
    }
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
        let style = self.settings.style.clone().into();
        let white = text_style::Color::Ansi { color: AnsiColor::White, mode: AnsiMode::Dark };

        self.commands.entity(self.column).with_children(|column| {
            for lines in strings.0
                .into_iter()
                .map(StyledString::from)
                // .split('\n')
                // .into_iter()
                // .enumerate()
                // .map(|(i, mut colored_line)| {
                //     if self.state.cursor_visible && i == self.state.cursor_pos[1] {
                //         let mut length = 0;
                //         let mut inserted = false;
                //         for i in 0..colored_line.len() {
                //             if self.state.cursor_pos[0]
                //                 < length + colored_line[i].input.chars().count()
                //             {
                //                 // The cursor is in this one.
                //                 let part = colored_line.remove(i);
                //                 for (j, new_part) in BevyRenderer::cursorify(
                //                     part,
                //                     self.state.cursor_pos[0] - length,
                //                     white
                //                 )
                //                 .enumerate()
                //                 {
                //                     colored_line.insert(i + j, new_part)
                //                 }
                //                 inserted = true;
                //                 break;
                //             }
                //             length += colored_line[i].input.chars().count();
                //         }
                //         if !inserted && self.state.cursor_pos[0] >= length {
                //             // Cursor is actually one character past string.
                //             colored_line.push(" ".on(white));
                //         }
                //     }
                //     colored_line
                //         .0
                //         .into_iter()
                //         .map(|cs| BevyRenderer::build_text_bundle(cs, style.clone()))
                // })
            {
                column
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        // text_style::bevy::render_iter(parent, &style.into(), lines);
                        text_style::bevy::render(parent, &style, &lines);
                        // for line in lines {
                        //     parent.spawn(line);
                        // }
                    });
            }
        });
        Ok(())
    }

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
                    commands.entity(entity).remove_children(&children);
                    for child in children {
                        commands.entity(*child).despawn_recursive();
                    }
                }
            }
            AskyState::Waiting(_) | AskyState::Reading => {
                if !is_abort_key(&key_event) && !prompt.will_handle_key(&key_event) && render_state.draw_time != DrawTime::First
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
                            Err(e) => promise.reject(e)
                        }
                    }
                    render_state.draw_time = DrawTime::Last;
                }
                if let Some(children) = children {
                    commands.entity(entity).remove_children(&children);
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
         app.insert_resource(AskyParamConfig { state: Arc::new(Mutex::new(AskyParamState { closures: Vec::new() })) })
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
            .add_systems(Update, asky_system::<Select<'static, &'static str>>)
            .add_systems(Update, asky_system::<Password>)
            .add_systems(Update, asky_system::<Message>)
            .add_systems(Update, asky_system::<MultiSelect<'static, &'static str>>)
            .add_systems(Update, poll_tasks::<()>)
            .add_systems(Update, poll_tasks_err::<()>)
            .add_systems(Update, run_closures)
            .add_systems(Update, run_timers)
            ;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use super::divide_and_separate;
    // #[test]
    // fn test_divide_and_partition() {
    //     let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    //     let result: Vec<Vec<i32>> = divide_and_separate(data.into_iter(), |x| {
    //         if x % 2 == 0 {
    //             Ok(())
    //         } else {
    //             Err(())
    //         }
    //     })
    //     .collect();

    //     println!("{:?}", result);
    // }

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

    #[test]
    fn test_divide_and_separate() {
        let a: [u32; 4] = [1,2,3,4];
        // let mut iter = divide_and_separate(a.into_iter(), |x| if x % 2 == 1 { Ok(x) } else { Err((Some(2), None)) });
        let mut iter = a.into_iter().divide_and_separate(|x| if x % 2 == 1 { Ok(x) } else { Err((None, None)) });

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_divide_and_separate_left() {
        let a: [u32; 4] = [1,2,3,4];
        // let mut iter = divide_and_separate(a.into_iter(), |x| if x % 2 == 1 { Ok(x) } else { Err((Some(2), None)) });
        let mut iter = a.into_iter().divide_and_separate(|x| if x % 2 == 1 { Ok(x) } else { Err((Some(x), None)) });

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_divide_and_separate_right() {
        let a: [u32; 4] = [1,2,3,4];
        // let mut iter = divide_and_separate(a.into_iter(), |x| if x % 2 == 1 { Ok(x) } else { Err((Some(2), None)) });
        let mut iter = a.into_iter().divide_and_separate(|x| if x % 2 == 1 { Ok(x) } else { Err((None, Some(x))) });

        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }


    #[test]
    fn test_divide_and_separate_peekable() {
        let a: [u32; 4] = [1,2,3,4];
        // let mut iter = divide_and_separate(a.into_iter(), |x| if x % 2 == 1 { Ok(x) } else { Err((Some(2), None)) });
        let mut iter = a.into_iter().divide_and_separate(|x| if x % 2 == 1 { Ok(x) } else { Err((None, Some(x))) }).peekable();
        let mut v: Vec<u32> = Vec::new();

        while iter.peek().is_some() {
            v.extend(iter.by_ref());
            v.push(100);
        }
        assert_eq!(v, [1,100, 2,3, 100,4, 100]);
    }
}
