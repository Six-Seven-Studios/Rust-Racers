use crate::car_skins::{AI_SKIN, CarSkinSelection};
use crate::car_state::CarState;
use crate::client_prediction::PredictionBuffer;
use crate::drift_settings::DriftSettings;
use crate::game_logic::{
    ACCEL_RATE, CAR_SIZE, EASY_DRIFT_LATERAL_FRICTION, EASY_DRIFT_SPEED_BONUS,
    EASY_DRIFT_TURN_MULTIPLIER, FRICTION, LATERAL_FRICTION, PLAYER_SPEED, START_ORIENTATION,
    START_POSITIONS, TURNING_RATE,
};
use crate::game_logic::{AIControlled, Car, Orientation, PlayerControlled, Velocity};
use crate::game_logic::{
    CpuDifficulty, GameMap, LapCounter, TILE_SIZE, ThetaCheckpointList, ThetaCommand,
    theta_star, handle_collision,
};
use crate::speed::SpeedBoost;
use bevy::prelude::*;
use crate::game_logic::theta_grid::ThetaGrid;


// Car-related components
#[derive(Component)]
pub struct Background;

// Car movement system
pub fn move_player_car(
    game_map: Res<GameMap>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    drift_settings: Res<DriftSettings>,
    player_car: Single<
        (
            &mut Transform,
            &mut Velocity,
            &mut Orientation,
            &mut Sprite,
            Option<&SpeedBoost>,
        ),
        (With<PlayerControlled>, Without<Background>),
    >,
    other_cars: Query<(&Transform, &Velocity), (With<Car>, Without<PlayerControlled>)>,
) {
    let (mut transform, mut velocity, mut orientation, mut sprite, speed_boost) =
        player_car.into_inner();

    let deltat = time.delta_secs();
    let accel = ACCEL_RATE * deltat;

    // Space bar to drift
    let is_drifting = input.pressed(KeyCode::Space);
    let easy_mode = drift_settings.easy_mode;
    let turn_scale = if is_drifting && easy_mode {
        EASY_DRIFT_TURN_MULTIPLIER
    } else {
        1.0
    };
    let speed_bonus = if is_drifting && easy_mode {
        EASY_DRIFT_SPEED_BONUS
    } else {
        1.0
    };

    // PLACEHOLDER LOGIC FOR TILE COLLISIONS

    // Get the current tile
    let pos = transform.translation.truncate();
    let tile = game_map.get_tile(pos.x, pos.y, TILE_SIZE as f32);
    // println!("title id: {}",tile.tile_id);
    // Modifiers from terrain
    let mut fric_mod = tile.friction_modifier;
    let mut speed_mod = tile.speed_modifier;
    let mut turn_mod = tile.turn_modifier;
    let decel_mod = tile.decel_modifier;

    //LOS DEBUG, ADD 'mut gizmos: Gizmos' to function input
    /*
    println!("Car Position: {}, {}", tile.x_coordinate, tile.y_coordinate);
    let los = game_map.line_of_sight((tile.x_coordinate, tile.y_coordinate), (77.0, 17.0));
    if(los)
    {
        println!("Line of sight found")
    } else { println!("Line of sight not found") };

    let world_pos1 = game_map.tile_to_world(tile.x_coordinate, tile.y_coordinate, 64.0);
    let world_pos2 = game_map.tile_to_world(77.0, 17.0, 64.0);

    // Choose color based on LOS result
    let color = if los {
        Color::srgb(0.0, 1.0, 0.0) // Green for clear LOS
    } else {
        Color::srgb(1.0, 0.0, 0.0) // Red for blocked
    };

    // Draw line between points
    gizmos.line_2d(world_pos1, world_pos2, color);

    // Draw dots at endpoints
    gizmos.circle_2d(world_pos1, 8.0, color);
    gizmos.circle_2d(world_pos2, 8.0, color);
    */

    // Speed boost override

    // if tile.speed_boost {
    //     **velocity = orientation.forward_vector() * PLAYER_SPEED * 1.5;
    // }

    if speed_boost.is_some() {
        fric_mod = 10.0;
        speed_mod = 3.0;
        turn_mod = 1.5;
        // print!("Speed boost on tile at {}, {}\n", x, y);
        // ADD SPEED BOOST COLOR CHANGE HERE
        let hue = (time.elapsed_secs() * 180.0) % 360.0; // Speed of 180 degrees/sec
        sprite.color = Color::hsl(hue, 1.0, 0.7); // Full saturation, 70% lightness
    } else {
        sprite.color = Color::WHITE; // Normal color (no tint)
    }

    // Turning
    if input.pressed(KeyCode::KeyA) {
        orientation.angle += TURNING_RATE * deltat * turn_mod * turn_scale;
    }
    if input.pressed(KeyCode::KeyD) {
        orientation.angle -= TURNING_RATE * deltat * turn_mod * turn_scale;
    }

    // Accelerate forward in the direction of car orientation
    if input.pressed(KeyCode::KeyW) {
        let forward = orientation.forward_vector() * accel;
        **velocity += forward;
        // println!("{},{}", x, y); commented by dvdzs for lap logic
        **velocity = velocity.clamp_length_max(PLAYER_SPEED * speed_mod * speed_bonus);
    }

    // Accelerate in the direction opposite of orientation
    if input.pressed(KeyCode::KeyS) {
        let backward = -orientation.forward_vector() * (accel / 2.0);
        **velocity += backward;
        **velocity = velocity.clamp_length_max(PLAYER_SPEED * (speed_mod / 2.0) * speed_bonus);
    }

    // Friction when not accelerating
    if !input.any_pressed([KeyCode::KeyW, KeyCode::KeyS]) {
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

    // Apply lateral friction when not drifting (or in easy mode drifts) to reduce sliding
    if (!is_drifting || easy_mode) && velocity.length() > 0.01 {
        let forward = orientation.forward_vector();
        let right = Vec2::new(-forward.y, forward.x);

        let forward_speed = velocity.dot(forward);
        let lateral_speed = velocity.dot(right);

        let damping_strength = if is_drifting && easy_mode {
            EASY_DRIFT_LATERAL_FRICTION
        } else {
            LATERAL_FRICTION
        };
        let damping = (1.0 - damping_strength * deltat).max(0.0);
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
    // Convert Query to iterator of (position, velocity) pairs
    let other_cars_iter = other_cars
        .iter()
        .map(|(t, v)| (t.translation.truncate(), v.velocity));
    let should_update = handle_collision(
        new_position,
        transform.translation.truncate(),
        &mut velocity.velocity,
        &game_map,
        other_cars_iter,
    );

    // Update position only if no collision occurred
    if should_update {
        transform.translation = new_position;
    }
}

pub fn move_ai_cars(
    game_map: Res<GameMap>,
    theta_grid: Res<ThetaGrid>,
    time: Res<Time>,
    mut ai_cars: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut Orientation,
            &mut ThetaCheckpointList,
        ),
        (With<AIControlled>, Without<Background>),
    >,
    other_cars: Query<(&Transform, &Velocity), (With<Car>, Without<AIControlled>)>,
) {
    let deltat = time.delta_secs();
    let accel = ACCEL_RATE * deltat;

    // Turning
    // Iterate through each AI-controlled car
    for (mut transform, mut velocity, mut orientation, mut theta_checkpoint_list) in
        ai_cars.iter_mut()
    {
        let pos = transform.translation.truncate();

        // Get the current tile
        let tile = game_map.get_tile(pos.x, pos.y, TILE_SIZE as f32);
        // Modifiers from terrain
        let fric_mod = tile.friction_modifier;
        let speed_mod = tile.speed_modifier;
        let turn_mod = tile.turn_modifier;
        let decel_mod = tile.decel_modifier;

        // Get command from steering helper using Theta* pathfinding
        let command = theta_star(
            (pos.x, pos.y),
            orientation.angle,
            &mut theta_checkpoint_list,
            &theta_grid,
        );

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
            ThetaCommand::Reverse => {
                let backward = -orientation.forward_vector() * (accel / 4.0);
                **velocity += backward;
                **velocity = velocity.clamp_length_max(PLAYER_SPEED * (speed_mod / 4.0));
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

        // Apply friction when not accelerating forward or reversing
        if !matches!(command, ThetaCommand::Forward | ThetaCommand::Reverse) {
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
        // Convert Query to iterator of (position, velocity) pairs
        let other_cars_iter = other_cars
            .iter()
            .map(|(t, v)| (t.translation.truncate(), v.velocity));
        let should_update = handle_collision(
            new_position,
            transform.translation.truncate(),
            &mut velocity.velocity,
            &game_map,
            other_cars_iter,
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
    state: Res<State<crate::GameState>>,
    skin_selection: Res<CarSkinSelection>,
) {
    let car_sheet_handle = asset_server.load(skin_selection.current_skin());
    let car_layout = TextureAtlasLayout::from_grid(UVec2::splat(CAR_SIZE), 2, 2, None, None);
    let car_layout_handle = texture_atlases.add(car_layout);

    let player_start = START_POSITIONS[0];

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
            translation: Vec3::new(player_start.0, player_start.1, 10.),
            rotation: Quat::from_rotation_z(START_ORIENTATION),
            ..default()
        },
        Velocity::new(),
        Orientation::new(START_ORIENTATION),
        Car,
        PlayerControlled,
        LapCounter::default(),
        PredictionBuffer::new(),
    ));

    // Spawn AI car IF in demo mode
    if *state.get() == crate::GameState::PlayingDemo {
        let ai_start = START_POSITIONS.get(1).copied().unwrap_or(player_start);
        commands.spawn((
            Sprite::from_atlas_image(
                asset_server.load(AI_SKIN),
                TextureAtlas {
                    layout: car_layout_handle.clone(),
                    index: 0,
                },
            ),
            Transform {
                translation: Vec3::new(ai_start.0, ai_start.1, 10.),
                rotation: Quat::from_rotation_z(START_ORIENTATION),
                ..default()
            },
            Velocity::new(),
            Orientation::new(START_ORIENTATION),
            Car,
            AIControlled,
            LapCounter::default(),
            CarState::new(), // carstate for the AI
            ThetaCheckpointList::new(Vec::new()),
        ));
    }
}

// beginnings of the fsm system
pub fn ai_car_fsm(
    mut ai_query: Query<
        (
            Entity,
            &mut CarState,
            &mut Transform,
            &mut Velocity,
            &mut Orientation,
        ),
        With<AIControlled>,
    >,
    other_cars: Query<&Transform, (With<Car>, Without<AIControlled>)>,
    mut delta_time: Res<Time>,
    difficulty: Res<CpuDifficulty>,
) {
    // define proximity threshold (in game units)
    const PROXIMITY_THRESHOLD: f32 = 300.0;

    // just an idea, but we COULD determine threshold based on difficulty
    /*
    let proximity_threshold = match *difficulty {
        CpuDifficulty::Easy => 200.0,   // Blind as a bat
        CpuDifficulty::Medium => 300.0, // Normal
        CpuDifficulty::Hard => 600.0,   // Eagle eyes
    };
    */

    for (entity, mut car_state, mut transform, mut velocity, mut orientation) in ai_query.iter_mut()
    {
        // check for nearby cars
        let ai_pos = transform.translation.truncate();
        let mut closest_car_distance = f32::MAX;
        let mut closest_car_position = None;

        for other_transform in other_cars.iter() {
            let other_pos = other_transform.translation.truncate();
            let distance = ai_pos.distance(other_pos);

            if distance < closest_car_distance {
                closest_car_distance = distance;
                closest_car_position = Some(other_pos);
            }
        }

        // determine if any car is within proximity threshold
        let car_nearby = closest_car_distance < PROXIMITY_THRESHOLD;

        // pass all the properties to the update function
        // maybe roll this into a struct in the future for readability

        car_state.update(
            &mut delta_time,
            &mut transform,
            &mut velocity,
            &mut orientation,
            car_nearby,
            closest_car_position,
            closest_car_distance,
            &difficulty,
        );
    }
}
