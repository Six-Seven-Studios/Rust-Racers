mod camera;
mod car;
mod car_skins;
mod car_state;
mod client_prediction;
mod credits;
mod drift_settings;
mod game_logic;
mod interpolation;
mod lobby;
mod multiplayer;
mod networking;
mod networking_plugin;
mod speed;
mod title_screen;
mod victory_screen;

use speed::{
    SpeedBoost, SpeedPowerup, collect_powerups, remove_boost_ui, spawn_boost_ui,
    spawn_speed_powerups, update_speed_boost,
};

use crate::game_logic::{AIControlled, Orientation, TILE_SIZE, ThetaCheckpointList, Velocity, MapLevelData};
use bevy::render::camera::{Projection, ScalingMode};
use bevy::{color::palettes::basic::*, input_focus::InputFocus, prelude::*, window::PresentMode};
use camera::{WIN_H, WIN_W, move_camera, reset_camera_for_credits};
use car::{Background, ai_car_fsm, move_ai_cars, move_player_car, spawn_cars};
use credits::{check_for_credits_input, setup_credits, show_credits};
use game_logic::{
    CpuDifficulty, GameMap, LapCounter, load_map_from_file, spawn_lap_triggers, spawn_map,
    update_laps,
};
use lobby::{LobbyList, LobbyListDirty, LobbyState, populate_lobby_list, update_lobby_display};
use networking_plugin::NetworkingPlugin;
use networking::SelectedMap;
use title_screen::{
    ServerAddress, check_for_lobby_input, check_for_title_input, pause, setup_title_screen,
    sync_server_address,
};
use victory_screen::setup_victory_screen;

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
        .init_resource::<car_skins::CarSkinSelection>()
        .init_resource::<networking::SelectedMap>()
        .init_resource::<title_screen::IpTypingMode>()
        .insert_resource(CpuDifficulty::default())
        .insert_resource(ClearColor(Color::WHITE))
        .insert_resource(ServerAddress {
            address: "167.172.23.173".to_string(),
        })
        .init_resource::<drift_settings::DriftSettings>()
        .init_resource::<client_prediction::InputSequence>()
        .init_resource::<client_prediction::InputBuffer>()
        .init_resource::<MapLevelData>()
        .insert_resource(Time::<Fixed>::from_hz(60.0)) // 60 Hz fixed update (60fps for input/physics)
        .init_state::<GameState>()
        .add_systems(OnEnter(GameState::Playing), load_map1)
        .add_systems(OnEnter(GameState::PlayingDemo), load_map1) // THETA* DEMO uses map 1 (which has checkpoints defined)
        //.insert_resource(load_map_from_file("assets/big-map.txt")) // to get a Res handle on GameMap
        .insert_resource(load_map_from_file("assets/big-map.txt")) // to get a Res handle on GameMap
        .init_resource::<LobbyState>()
        .init_resource::<LobbyList>()
        .init_resource::<LobbyListDirty>()
        .init_resource::<interpolation::InterpolationDelay>()
        .add_systems(Startup, (camera_setup, setup_title_screen))
        .add_systems(
            OnEnter(GameState::Playing),
            (initialize_theta_grid, car_setup, spawn_map, spawn_lap_triggers).chain().after(load_map1),
        )
        .add_systems(Startup, camera_setup)
        .add_systems(OnEnter(GameState::Title), setup_title_screen)
        .add_systems(
            OnEnter(GameState::PlayingDemo),
            (initialize_theta_grid, car_setup, spawn_map, spawn_lap_triggers).chain().after(load_map1),
        )
        .add_systems(
            OnEnter(GameState::PlayingDemo),
            (ai_car_setup).after(car_setup),
        )
        // .add_systems(Startup, intro::setup_intro)
        // .add_systems(Update, intro::check_for_intro_input)
        .add_systems(Update, sync_server_address)
        .add_systems(Update, check_for_title_input)
        .add_systems(Update, check_for_lobby_input)
        .add_systems(Update, check_for_credits_input)
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
        // .add_systems(
        //     OnEnter(GameState::PlayingDemo),
        //     spawn_speed_powerups,
        // )
        .add_systems(
            Update,
            (
                spawn_speed_powerups,
                collect_powerups,
                update_speed_boost,
                spawn_boost_ui,
                remove_boost_ui,
            )
                .run_if(in_state(GameState::PlayingDemo).or(in_state(GameState::Playing))),
        )
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
    map_data: Res<MapLevelData>,
    skin_selection: Res<car_skins::CarSkinSelection>,
) {
    // spawn_cars now detects the game mode and spawns accordingly
    // - Playing (multiplayer): Only player car
    // - PlayingDemo: Player car + AI car
    spawn_cars(
        commands,
        asset_server,
        texture_atlases,
        state,
        skin_selection,
    );
}
fn ai_car_setup(
    mut ai_cars: Query<(&mut ThetaCheckpointList), (With<AIControlled>, Without<Background>)>,
) {
    for (mut theta_checkpoint_list) in ai_cars.iter_mut() {
        *theta_checkpoint_list = theta_checkpoint_list.load_checkpoint_list(1);
    }
}

// Merged Function: Loads map file AND sets up MapLevelData based on selection
fn load_selected_map(mut commands: Commands, selected_map: Res<SelectedMap>) {
    let map_path = selected_map.choice.path();
    commands.insert_resource(load_map_from_file(map_path));

    // Define data for Map 1 (Default/Big Map)
    let map1_data = MapLevelData {
        start_position: Vec3::new(0.0, 0.0, 5.0),
        finish_line_pos: Vec3::new(2752., 960., 5.),
        checkpoints: vec![
            (Vec3::new(2752., 1500., 10.), 0.0),
            (Vec3::new(2700., 2700., 10.), std::f32::consts::PI / 4.0),
            (Vec3::new(425., 2725., 10.), std::f32::consts::PI / -4.0),
            (Vec3::new(-1600., 400., 10.), std::f32::consts::PI / -4.0),
            (Vec3::new(-2044., -1493., 10.), 0.0),
            (Vec3::new(-1979., -2750., 10.), std::f32::consts::PI / 2.0),
            (Vec3::new(1515., -2750., 10.), std::f32::consts::PI / 2.0),
            (Vec3::new(2100., -150., 10.), 0.0),
        ],
    };

    // Define data for Map 2
    let map2_data = MapLevelData {
        start_position: Vec3::new(1300.0, -1131.0, 5.0), 
        finish_line_pos: Vec3::new(1300.0, -1131.0, 5.0), 
        checkpoints: vec![
            (Vec3::new(1386., 974., 10.), 0.0),
            (Vec3::new(3175., 1949., 10.), std::f32::consts::PI / 4.0),
            (Vec3::new(-1891., 2167., 10.), std::f32::consts::PI / -4.0),
            (Vec3::new(-471., 2146., 10.), std::f32::consts::PI / -4.0),
            (Vec3::new(862., 1907., 10.), 0.0),
            (Vec3::new(-1834., 30., 10.), std::f32::consts::PI / 2.0),
            (Vec3::new(-2841., 2059., 10.), 0.0),
            (Vec3::new(-3738., 1465., 10.), 0.0),
            (Vec3::new(-91., -2441., 10.), 0.0),
            (Vec3::new(3117., -2376., 10.), 0.0),
        ],
    };

    // Determine which data to inject
    if map_path.contains("map2") {
        commands.insert_resource(map2_data);
    } else {
        commands.insert_resource(map1_data);
    }
}

// Map 2 Loader for PlayingDemo state
fn load_map2(mut commands: Commands) {
    // load grid
    commands.insert_resource(load_map_from_file("assets/map2.txt"));
}

// Initialize ThetaGrid from GameMap for pathfinding
fn initialize_theta_grid(mut commands: Commands, game_map: Res<GameMap>) {
    use game_logic::theta_grid::ThetaGrid;
    let theta_grid = ThetaGrid::create_theta_grid(&game_map, TILE_SIZE as f32);
    commands.insert_resource(theta_grid);
}

    // define triggers and positions
    let map2_data = MapLevelData {
        start_position: Vec3::new(1300.0, -1131.0, 5.0), 
        finish_line_pos: Vec3::new(1300.0, -1131.0, 5.0), 
        checkpoints: vec![
            (Vec3::new(1386., 974., 10.), 0.0),
            (Vec3::new(3175., 1949., 10.), std::f32::consts::PI / 4.0),
            (Vec3::new(-1891., 2167., 10.), std::f32::consts::PI / -4.0),
            (Vec3::new(-471., 2146., 10.), std::f32::consts::PI / -4.0),
            (Vec3::new(862., 1907., 10.), 0.0),
            (Vec3::new(-1834., 30., 10.), std::f32::consts::PI / 2.0),
            (Vec3::new(-2841., 2059., 10.), 0.0),
            (Vec3::new(-3738., 1465., 10.), 0.0),
            (Vec3::new(-91., -2441., 10.), 0.0),
            (Vec3::new(3117., -2376., 10.), 0.0),
        ],
    };

    commands.insert_resource(map2_data);
}
