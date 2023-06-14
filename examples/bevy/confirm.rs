use asky::Confirm;

// fn main() -> std::io::Result<()> {
//     if Confirm::new("Do you like coffe?").prompt()? {
//         println!("Great, me too!");
//     }

//     // ...

//     Ok(())
// }

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    input::keyboard::KeyboardInput,
};

use asky::bevy::ColoredBuilder;
use asky::Typeable;

use asky::DrawTime;

use colored::{Colorize, ColoredString, ColoredStrings};

fn main() {
    App::new()
        // .insert_resource(ColoredBuilder { style:
        //     TextStyle {
        //         font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        //         font_size: 100.0,
        //         color: Color::WHITE,
        //     },
        // })
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_startup_system(setup)
        .add_system(text_update_system)
        .add_system(text_color_system)
        .add_system(asky_confirm_system)
        .run();
}

// A unit struct to help identify the FPS UI component, since there may be many Text components
#[derive(Component)]
struct FpsText;

// A unit struct to help identify the color-changing Text component
#[derive(Component)]
struct ColorText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // UI camera
    commands.spawn(Camera2dBundle::default());
    // Text with one section
    commands.spawn((
        // Create a TextBundle that has a Text with a single section.
        TextBundle::from_section(
            // Accepts a `String` or any type that converts into a `String`, such as `&str`
            "hello\nbevy!",
            TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 100.0,
                color: Color::WHITE,
            },
        ) // Set the alignment of the Text
        .with_text_alignment(TextAlignment::Center)
        // Set the style of the TextBundle itself.
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(5.0),
                right: Val::Px(15.0),
                ..default()
            },
            ..default()
        }),
        ColorText,
    ));
    // Text with multiple sections
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 60.0,
                color: Color::GOLD,
            }),
        ]),
        FpsText,
    ));
    let confirm: Confirm<'static> = Confirm::new("Hi?");

    // Text with multiple sections
    commands.spawn((
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "Confirm",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
            ),
        ]),
        //Confirm::new("Do you like coffee?")
    ))
    .insert(confirm)
        ;
}

fn asky_confirm_system(
    mut commands: Commands,
    char_evr: EventReader<ReceivedCharacter>,
    keys: Res<Input<KeyCode>>,
    mut key_evr: EventReader<KeyboardInput>,
    asset_server: Res<AssetServer>,
    // mut query: Query<&mut Text, With<Confirm>>) { // Compiler goes broke on this line.
    mut query: Query<(Entity, &mut Text, &mut Confirm<'static>)>) {

    let builder = ColoredBuilder { style:
        TextStyle {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 100.0,
            color: Color::WHITE,
        },
    };

    let key_event = asky::bevy::KeyEvent::new(char_evr, &keys, key_evr);
    'outer: for (entity, mut text, mut confirm) in query.iter_mut() {
        for key in key_event.key_codes.iter() {
            if confirm.handle_key(*key) {
                // It's done.
                // let mut out = ColoredStrings::default();
                // builder.to_text(out, &mut text);
                // XXX: text.sections.clear() doesn't work.
                text.sections.clear();
                // text.sections.push(TextSection::new("waaah".to_owned(), builder.style.clone()));
                commands.entity(entity).remove::<Confirm<'static>>();
                // commands.entity(entity).remove::<Text>();
                continue 'outer;
            }
        }
        // if confirm.active {

        let mut out = ColoredStrings::default();
        // confirm.formatter.format(confirm, renderer.draw_time(), &mut out);
        confirm.formatter.format(&confirm, DrawTime::Update, &mut out);
        builder.to_text(out, &mut text);
        // }
    }

}

fn text_color_system(time: Res<Time>, mut query: Query<&mut Text, With<ColorText>>) {
    for mut text in &mut query {
        let seconds = time.elapsed_seconds();

        // Update the color of the first and only section.
        text.sections[0].style.color = Color::Rgba {
            red: (1.25 * seconds).sin() / 2.0 + 0.5,
            green: (0.75 * seconds).sin() / 2.0 + 0.5,
            blue: (0.50 * seconds).sin() / 2.0 + 0.5,
            alpha: 1.0,
        };
    }
}

fn text_update_system(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in &mut query {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                // Update the value of the second section
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}
