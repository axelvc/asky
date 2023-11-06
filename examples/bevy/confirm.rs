use asky::bevy::*;

use asky::{Confirm, MultiSelect, Number, Password, Select, Toggle};

use bevy::{
    prelude::*,
    window::{PresentMode},
};

#[derive(Component)]
struct Page;

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
        .add_plugins(AskyPlugin)
        .add_systems(Startup, setup)
        // .add_systems(Update, response)
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
    commands.spawn(Camera2dBundle::default());

    let confirm: Confirm<'static> = Confirm::new("Do you like coffee?");
    let node =
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        };
    commands.spawn((node.clone(), Page))
        .with_children(|parent| {
        parent.spawn(node).insert(Asky(confirm, AskyState::Reading));
        }
    );
}

// fn setup(mut commands: Commands,
//          query: Query<(Asky<Confirm>, &Parent)>) {

// }
