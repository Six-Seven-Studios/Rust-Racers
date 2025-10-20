mod map;
mod terrain;
mod car;
mod collisions;
mod camera;
mod credits;
mod title_screen;
mod lobby;
mod intro;
mod theta;
mod lap_system;
mod victory_screen;

use title_screen::{check_for_title_input, setup_title_screen};
use lobby::LobbyState;
use map::{load_map_from_file, GameMap, spawn_map};
use car::{Background, move_player_car, spawn_cars};
use camera::{move_camera, reset_camera_for_credits, WIN_W, WIN_H};
use credits::{check_for_credits_input, setup_credits, show_credits};
use victory_screen::setup_victory_screen;
use bevy::{prelude::*, window::PresentMode};
use bevy::render::camera::{Projection, ScalingMode};
use lap_system::{spawn_lap_triggers, LapCounter, update_laps};

use bevy::{color::palettes::basic::*, input_focus::InputFocus, prelude::*};
use crate::car::move_ai_cars;
// use bevy::render::

const TILE_SIZE: u32 = 64;  //Tentative

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Title,
    Lobby,
    Joining,
    Customizing,
    Settings,
    Playing,
    PlayingDemo,
    Victory,
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
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::Srgba(Srgba::WHITE)))
        .add_systems(OnEnter(GameState::Playing), load_map1)
        .add_systems(OnEnter(GameState::PlayingDemo), load_map_demo) // THETA* DEMO (but could support our second map)
        //.insert_resource(load_map_from_file("assets/big-map.txt")) // to get a Res handle on GameMap
        .insert_resource(load_map_from_file("assets/big-map.txt")) // to get a Res handle on GameMap
        .init_resource::<LobbyState>()
        .add_systems(Startup, (camera_setup, setup_title_screen))
        .add_systems(OnEnter(GameState::Playing), (car_setup, spawn_map, spawn_lap_triggers).after(load_map1))
        .add_systems(OnEnter(GameState::PlayingDemo), (car_setup, spawn_map).after(load_map_demo))
        // .add_systems(Startup, intro::setup_intro)
        // .add_systems(Update, intro::check_for_intro_input)
        .add_systems(Update, (
            check_for_title_input,
            check_for_credits_input,
            //move_car.run_if(in_state(GameState::Playing)),
            move_player_car.run_if(in_state(GameState::Playing).or(in_state(GameState::PlayingDemo))),
            //move_camera.after(move_car).run_if(in_state(GameState::Playing)),
            move_camera.after(move_player_car).run_if(in_state(GameState::Playing).or(in_state(GameState::PlayingDemo))),
            move_ai_cars.run_if(in_state(GameState::Playing).or(in_state(GameState::PlayingDemo))),
            update_laps.run_if(in_state(GameState::Playing)),
        ))
        .add_systems(OnEnter(GameState::Victory), setup_victory_screen)
        .add_systems(OnEnter(GameState::Credits), (reset_camera_for_credits, setup_credits))
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

fn load_map1(mut commands: Commands) {
    commands.insert_resource(load_map_from_file("assets/big-map.txt"));
}

//THETA* DEMO
fn load_map_demo(mut commands: Commands) {
    commands.insert_resource(load_map_from_file("assets/map_demo.txt"));
}