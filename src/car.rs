use crate::LapCounter;
use bevy::prelude::*;
use crate::map::GameMap;
use crate::theta::{theta_star, ThetaCommand};
use crate::TILE_SIZE;
use crate::collisions::handle_collision;

// Car-related constants
pub const PLAYER_SPEED: f32 = 350.;
pub const ACCEL_RATE: f32 = 700.;
pub const FRICTION: f32 = 0.95;
pub const TURNING_RATE: f32 = 3.0;
pub const LATERAL_FRICTION: f32 = 8.0;
pub const CAR_SIZE: u32 = 64;

// Car-related components
#[derive(Component)]
pub struct Car;

#[derive(Component)]
pub struct PlayerControlled;

#[derive(Component)]
pub struct AIControlled;

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
pub fn move_player_car(
    game_map: Res<GameMap>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    player_car: Single<(&mut Transform, &mut Velocity, &mut Orientation), (With<PlayerControlled>, Without<Background>)>,
    other_cars: Query<(&Transform, &Velocity), (With<Car>, Without<PlayerControlled>)>,
) {
    let (mut transform, mut velocity, mut orientation) = player_car.into_inner();

    let deltat = time.delta_secs();
    let accel = ACCEL_RATE * deltat;

    // Space bar to drift
    let is_drifting = input.pressed(KeyCode::Space);

    // PLACEHOLDER LOGIC FOR TILE COLLISIONS

    // Get the current tile
    let pos = transform.translation.truncate();
    let tile = game_map.get_tile(pos.x, pos.y, TILE_SIZE as f32);
    
    // Modifiers from terrain
    let fric_mod  = tile.friction_modifier;
    let speed_mod = tile.speed_modifier;
    let turn_mod  = tile.turn_modifier;
    let decel_mod = tile.decel_modifier;
    let x = tile.x_coordinate;
    let y = tile.y_coordinate;

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
        // println!("{},{}", x, y); commented by dvdzs for lap logic
        **velocity = velocity.clamp_length_max(PLAYER_SPEED*speed_mod);
    }

    // Accelerate in the direction opposite of orientation
    if input.pressed(KeyCode::KeyS) {
        let backward = -orientation.forward_vector() * (accel / 2.0);
        **velocity += backward;
        **velocity = velocity.clamp_length_max(PLAYER_SPEED*(speed_mod / 2.0));
    }

    // Friction when not accelerating
    if !input.any_pressed([KeyCode::KeyW, KeyCode::KeyS]) {
        let decel_rate = decel_mod * fric_mod * deltat;
        let curr_speed =  velocity.length();
        if curr_speed > 0.0 {
            let new_speed = (curr_speed - decel_rate).max(0.0);
            **velocity = velocity.normalize() * new_speed;
        }
    }

    // Apply lateral friction when not drifting to reduce sliding
    if !is_drifting && velocity.length() > 0.01 {
        let forward = orientation.forward_vector();
        let right = Vec2::new(-forward.y, forward.x);

        let forward_speed = velocity.dot(forward);
        let lateral_speed = velocity.dot(right);

        let damping = (1.0 - LATERAL_FRICTION * deltat).max(0.0);
        let new_lateral_speed = lateral_speed * damping;

        **velocity = forward * forward_speed + right * new_lateral_speed;
    }

    // Updated position
    let change = **velocity * deltat;

    let min = Vec3::new(
        -game_map.width / 2. + (CAR_SIZE as f32) / 2.,
        -game_map.height / 2. + (CAR_SIZE as f32) / 2.,
        900.,
    );
    let max = Vec3::new(
        game_map.width / 2. - (CAR_SIZE as f32) / 2.,
        game_map.height / 2. - (CAR_SIZE as f32) / 2.,
        900.,
    );

    // Rotate car to match orientation
    transform.rotation = Quat::from_rotation_z(orientation.angle);

    // Calculate new position
    let new_position = (transform.translation + change.extend(0.)).clamp(min, max);

    // Handle collision detection and response
    let should_update = handle_collision(
        new_position,
        transform.translation.truncate(),
        &mut velocity.velocity,
        &game_map,
        &other_cars,
    );

    // Update position only if no collision occurred
    if should_update {
        transform.translation = new_position;
    }
}

pub fn move_ai_cars(
    mut game_map: ResMut<GameMap>,
    time: Res<Time>,
    mut ai_cars: Query<(&mut Transform, &mut Velocity, &mut Orientation), (With<AIControlled>, Without<Background>)>,
    other_cars: Query<(&Transform, &Velocity), (With<Car>, Without<AIControlled>)>,
) {

    let deltat = time.delta_secs();
    let accel = ACCEL_RATE * deltat;

    // Turning
    // Iterate through each AI-controlled car
    for (mut transform, mut velocity, mut orientation) in ai_cars.iter_mut() {
        let pos = transform.translation.truncate();
        let current_pos = (pos.x, pos.y);

        // Get the current tile
        let tile = game_map.get_tile(pos.x, pos.y, TILE_SIZE as f32);

        // Modifiers from terrain
        let fric_mod = tile.friction_modifier;
        let speed_mod = tile.speed_modifier;
        let turn_mod = tile.turn_modifier;
        let decel_mod = tile.decel_modifier;

        // Get command from theta_star algorithm
        let command = theta_star(current_pos, orientation.angle, &mut game_map.theta_checkpoint_list);

        // Execute the command
        match command {
            ThetaCommand::TurnLeft => {
                orientation.angle += TURNING_RATE * deltat * turn_mod;
            }
            ThetaCommand::TurnRight => {
                orientation.angle -= TURNING_RATE * deltat * turn_mod;
            }
            ThetaCommand::Forward => {
                let forward = orientation.forward_vector() * accel;
                **velocity += forward;
                **velocity = velocity.clamp_length_max(PLAYER_SPEED * speed_mod);
            }
            ThetaCommand::Stop => {
                if velocity.length() > 0.0 {
                    let backward = -orientation.forward_vector() * (accel / 2.0);
                    **velocity += backward;
                    **velocity = velocity.clamp_length_max(PLAYER_SPEED * (speed_mod / 2.0));
                } else {
                    **velocity = Vec2::ZERO;
                }
            }
        }


        // Apply friction when not accelerating forward
        if !matches!(command, ThetaCommand::Forward) {
            let decel_rate = decel_mod * fric_mod * deltat;
            let curr_speed = velocity.length();
            if curr_speed > 0.0 {
                let new_speed = (curr_speed - decel_rate).max(0.0);
                if new_speed > 0.0 {
                    **velocity = velocity.normalize() * new_speed;
                } else {
                    **velocity = Vec2::ZERO;
                }
            }
        }


        // Updated position
        let change = **velocity * deltat;

        let min = Vec3::new(
            -game_map.width / 2. + (CAR_SIZE as f32) / 2.,
            -game_map.height / 2. + (CAR_SIZE as f32) / 2.,
            900.,
        );
        let max = Vec3::new(
            game_map.width / 2. - (CAR_SIZE as f32) / 2.,
            game_map.height / 2. - (CAR_SIZE as f32) / 2.,
            900.,
        );

        // Rotate car to match orientation
        transform.rotation = Quat::from_rotation_z(orientation.angle);

        // Calculate new position
        let new_position = (transform.translation + change.extend(0.)).clamp(min, max);

        // Handle collision detection and response
        let should_update = handle_collision(
            new_position,
            transform.translation.truncate(),
            &mut velocity.velocity,
            &game_map,
            &other_cars,
        );

        // Update position only if no collision occurred
        if should_update {
            transform.translation = new_position;
        }
    }
}
// Car spawning functionality
pub fn spawn_cars(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let car_sheet_handle = asset_server.load("updatedCar.png");
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
            translation: Vec3::new(2752., 960., 10.),
            ..default()
        },
        Velocity::new(),
        Orientation::new(0.0),
        Car,
        PlayerControlled,
        LapCounter::default(),
    ));


    commands.spawn((
        Sprite::from_atlas_image(
            car_sheet_handle.clone(),
            TextureAtlas {
                layout: car_layout_handle.clone(),
                index: 0,
            },
        ),
        Transform {
            translation: Vec3::new(2752., 960., 10.),
            ..default()
        },
        Velocity::new(),
        Orientation::new(0.0),
        Car,
        AIControlled,
        LapCounter::default(),
    ));


}
