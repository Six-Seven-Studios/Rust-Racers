// Simple client-side prediction for responsive controls
// Predicts movement locally and stores buffer for comparison with server

use bevy::prelude::*;
use bevy::input::ButtonInput;
use crate::game_logic::{Velocity, Orientation, PlayerControlled, PhysicsInput, TILE_SIZE, FIXED_TIMESTEP};
use crate::networking_plugin::NetworkClient;

// Track input sequence numbers
#[derive(Resource, Default)]
pub struct InputSequence {
    pub current: u64,
}

// Store a single predicted state
#[derive(Clone)]
pub struct PredictedState {
    pub sequence: u64,
    pub input: PhysicsInput,
    pub position: Vec2,
    pub velocity: Vec2,
    pub angle: f32,
}

// Buffer of recent predictions
#[derive(Component)]
pub struct PredictionBuffer {
    pub states: Vec<PredictedState>,
}

impl PredictionBuffer {
    pub fn new() -> Self {
        Self { states: Vec::new() }
    }
}

// Send input and predict movement locally
// Runs at fixed 60 Hz - must run in FixedUpdate schedule
pub fn send_keyboard_input(
    mut network_client: ResMut<NetworkClient>,
    input: Res<ButtonInput<KeyCode>>,
    mut input_sequence: ResMut<InputSequence>,
    mut player_car: Query<(&mut Transform, &mut Velocity, &mut Orientation, &mut PredictionBuffer), With<PlayerControlled>>,
    game_map: Res<crate::game_logic::GameMap>,
) {
    let Some(client) = network_client.client.as_mut() else { return };

    let forward = input.pressed(KeyCode::KeyW);
    let backward = input.pressed(KeyCode::KeyS);
    let left = input.pressed(KeyCode::KeyA);
    let right = input.pressed(KeyCode::KeyD);
    let drift = input.pressed(KeyCode::Space);

    input_sequence.current += 1;
    let sequence = input_sequence.current;

    // Send input to server with sequence number
    let _ = client.send_player_input(sequence, forward, backward, left, right, drift);

    // Predict movement locally for instant feedback
    if let Ok((mut transform, mut velocity, mut orientation, mut buffer)) = player_car.get_single_mut() {
        let physics_input = PhysicsInput { forward, backward, left, right, drift };

        let old_pos = transform.translation.truncate();
        let mut pos = old_pos;
        let tile = game_map.get_tile(pos.x, pos.y, TILE_SIZE as f32);

        // Apply physics locally with fixed timestep (same as server)
        crate::game_logic::apply_physics(
            &mut pos,
            &mut velocity,
            &mut orientation,
            &physics_input,
            FIXED_TIMESTEP,
            tile.speed_modifier,
            tile.friction_modifier,
            tile.turn_modifier,
            tile.decel_modifier,
        );

        // Update visuals immediately (responsive!)
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

        // Keep last 60 states (~1 second at 60fps)
        if buffer.states.len() > 60 {
            buffer.states.remove(0);
        }
    }
}
