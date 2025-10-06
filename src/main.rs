mod map;
mod terrain;
mod car;
mod camera;
mod credits;
mod server;

use map::{load_map_from_file, GameMap, spawn_map};
use car::{Background, move_car, spawn_cars};
use camera::{move_camera, reset_camera_for_credits, WIN_W, WIN_H};
use credits::{GameState, check_for_credits_input, setup_credits, show_credits};
use bevy::{prelude::*, window::PresentMode};
use bevy::render::camera::{Projection, ScalingMode};
use server::ServerPlugin;

const TILE_SIZE: u32 = 64;  //Tentative

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Rust Racers".into(),
                    resolution: (WIN_W, WIN_H).into(),
                    present_mode: PresentMode::AutoVsync,
                    resizable: false, // making the window not resizable for now, since resizing it causes some tiling issues
                    
                    ..default()
                }),
            ..default()
        }))
        .add_plugins(ServerPlugin)
        .init_state::<GameState>()
        .insert_resource(load_map_from_file("assets/map.txt")) // to get a Res handle on GameMap
        .add_systems(Startup, (setup, spawn_map))
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
    // create a projection
    let mut projection = OrthographicProjection::default_2d();

    // modify the fields
    projection.scaling_mode = ScalingMode::WindowSize;
    projection.scale = 1.0;

    // spawn with the custom projection
    commands.spawn(Camera2d::default())
        .insert(Projection::Orthographic(projection));
    
    // Spawn cars using the car module
    spawn_cars(commands, asset_server, texture_atlases);

}   
