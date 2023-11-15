use asky::bevy::*;
use std::future::Future;
use asky::{Confirm, Message, Text, Select, Error, MultiSelect, Password};
use bevy::{prelude::*, window::PresentMode, utils::Duration};

#[derive(Component)]
struct Page;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Asky Example".into(),
                resolution: (500., 500.).into(),
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
        // .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Startup, setup)
        // .add_systems(Update, ask_question.pipe(future_sink))
        // .add_systems(Update, ask_question.pipe(option_future_sink))
        .add_systems(Startup, ask_user.pipe(future_sink))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let settings = BevyAskySettings {
        style: TextStyle {
            font: asset_server.load("fonts/DejaVuSansMono.ttf"),
            font_size: 25.0,
            color: Color::WHITE,
        },
    };
    commands.insert_resource(settings);
    commands.spawn(Camera2dBundle::default());
}

fn ask_user(mut asky: Asky, mut commands: Commands) -> impl Future<Output = Result<(), Error>> {
    let node = NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    };
    let id = commands.spawn(node).id();
    async move {
        let yes = asky.listen(Confirm::new("Want to see something cool?"), id).await?;
        let _ = asky.listen(Message::new(if yes
                                         { "Oh, good!" }
                                         else
                                         { "Oh, nevermind." }), id).await?;
        if ! yes { return Ok(()); }
        let lang = asky.listen(Select::new("Which do you prefer?",
                                           ["brainfuck", "rust", "x86 machine code"]), id).await?;
        let _ = asky.listen(Message::new(if lang == 1
                                         { "Me too!" }
                                         else
                                         { "More power to you." }), id).await?;
        let bitfield = asky.listen(MultiSelect::new("What engines do you use?",
                                                    ["Unity", "Unreal", "Godot", "bevy"]), id).await?;
        let _ = asky.listen(Message::new(if bitfield & 0b1000 != 0
                                         { "Well, have I got news for you!" }
                                         else
                                         { "Those are also great." }), id).await?;
        let _ = asky.listen(Message::new("The asky lib works for bevy now!"), id).await?;
        let _ = asky.listen(Message::new("So..."), id).await?;
        let _ = asky.listen(Confirm::new("Let's sign you up on our email list."), id).await?;
        let email = asky.listen(Text::new("What's your email?"), id).await?;
        let password = match asky.listen(Password::new("I'm gonna need your password too."), id).await {
            Ok(p) =>
                {
                    let _ = asky.listen(Message::new("Heh heh."), id).await?;
                    p
                }
            Err(_) =>
                asky.listen(Password::new("Please, I need it for real."), id).await?
        };
        let _ = asky.listen(Message::new("Just kidding."), id).await?;
        let _ = asky.listen(Message::new("I don't NEED your password."), id).await?;
        let _ = asky.listen(Message::new("I just wanted it for REASONS."), id).await?;
        Ok::<(), Error>(())
    }
}

