use bevy::prelude::*;
use bevy::input::ButtonInput;
use crate::game_logic::{Velocity, Orientation, PlayerControlled, PhysicsInput, TILE_SIZE, CLIENT_TIMESTEP};
use crate::networking_plugin::NetworkClient;
use crate::networking::InputData;
use crate::drift_settings::DriftSettings;

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
    mut player_car: Query<(&mut Transform, &mut Velocity, &mut Orientation, &mut PredictionBuffer), With<PlayerControlled>>,
    game_map: Res<crate::game_logic::GameMap>,
    drift_settings: Res<DriftSettings>,
) {
    let Some(client) = network_client.client.as_mut() else { return };

    let forward = input.pressed(KeyCode::KeyW);
    let backward = input.pressed(KeyCode::KeyS);
    let left = input.pressed(KeyCode::KeyA);
    let right = input.pressed(KeyCode::KeyD);
    let drift = input.pressed(KeyCode::Space);

    input_sequence.current += 1;
    let sequence = input_sequence.current;

    // Buffer this input to send later
    let easy_drift = drift_settings.easy_mode;
    input_buffer.pending_inputs.push(InputData {
        sequence,
        forward,
        backward,
        left,
        right,
        drift,
        easy_drift,
    });

    // Send buffered inputs to server (client sends at 60 Hz)
    if !input_buffer.pending_inputs.is_empty() {
        let _ = client.send_player_input_buffer(input_buffer.pending_inputs.clone());
        input_buffer.pending_inputs.clear();
    }

    // Predict movement locally for instant feedback
    if let Ok((mut transform, mut velocity, mut orientation, mut buffer)) = player_car.get_single_mut() {
        let physics_input = PhysicsInput { forward, backward, left, right, drift, easy_drift };

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

        // Update visuals immediately
        transform.translation = pos.extend(transform.translation.z);
        transform.rotation = Quat::from_rotation_z(orientation.angle);

        // Store prediction
        buffer.states.push(PredictedState {
            sequence,
            input: physics_input,
            position: pos,
            velocity: velocity.velocity,
            angle: orientation.angle,
        });

        // Keep last 120 states (2 seconds at 60 Hz)
        if buffer.states.len() > 120 {
            buffer.states.remove(0);
        }
    }
}
