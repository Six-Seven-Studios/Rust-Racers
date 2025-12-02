use crate::drift_settings::DriftSettings;
use crate::game_logic::{
    CAR_SIZE, CLIENT_TIMESTEP, DRIFT_RELEASE_BOOST, Orientation, PhysicsInput, PLAYER_SPEED,
    PlayerControlled, TILE_SIZE, Velocity, handle_collision,
};
use crate::multiplayer::NetworkPlayer;
use crate::networking::InputData;
use crate::networking_plugin::NetworkClient;
use crate::speed::SpeedBoost;
use bevy::input::ButtonInput;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct InputSequence {
    pub current: u64,
}

// Buffer to accumulate inputs before sending to server
#[derive(Resource, Default)]
pub struct InputBuffer {
    pub pending_inputs: Vec<InputData>,
}

#[derive(Clone)]
pub struct PredictedState {
    pub sequence: u64,
    pub input: PhysicsInput,
    pub position: Vec2,
    pub velocity: Vec2,
    pub angle: f32,
    pub was_drifting: bool,
}

#[derive(Component)]
pub struct PredictionBuffer {
    pub states: Vec<PredictedState>,
}

impl PredictionBuffer {
    pub fn new() -> Self {
        Self { states: Vec::new() }
    }
}

// Capture input, predict movement locally, and buffer for sending
pub fn send_keyboard_input(
    mut network_client: ResMut<NetworkClient>,
    input: Res<ButtonInput<KeyCode>>,
    mut input_sequence: ResMut<InputSequence>,
    mut input_buffer: ResMut<InputBuffer>,
    mut player_car: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut Orientation,
            &mut PredictionBuffer,
            Option<&SpeedBoost>,
        ),
        With<PlayerControlled>,
    >,
    other_cars: Query<(&Transform, &Velocity), (With<NetworkPlayer>, Without<PlayerControlled>)>,
    game_map: Res<crate::game_logic::GameMap>,
    drift_settings: Res<DriftSettings>,
) {
    let Some(client) = network_client.client.as_mut() else {
        return;
    };

    let forward = input.pressed(KeyCode::KeyW);
    let backward = input.pressed(KeyCode::KeyS);
    let left = input.pressed(KeyCode::KeyA);
    let right = input.pressed(KeyCode::KeyD);
    let drift = input.pressed(KeyCode::Space);

    input_sequence.current += 1;
    let sequence = input_sequence.current;

    // Buffer this input to send later
    let easy_drift = drift_settings.easy_mode;
    let boost_active = player_car
        .get_single()
        .map(|(_, _, _, _, boost)| boost.is_some())
        .unwrap_or(false);

    input_buffer.pending_inputs.push(InputData {
        sequence,
        forward,
        backward,
        left,
        right,
        drift,
        easy_drift,
        boost: boost_active,
    });

    // Send buffered inputs to server (client sends at 60 Hz)
    if !input_buffer.pending_inputs.is_empty() {
        let _ = client.send_player_input_buffer(input_buffer.pending_inputs.clone());
        input_buffer.pending_inputs.clear();
    }

    // Predict movement locally for instant feedback
    if let Ok((mut transform, mut velocity, mut orientation, mut buffer, speed_boost)) =
        player_car.get_single_mut()
    {
        let physics_input = PhysicsInput {
            forward,
            backward,
            left,
            right,
            drift,
            easy_drift,
            boost: speed_boost.is_some(),
        };

        let was_drifting = buffer.states.last()
            .map(|s| s.was_drifting)
            .unwrap_or(false);

        let old_pos = transform.translation.truncate();
        let mut pos = old_pos;
        let tile = game_map.get_tile(pos.x, pos.y, TILE_SIZE as f32);

        // Apply physics locally with client timestep
        crate::game_logic::apply_physics(
            &mut pos,
            &mut velocity,
            &mut orientation,
            &physics_input,
            CLIENT_TIMESTEP,
            tile.speed_modifier,
            tile.friction_modifier,
            tile.turn_modifier,
            tile.decel_modifier,
        );

        // Drift boost
        if was_drifting && !drift {
            let boost_velocity = orientation.forward_vector() * PLAYER_SPEED * DRIFT_RELEASE_BOOST;
            velocity.velocity += boost_velocity;
        }

        // Clamp to map bounds to stay in the playable area
        let half_width = game_map.width / 2.0;
        let half_height = game_map.height / 2.0;
        let car_half_size = (CAR_SIZE as f32) / 2.0;
        pos.x = pos
            .x
            .clamp(-half_width + car_half_size, half_width - car_half_size);
        pos.y = pos
            .y
            .clamp(-half_height + car_half_size, half_height - car_half_size);

        // Handle collisions against walls/other cars so prediction matches authoritative sim
        let new_position = pos.extend(transform.translation.z);
        let other_cars_iter = other_cars
            .iter()
            .map(|(t, v)| (t.translation.truncate(), v.velocity));
        let should_update = handle_collision(
            new_position,
            old_pos,
            &mut velocity.velocity,
            &game_map,
            other_cars_iter,
        );

        if should_update {
            transform.translation = new_position;
        } else {
            pos = old_pos;
        }
        transform.rotation = Quat::from_rotation_z(orientation.angle);

        // Store prediction
        buffer.states.push(PredictedState {
            sequence,
            input: physics_input,
            position: transform.translation.truncate(),
            velocity: velocity.velocity,
            angle: orientation.angle,
            was_drifting: drift,
        });

        // Keep last 120 states (2 seconds at 60 Hz)
        if buffer.states.len() > 120 {
            buffer.states.remove(0);
        }
    }
}
