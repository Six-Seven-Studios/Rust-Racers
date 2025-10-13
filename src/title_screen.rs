use bevy::input::ButtonInput;
use bevy::prelude::*;
use crate::GameState;

use bevy::{color::palettes::basic::*, input_focus::InputFocus, prelude::*};


#[derive(Component)]
pub struct SettingsButton;

#[derive(Component)]
pub struct CreateButton;

#[derive(Component)]
pub struct JoinButton;

#[derive(Component)]
pub struct CustomizeButton;

#[derive(Component)]
pub struct Logo;

// #[derive(Component)]
// pub struct CreditsEntity;

// #[derive(Resource)]
// pub struct CreditsTimer(pub Timer);

// #[derive(Component, Deref, DerefMut)]
// pub struct PopupTimer(pub Timer);


pub fn check_for_title_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    // Default to get to gameplay state. DELETE LATER --------------------------
    if input.just_pressed(KeyCode::Space) && *current_state == GameState::Title {
        next_state.set(GameState::Playing);
    }
    // -------------------------------------------------------------------------

    else if input.just_pressed(KeyCode::Digit1) && *current_state == GameState::Title {
        println!("CREATE");
    }
    else if input.just_pressed(KeyCode::Digit2) && *current_state == GameState::Title {
        println!("JOIN");
    }
    else if input.just_pressed(KeyCode::Digit3) && *current_state == GameState::Title {
        println!("CUSTOMIZE");
    }
    else if input.just_pressed(KeyCode::Digit4) && *current_state == GameState::Title {
        println!("SETTINGS");
    }
    
}

pub fn setup_title_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>) {

    // commands.insert_resource(CreditsTimer(Timer::from_seconds(20.0, TimerMode::Once)));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/settingsGear.png")),
        Transform {
            translation: Vec3::new(-570., 300., 1.),
            ..default()
        },
        SettingsButton
    ));
    // // .observe(create::<Pointer<Press>>);

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/slantedButton.png")),
        Transform {
            translation: Vec3::new(0., -100., 1.),
            ..default()
        },
        CreateButton
    ));

    // commands.spawn((
    //     button(&asset_server, "Join Game"),
    //     Transform {
    //         translation: Vec3::new(0., -100., 1.),
    //         ..default()
    //     },
    //     CreateButton
    // ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/slantedButton.png")),
        Transform {
            translation: Vec3::new(0., -200., 1.),
            ..default()
        },
        JoinButton
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/slantedButton.png")),
        Transform {
            translation: Vec3::new(0., -300., 1.),
            ..default()
        },
        CustomizeButton
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/rustRacersLogo.png")),
        Transform {
            translation: Vec3::new(0., 100., 1.),
            ..default()
        },
        Logo
    ));
}