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

    let car_sheet_handle = asset_server.load("car.png");
    let car_layout = TextureAtlasLayout::from_grid(UVec2::splat(CAR_SIZE), 2, 2, None, None);
    let car_layout_handle = texture_atlases.add(car_layout);

    // Spawn player car
    commands.spawn((
        Sprite::from_atlas_image(
            car_sheet_handle.clone(),
            TextureAtlas {
                layout: car_layout_handle.clone(),
                index: 0,
            },
        ),
        Transform {
            translation: Vec3::new(0., 0., 50.),
            ..default()
        },
        Velocity::new(),
        Orientation::new(0.0),
        Car,
        PlayerControlled,
    ));

    // Spawn second car
    commands.spawn((
        Sprite::from_atlas_image(
            car_sheet_handle,
            TextureAtlas {
                layout: car_layout_handle,
                index: 0,
            },
        ),
        Transform {
            translation: Vec3::new(200., 200., 50.),
            ..default()
        },
        Velocity::new(),
        Orientation::new(1.57), 
        Car,
    ));
}

fn move_car(
    game_map: Res<GameMap>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    player_car: Single<(&mut Transform, &mut Velocity, &mut Orientation), (With<PlayerControlled>, Without<Background>)>,
    other_cars: Query<&Transform, (With<Car>, Without<PlayerControlled>)>,
) {
    let (mut transform, mut velocity, mut orientation) = player_car.into_inner();

    let deltat = time.delta_secs();
    let accel = ACCEL_RATE * deltat;

    // Get the current tile
    let pos = transform.translation.truncate();
    let tile_id = game_map.get_tile(pos.x, pos.y, TILE_SIZE as f32) as usize;
    let terrain = &TILES[tile_id];
    
    // Modifiers from terrain
    let fric_mod  = terrain.friction_modifier;
    let speed_mod = terrain.speed_modifier;
    let turn_mod  = terrain.turn_modifier;

    // Turning
    if input.pressed(KeyCode::KeyA) {
        orientation.angle += TURNING_RATE * deltat * turn_mod;
    }
    if input.pressed(KeyCode::KeyD) {
        orientation.angle -= TURNING_RATE * deltat * turn_mod;
    }

    // Accelerate forward in the direction of car orientation
    if input.pressed(KeyCode::KeyW) {
        let forward = orientation.forward_vector() * accel;
        **velocity += forward;
        **velocity = velocity.clamp_length_max(PLAYER_SPEED);
        **velocity *= speed_mod;
    }

}   
