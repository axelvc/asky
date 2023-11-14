use asky::bevy::*;
use std::future::Future;
use asky::{Confirm, Message, Text};
use bevy::{prelude::*, window::PresentMode};

#[derive(Component)]
struct Page;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Asky Example".into(),
                resolution: (1000., 400.).into(),
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
        // .add_systems(Update, ask_question.pipe(future_sink))
        // .add_systems(Update, ask_question.pipe(option_future_sink))
        .add_systems(Update, ask_name.pipe(option_future_sink))
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
    let node = NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    };
    commands.spawn((node, Page));
}

fn ask_name(mut asky: Asky, query: Query<Entity, Added<Page>>) -> Option<impl Future<Output = ()>> {
    if let Ok(id) = query.get_single() {
        Some(async move {
            if let Ok(first_name) = asky.listen(Text::new("What's your first name? "), id).await {
                let _ = asky.clear(id).await;
                if let Ok(last_name) = asky.listen(Text::new("What's your last name? "), id).await {
                    let _ = asky.clear(id).await;
                    let _ = asky.listen(Message::new(format!("Hello, {first_name} {last_name}!")), id).await;
                }
            } else {
                eprintln!("Got err in ask name");
            }
        })
    } else {
        None
    }
}

fn ask_question(mut asky: Asky, query: Query<Entity, Added<Page>>) -> Option<impl Future<Output = ()>> {
    if let Ok(id) = query.get_single() {
        Some(async move {
            let confirm = Confirm::new("Do you like coffee?");
            let promise = asky.listen(confirm, id);
            let msg = match promise.await {
                Ok(yes) => {
                    if yes {
                        "Great, me too."
                    } else {
                        "Oh, ok."
                    }
                },
                Err(_) => "Uh oh, had a problem.",
            };
            let _ = asky.listen(Message::new(msg), id);
        })
    } else {
        None
    }
}

fn ask_question2(query: Query<Entity, Added<Page>>, mut asky: Asky) -> impl Future<Output = ()> {
    let id = query.get_single().expect("No Page");
    async move {
        let confirm = Confirm::new("Do you like coffee?");
        let promise = asky.listen(confirm, id);
        let msg = match promise.await {
            Ok(yes) => {
                if yes {
                    "Great, me too."
                } else {
                    "Oh, ok."
                }
            },
            Err(_) => "Uh oh, had a problem.",
        };
        println!("{}", msg);
    }
}
