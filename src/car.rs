use bevy::prelude::*;
use crate::map::GameMap;
use crate::terrain::TILES;
use crate::TILE_SIZE;

// Car-related constants
pub const PLAYER_SPEED: f32 = 350.;
pub const ACCEL_RATE: f32 = 700.;
pub const FRICTION: f32 = 0.95;
pub const TURNING_RATE: f32 = 3.5;
pub const CAR_SIZE: u32 = 64;

// Car-related components
#[derive(Component)]
pub struct Car;

#[derive(Component)]
pub struct PlayerControlled;

#[derive(Component)]
pub struct Background;

#[derive(Component)]
pub struct Orientation {
    pub angle: f32,
}

impl Orientation {
    pub fn new(angle: f32) -> Self {
        Self { angle }
    }
    
    pub fn forward_vector(&self) -> Vec2 {
        Vec2::new(self.angle.cos(), self.angle.sin())
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct Velocity {
    pub velocity: Vec2,
}

impl Velocity {
    pub fn new() -> Self {
        Self {
            velocity: Vec2::ZERO,
        }
    }
}

impl From<Vec2> for Velocity {
    fn from(velocity: Vec2) -> Self {
        Self { velocity }
    }
}

// Car movement system
pub fn move_car(
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
    let terrain = game_map.get_tile(pos.x, pos.y, TILE_SIZE as f32);
    //let terrain = &TILES[tile_id];

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

// Car spawning functionality
pub fn spawn_cars(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
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
