mod map;
mod terrain;
mod car;
mod camera;
mod credits;

use map::{load_map_from_file};
use car::{Background, move_car, spawn_cars};
use camera::{move_camera, reset_camera_for_credits, WIN_W, WIN_H};
use credits::{GameState, check_for_credits_input, setup_credits, show_credits};
use bevy::{prelude::*, window::PresentMode};

const TILE_SIZE: u32 = 64;  //Tentative

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
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            check_for_credits_input,
            move_car.run_if(in_state(GameState::Playing)),
            move_camera.after(move_car).run_if(in_state(GameState::Playing)),
        ))
        .add_systems(OnEnter(GameState::Credits), setup_credits)
        .add_systems(OnEnter(GameState::Credits), reset_camera_for_credits.after(setup_credits))
        .add_systems(Update, show_credits.run_if(in_state(GameState::Credits)))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlasLayout>>)
{
    commands.spawn(Camera2d);

    let track_texture_handle = asset_server.load("track.png");

    commands.spawn((
        Sprite::from_image(track_texture_handle.clone()),
        Transform::from_translation(Vec3::ZERO),
        Background,
    ));

    let game_map = load_map_from_file("assets/map.txt");
    commands.insert_resource(game_map);

    // Spawn cars using the car module
    spawn_cars(commands, asset_server, texture_atlases);
}
