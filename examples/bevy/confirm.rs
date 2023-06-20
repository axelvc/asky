use asky::{Confirm, Toggle, Number, Select, Password, MultiSelect};
use asky::bevy::*;
use asky::utils::renderer::*;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    input::keyboard::KeyboardInput,
};

use asky::Typeable;

use asky::DrawTime;

use colored::{Colorize, ColoredString, ColoredStrings};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
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

    let settings = BevyAskySettings { style:
        TextStyle {
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
    let select: Select<'static, &'static str> = Select::new("Favorite animal?", ["dog", "cow", "cat"]);
    let password: Password<'static> = Password::new("Password: ");

    let multi_select: MultiSelect<'static, &'static str> = MultiSelect::new("Favorite animal?", ["dog", "cow", "cat"]);

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
        //Confirm::new("Do you like coffee?")
    )
    // .insert(Asky(confirm))
    // .insert(Asky(toggle))
    // .insert(Asky(text_input))
    // .insert(Asky(number))
    // .insert(Asky(float))
    // .insert(Asky(select))
    // .insert(Asky(password))
    .insert(Asky(multi_select))
        ;
}

fn asky_system<T: Printable + for<'a> Typeable<KeyEvent> + Send + Sync + 'static>(
    mut commands: Commands,
    char_evr: EventReader<ReceivedCharacter>,
    mut key_evr: EventReader<KeyboardInput>,
    asky_settings: Res<BevyAskySettings>,
    mut render_state: Local<BevyRendererState>,
    // mut query: Query<&mut Text, With<Confirm>>) { // Compiler goes broke on this line.
    mut query: Query<(Entity, &mut Asky<T>, Option<&Children>)>) {

    let key_event = asky::bevy::KeyEvent::new(char_evr, key_evr);
    'outer: for (entity, mut confirm, children) in query.iter_mut() {
        if confirm.handle_key(&key_event) {
            // It's done.
            commands.entity(entity).remove::<Asky<T>>();
            let mut renderer = BevyRenderer::new(&asky_settings, &mut render_state);
            renderer.update_draw_time();
        }
        let mut renderer = BevyRenderer::new(&asky_settings, &mut render_state);
        let draw_time = renderer.draw_time();
        confirm.draw(&mut renderer);
        let children: Vec<Entity> = children.map(|c| c.to_vec()).unwrap_or_else(Vec::new);
        commands.entity(entity).remove_children(&children);
        for child in children {
            commands.entity(child).despawn();
        }
        let new_children: Vec<Entity> = renderer.children.drain(0..).map(|b| commands.spawn(b).id()).collect();
        commands.entity(entity).push_children(&new_children);
        if draw_time == DrawTime::First {
            renderer.update_draw_time();
        } else if draw_time == DrawTime::Last {
            render_state.clear();
        }
    }
}

