use asky::bevy::*;

use asky::{Confirm, Message, Valuable};

use bevy::{prelude::*, window::PresentMode};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
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
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Startup, setup)
        .add_systems(Update, response)
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
    let node = NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    };
    commands.spawn(node.clone()).with_children(|parent| {
        parent.spawn(node).insert(Asky(confirm, AskyState::Reading));
    });
}

fn response(mut commands: Commands, mut query: Query<(Entity, &mut Asky<Confirm<'static>>)>) {
    for (entity, mut prompt) in query.iter_mut() {
        match prompt.1 {
            AskyState::Complete(0) => {
                // Mark the complete state someway so we don't repeat the same handling action.
                prompt.1 = AskyState::Complete(1);
                let response = match prompt.0.value() {
                    Ok(yes) => {
                        if yes {
                            "Great, me too."
                        } else {
                            "Oh, ok."
                        }
                    }
                    Err(_) => "Uh oh, had a problem.",
                };

                let child = commands
                    .spawn(NodeBundle { ..default() })
                    .insert(Asky(Message::new(response), AskyState::Reading))
                    .id();
                commands.entity(entity).push_children(&[child]);
            }
            _ => {}
        }
    }
}
