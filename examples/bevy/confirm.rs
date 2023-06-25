use asky::bevy::*;
use asky::utils::renderer::*;
use asky::{Confirm, MultiSelect, Number, Password, Select, Toggle};

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    input::keyboard::KeyboardInput,
    prelude::*,
    window::{CursorGrabMode, PresentMode, WindowLevel},
};

use asky::Typeable;

use asky::DrawTime;

use colored::{ColoredString, ColoredStrings, Colorize};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Bevy Asky Example".into(),
                    resolution: (600., 400.).into(),
                    present_mode: PresentMode::AutoVsync,
                    // Tells wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,
                    // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,
                    // window_theme: Some(WindowTheme::Dark),
                    ..default()
                }),
                ..default()
            }))

        // .add_plugins(DefaultPlugins)
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        //
        .add_startup_system(setup)
        .add_system(asky_system::<Confirm>)
        .add_system(asky_system::<Toggle>)
        .add_system(asky_system::<asky::Text>)
        .add_system(asky_system::<Number<u8>>)
        .add_system(asky_system::<Number<f32>>)
        .add_system(asky_system::<Select<'static, &'static str>>)
        .add_system(asky_system::<Password>)
        .add_system(asky_system::<MultiSelect<'static, &'static str>>)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let settings = BevyAskySettings {
        style: TextStyle {
            font: asset_server.load("fonts/DejaVuSansMono.ttf"),
            font_size: 50.0,
            color: Color::WHITE,
        },
    };
    commands.insert_resource(settings);
    // UI camera
    commands.spawn(Camera2dBundle::default());
    let confirm: Confirm<'static> = Confirm::new("Hi?");
    let toggle: Toggle<'static> = Toggle::new("Hi?", ["Bye", "What?"]);
    let text_input: asky::Text<'static> = asky::Text::new("Hi?");
    let number: Number<'static, u8> = Number::new("Number?");
    let float: Number<'static, f32> = Number::new("Float?");
    let select: Select<'static, &'static str> =
        Select::new("Favorite animal?", ["dog", "cow", "cat"]);
    let password: Password<'static> = Password::new("Password: ");

    let multi_select: MultiSelect<'static, &'static str> =
        MultiSelect::new("Favorite animal?", ["dog", "cow", "cat"]);

    // Text with multiple sections
    commands.spawn(
        // Create a TextBundle that has a Text with a list of sections.
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        }
    )
    // .insert(Asky(confirm))
    // .insert(Asky(toggle))
    // .insert(Asky(text_input))
    // .insert(Asky(number))
    // .insert(Asky(float))
    .insert(Asky(select, AskyState::Reading))
    // .insert(Asky(password))
    // .insert(Asky(multi_select))
        ;
}

fn asky_system<T: Printable + Typeable<KeyEvent> + Send + Sync + 'static>(
    mut commands: Commands,
    char_evr: EventReader<ReceivedCharacter>,
    mut key_evr: EventReader<KeyboardInput>,
    asky_settings: Res<BevyAskySettings>,
    mut render_state: Local<BevyRendererState>,
    // mut query: Query<&mut Text, With<Confirm>>) { // Compiler goes broke on this line.
    mut query: Query<(Entity, &mut Asky<T>, Option<&Children>)>,
) {
    let key_event = asky::bevy::KeyEvent::new(char_evr, key_evr);
    for (entity, mut confirm, children) in query.iter_mut() {
        match confirm.1 {
            AskyState::Complete => {
                continue;
            },
            AskyState::Hidden => {
                if children.is_some() {
                    let children: Vec<Entity> = children.map(|c| c.to_vec()).unwrap_or_else(Vec::new);
                    commands.entity(entity).remove_children(&children);
                    for child in children {
                        commands.entity(child).despawn_recursive();
                    }
                }
            },
            AskyState::Reading => {
                if confirm.handle_key(&key_event) {
                    // It's done.
                    confirm.1 = AskyState::Complete;
                    // commands.entity(entity).remove::<Asky<T>>();
                    let mut renderer =
                        BevyRenderer::new(&asky_settings, &mut render_state, &mut commands, entity);
                    renderer.update_draw_time();
                }

                let children: Vec<Entity> = children.map(|c| c.to_vec()).unwrap_or_else(Vec::new);
                commands.entity(entity).remove_children(&children);
                for child in children {
                    commands.entity(child).despawn_recursive();
                }
                let mut renderer =
                    BevyRenderer::new(&asky_settings, &mut render_state, &mut commands, entity);
                let draw_time = renderer.draw_time();
                confirm.draw(&mut renderer);
                if draw_time == DrawTime::First {
                    renderer.update_draw_time();
                } else if draw_time == DrawTime::Last {
                    render_state.clear();
                }
            }
        }
    }
}
