use bevy::{prelude::*, window::PresentMode};

#[derive(Component, Deref, DerefMut)]
struct PopupTimer(Timer);

const WIN_W: f32 = 1280.;
const WIN_H: f32 = 720.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rust Racers".into(),
                resolution: (WIN_W, WIN_H).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_credits)
        .add_systems(Update, show_credits)
        .run();
}

fn setup_credits(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite::from_image(asset_server.load("credits/rust-racers.png")));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/developed-by.png")),
        Transform {
            translation: Vec3::new(0., 0., -1.),
            ..default()
        },
        PopupTimer(Timer::from_seconds(2., TimerMode::Once)),
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/kameren-jouhal.png")),
        Transform {
            translation: Vec3::new(0., 0., -1.),
            ..default()
        },
        PopupTimer(Timer::from_seconds(4., TimerMode::Once)),
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/greyson-barsotti.png")),
        Transform {
            translation: Vec3::new(0., 0., -1.),
            ..default()
        },
        PopupTimer(Timer::from_seconds(6., TimerMode::Once)),
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/ethan-defilippi.png")),
        Transform {
            translation: Vec3::new(0., 0., -1.),
            ..default()
        },
        PopupTimer(Timer::from_seconds(8., TimerMode::Once)),
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/carson-gollinger.png")),
        Transform {
            translation: Vec3::new(0., 0., -1.),
            ..default()
        },
        PopupTimer(Timer::from_seconds(10., TimerMode::Once)),
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/jonathan-coulter.png")),
        Transform {
            translation: Vec3::new(0., 0., -1.),
            ..default()
        },
        PopupTimer(Timer::from_seconds(12., TimerMode::Once)),
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("credits/jeremy-luu.png")),
        Transform {
            translation: Vec3::new(0., 0., -1.),
            ..default()
        },
        PopupTimer(Timer::from_seconds(14., TimerMode::Once)),
    ));
}

fn show_credits(time: Res<Time>, mut popup: Query<(&mut PopupTimer, &mut Transform)>) {
    let mut counter = 2.;
    for (mut timer, mut transform) in popup.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            transform.translation.z += counter;
            counter += 1.;
        }
    }
}