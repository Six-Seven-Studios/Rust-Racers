use bevy::prelude::*;
use crate::GameState;

// Credits-related components and resources
#[derive(Component, Deref, DerefMut)]
pub struct PopupTimer(pub Timer);

#[derive(Component)]
pub struct CreditsEntity;

// Check for input to transition to credits
pub fn check_for_credits_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    if input.just_pressed(KeyCode::KeyP) && *current_state == GameState::Playing {
        next_state.set(GameState::Credits);
    }
}

// Setup the credits screen
pub fn setup_credits(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
) {

    commands.spawn((
        Sprite::from_image(asset_server.load("credits/rust-racers.png")),
        Transform {
            translation: Vec3::new(0., 0., -10.9),
            ..default()
        },
        PopupTimer(Timer::from_seconds(0., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/developed-by.png")),
        Transform {
            translation: Vec3::new(0., 0., -10.8),
            ..default()
        },
        PopupTimer(Timer::from_seconds(2., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/kameren-jouhal.png")),
        Transform {
            translation: Vec3::new(0., 0., -10.7),
            ..default()
        },
        PopupTimer(Timer::from_seconds(4., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/greyson-barsotti.png")),
        Transform {
            translation: Vec3::new(0., 0., -10.6),
            ..default()
        },
        PopupTimer(Timer::from_seconds(6., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/ethan-defilippi.png")),
        Transform {
            translation: Vec3::new(0., 0., -10.5),
            ..default()
        },
        PopupTimer(Timer::from_seconds(8., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/carson-gollinger.png")),
        Transform {
            translation: Vec3::new(0., 0., -10.4),
            ..default()
        },
        PopupTimer(Timer::from_seconds(10., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/jonathan-coulter.png")),
        Transform {
            translation: Vec3::new(0., 0., -10.3),
            ..default()
        },
        PopupTimer(Timer::from_seconds(12., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/jeremy-luu.png")),
        Transform {
            translation: Vec3::new(0., 0., -10.2),
            ..default()
        },
        PopupTimer(Timer::from_seconds(14., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/david-shi.png")),
        Transform {
            translation: Vec3::new(0., 0., -10.1),
            ..default()
        },
        PopupTimer(Timer::from_seconds(16., TimerMode::Once)),
        CreditsEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/Daniel.png")),
        Transform {
            translation: Vec3::new(0., 0., -10.),
            ..default()
        },
        PopupTimer(Timer::from_seconds(18., TimerMode::Once)),
        CreditsEntity,
    ));
}

// Show credits animation
pub fn show_credits(
    time: Res<Time>, 
    mut popup: Query<(&mut PopupTimer, &mut Transform), With<CreditsEntity>>,
) {
    let mut counter = 100.;
    
    for (mut timer, mut transform) in popup.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            transform.translation.z += counter;
        }
    }
}
