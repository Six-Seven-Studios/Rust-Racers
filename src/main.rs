mod camera;
mod car;
mod car_state;
mod client_prediction;
mod credits;
mod drift_settings;
mod game_logic;
mod lobby;
mod multiplayer;
mod networking;
mod networking_plugin;
mod title_screen;
mod victory_screen;
mod interpolation;

use title_screen::{check_for_title_input, setup_title_screen, pause, sync_server_address, ServerAddress, check_for_lobby_input};
use lobby::{LobbyState, update_lobby_display, LobbyList, LobbyListDirty, populate_lobby_list};
use game_logic::{load_map_from_file, GameMap, spawn_map, CpuDifficulty, LapCounter, spawn_lap_triggers, update_laps};
use car::{Background, move_player_car, spawn_cars, move_ai_cars, ai_car_fsm};
use camera::{move_camera, reset_camera_for_credits, WIN_W, WIN_H};
use credits::{check_for_credits_input, setup_credits, show_credits};
use victory_screen::setup_victory_screen;
use bevy::{prelude::*, window::PresentMode, color::palettes::basic::*, input_focus::InputFocus};
use bevy::render::camera::{Projection, ScalingMode};
use networking_plugin::NetworkingPlugin;
use crate::game_logic::{AIControlled, Orientation, TILE_SIZE, ThetaCheckpointList, Velocity};


#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Title,
    Lobby,
    Creating,
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
        .add_plugins(
            DefaultPlugins
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
                }),
        )
        .add_plugins(NetworkingPlugin)
        .insert_resource(CpuDifficulty::default())
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(ServerAddress {
            address: String::new(),
        })
        .init_resource::<drift_settings::DriftSettings>()
        .init_resource::<client_prediction::InputSequence>()
        .init_resource::<client_prediction::InputBuffer>()
        .insert_resource(Time::<Fixed>::from_hz(60.0)) // 60 Hz fixed update (60fps for input/physics)
        .init_state::<GameState>()
        .add_systems(OnEnter(GameState::Playing), load_map1)
        .add_systems(OnEnter(GameState::PlayingDemo), load_map_demo) // THETA* DEMO (but could support our second map)
        //.insert_resource(load_map_from_file("assets/big-map.txt")) // to get a Res handle on GameMap
        .insert_resource(load_map_from_file("assets/big-map.txt")) // to get a Res handle on GameMap
        .init_resource::<LobbyState>()
        .init_resource::<LobbyList>()
        .init_resource::<LobbyListDirty>()
        .init_resource::<interpolation::InterpolationDelay>()
        .add_systems(Startup, (camera_setup, setup_title_screen))
        .add_systems(
            OnEnter(GameState::Playing),
            (car_setup, spawn_map, spawn_lap_triggers).after(load_map1),
        )
        .add_systems(
            OnEnter(GameState::PlayingDemo),
            (car_setup, spawn_map, spawn_lap_triggers).after(load_map_demo),
        )
        .add_systems(
            OnEnter(GameState::PlayingDemo),
            (ai_car_setup).after(car_setup),
        )
        // .add_systems(Startup, intro::setup_intro)
        // .add_systems(Update, intro::check_for_intro_input)
        .add_systems(
            Update,
            (
                sync_server_address,
                check_for_title_input,
                check_for_lobby_input,
                check_for_credits_input,
            ),
        )
        .add_systems(
            Update,
            title_screen::update_easy_drift_label.run_if(in_state(GameState::Settings)),
        )
        .add_systems(
            Update,
            (
                update_lobby_display.run_if(in_state(GameState::Lobby)),
                //move_car.run_if(in_state(GameState::Playing)),
                // Server now controls player physics, client just renders server position
                // Client only controls game state in GameState::PlayingDemo
                move_player_car.run_if(in_state(GameState::PlayingDemo)),
                //move_camera.after(move_car).run_if(in_state(GameState::Playing)),
                move_camera
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::PlayingDemo))),
                move_ai_cars
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::PlayingDemo))),
                ai_car_fsm.run_if(in_state(GameState::PlayingDemo)),
                update_laps
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::PlayingDemo))),
                interpolation::interpolate_networked_cars.run_if(in_state(GameState::Playing)),
                populate_lobby_list.run_if(in_state(GameState::Joining)),
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                // Client-side prediction and reconciliation run at fixed 30 Hz
                client_prediction::send_keyboard_input.run_if(in_state(GameState::Playing)),
                multiplayer::get_car_positions.run_if(in_state(GameState::Playing)),
            )
                .chain(),
        )
        .add_systems(OnEnter(GameState::Victory), setup_victory_screen)
        .add_systems(
            OnEnter(GameState::Credits),
            (reset_camera_for_credits, setup_credits),
        )
        .add_systems(Update, show_credits.run_if(in_state(GameState::Credits)))
        .add_systems(Update, pause)
        .run();
}

fn camera_setup(mut commands: Commands) {
    // create a projection
    let mut projection = OrthographicProjection::default_2d();

    // modify the fields
    projection.scaling_mode = ScalingMode::WindowSize;
    projection.scale = 1.0;

    // spawn with the custom projection
    commands
        .spawn(Camera2d::default())
        .insert(Projection::Orthographic(projection));
}

fn car_setup(
    commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    state: Res<State<GameState>>,
) {
    // spawn_cars now detects the game mode and spawns accordingly
    // - Playing (multiplayer): Only player car
    // - PlayingDemo: Player car + AI car
    spawn_cars(commands, asset_server, texture_atlases, state);
}
fn ai_car_setup(
    mut ai_cars: Query<(&mut ThetaCheckpointList), (With<AIControlled>, Without<Background>)>,
) {
    for (mut theta_checkpoint_list) in ai_cars.iter_mut() {
        *theta_checkpoint_list = theta_checkpoint_list.load_checkpoint_list(1);
    }
}

fn load_map1(mut commands: Commands) {
    commands.insert_resource(load_map_from_file("assets/big-map.txt"));
}

//THETA* DEMO
fn load_map_demo(mut commands: Commands) {
    commands.insert_resource(load_map_from_file("assets/big-map.txt"));
}

// map2
fn load_map2(mut commands: Commands) {
    commands.insert_resource(load_map_from_file("assets/map2.txt"));
}
