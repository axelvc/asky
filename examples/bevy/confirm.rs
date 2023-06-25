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
        .add_plugin(AskyPlugin)
        .add_startup_system(setup)
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
    .insert(Asky(confirm, AskyState::Reading))
    // .insert(Asky(toggle))
    // .insert(Asky(text_input))
    // .insert(Asky(number))
    // .insert(Asky(float))
    // .insert(Asky(select, AskyState::Reading))
    // .insert(Asky(password))
    // .insert(Asky(multi_select))
        ;
}

