mod map;
mod terrain;
mod car;
mod camera;
mod credits;
mod title_screen;
mod server;
mod get_ip;
mod networking;

use title_screen::{check_for_title_input, setup_title_screen};
use map::{load_map_from_file, GameMap, spawn_map};
use car::{Background, move_car, spawn_cars};
use camera::{move_camera, reset_camera_for_credits, WIN_W, WIN_H};
use credits::{check_for_credits_input, setup_credits, show_credits};
use bevy::{prelude::*, window::PresentMode};
use bevy::render::camera::{Projection, ScalingMode};
use server::ServerPlugin;
use networking::NetworkingPlugin;

const TILE_SIZE: u32 = 64;  //Tentative

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Title,
    Playing,
    Credits,
}

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
        .add_plugins(NetworkingPlugin)
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::Srgba(Srgba::WHITE)))
        .insert_resource(load_map_from_file("assets/map.txt")) // to get a Res handle on GameMap
        .add_systems(Startup, (camera_setup, setup_title_screen))
        .add_systems(OnEnter(GameState::Playing), (car_setup, spawn_map))
        .add_systems(Update, (
            check_for_title_input,
            check_for_credits_input,
            move_car.run_if(in_state(GameState::Playing)),
            move_camera.after(move_car).run_if(in_state(GameState::Playing)),
        ))
        .add_systems(OnEnter(GameState::Credits), setup_credits)
        .add_systems(OnEnter(GameState::Credits), reset_camera_for_credits.after(setup_credits))
        .add_systems(Update, show_credits.run_if(in_state(GameState::Credits)))
        .run();
}
fn camera_setup(mut commands: Commands)
{
    // create a projection
    let mut projection = OrthographicProjection::default_2d();

    // modify the fields
    projection.scaling_mode = ScalingMode::WindowSize;
    projection.scale = 1.0;

    // spawn with the custom projection
    commands.spawn(Camera2d::default())
        .insert(Projection::Orthographic(projection));
}

fn car_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlasLayout>>)
{
    // Spawn cars using the car module
    spawn_cars(commands, asset_server, texture_atlases);
}
