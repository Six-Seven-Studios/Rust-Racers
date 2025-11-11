use bevy::prelude::*;
use crate::game_logic::{
    Velocity, Orientation, PlayerControlled,
    physics::{PhysicsInput, apply_physics},
    GameMap, TILE_SIZE,
};
use crate::prediction::{InputBuffer, StateSnapshot};
use crate::multiplayer::NetworkPlayer;
use crate::networking_plugin::NetworkClient;
use std::collections::HashMap;

/// Feature flag to enable/disable client-side prediction
///
/// Set to false to fall back to purely server-authoritative movement
/// Useful for debugging and comparing prediction vs. no prediction
pub const ENABLE_PREDICTION: bool = true;

/// Bevy resource that manages client-side prediction state
///
/// This is the main orchestrator for the prediction system.
/// It tracks:
/// - Input buffer for reconciliation
/// - Predicted states for comparison with server
/// - Last acknowledged server state
#[derive(Resource)]
pub struct ClientPredictionState {
    /// Buffer of recent inputs for reconciliation
    pub input_buffer: InputBuffer,

    /// Predicted states indexed by sequence number
    pub predicted_states: HashMap<u64, StateSnapshot>,

    /// The last server state we received
    pub last_server_state: Option<StateSnapshot>,

    /// Current sequence number (incremented with each input)
    pub current_sequence: u64,

    /// Game time accumulator for timestamping
    pub game_time: f64,
}

impl Default for ClientPredictionState {
    fn default() -> Self {
        Self {
            input_buffer: InputBuffer::new(120), // ~2 seconds at 60fps
            predicted_states: HashMap::new(),
            last_server_state: None,
            current_sequence: 0,
            game_time: 0.0,
        }
    }
}

impl ClientPredictionState {
    /// Add a new input and return its sequence number
    pub fn add_input(&mut self, input: PhysicsInput) -> u64 {
        let sequence = self.input_buffer.add(input, self.game_time);
        self.current_sequence = sequence;
        sequence
    }

    /// Store a predicted state for later comparison
    pub fn store_predicted_state(&mut self, state: StateSnapshot) {
        // Keep only the last 60 predicted states (~1 second)
        if self.predicted_states.len() > 60 {
            // Remove oldest states
            let min_sequence = state.sequence.saturating_sub(60);
            self.predicted_states.retain(|seq, _| *seq >= min_sequence);
        }

        self.predicted_states.insert(state.sequence, state);
    }

    /// Get a predicted state at a specific sequence number
    pub fn get_predicted_state(&self, sequence: u64) -> Option<&StateSnapshot> {
        self.predicted_states.get(&sequence)
    }

    /// Update game time
    pub fn update_time(&mut self, delta: f32) {
        self.game_time += delta as f64;
    }
}

/// System that predicts local player movement based on local inputs
///
/// This runs every frame to give instant feedback on player inputs.
/// The prediction uses the same physics as the server for accuracy.
/// Also sends inputs to the server for authoritative processing.
pub fn predict_local_movement(
    mut prediction_state: ResMut<ClientPredictionState>,
    mut player_query: Query<
        (&mut Transform, &mut Velocity, &mut Orientation),
        (With<PlayerControlled>, Without<NetworkPlayer>)
    >,
    input: Res<ButtonInput<KeyCode>>,
    game_map: Res<GameMap>,
    time: Res<Time>,
    mut network_client: ResMut<NetworkClient>,
) {
    // Update game time
    prediction_state.update_time(time.delta_secs());

    // Get current input state
    let forward = input.pressed(KeyCode::KeyW);
    let backward = input.pressed(KeyCode::KeyS);
    let left = input.pressed(KeyCode::KeyA);
    let right = input.pressed(KeyCode::KeyD);
    let drift = input.pressed(KeyCode::Space);

    let physics_input = PhysicsInput {
        forward,
        backward,
        left,
        right,
        drift,
    };

    // Send input to server
    if let Some(client) = network_client.client.as_mut() {
        let _ = client.send_player_input(forward, backward, left, right, drift);
    }

    if !ENABLE_PREDICTION {
        // If prediction is disabled, just send inputs and return
        return;
    }

    // Add input to buffer for reconciliation
    let sequence = prediction_state.add_input(physics_input.clone());

    // Apply prediction to local player
    if let Ok((mut transform, mut velocity, mut orientation)) = player_query.get_single_mut() {
        let mut position = Vec2::new(transform.translation.x, transform.translation.y);

        // Get terrain modifiers
        let tile = game_map.get_tile(position.x, position.y, TILE_SIZE as f32);

        // Apply physics prediction (same as server)
        apply_physics(
            &mut position,
            &mut velocity,
            &mut orientation,
            &physics_input,
            time.delta_secs(),
            tile.speed_modifier,
            tile.friction_modifier,
            tile.turn_modifier,
            tile.decel_modifier,
        );

        // Update transform
        transform.translation.x = position.x;
        transform.translation.y = position.y;
        transform.rotation = Quat::from_rotation_z(orientation.angle);

        // Store predicted state for reconciliation
        let predicted_state = StateSnapshot {
            position,
            velocity: velocity.velocity,
            angle: orientation.angle,
            sequence,
            timestamp: prediction_state.game_time,
        };

        prediction_state.store_predicted_state(predicted_state);
    }
}

/// Component marker for entities that have prediction enabled
#[derive(Component)]
pub struct Predicted;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prediction_state_initialization() {
        let state = ClientPredictionState::default();
        assert_eq!(state.current_sequence, 0);
        assert!(state.last_server_state.is_none());
    }

    #[test]
    fn test_add_input_increments_sequence() {
        let mut state = ClientPredictionState::default();
        let input = PhysicsInput::default();

        let seq1 = state.add_input(input.clone());
        let seq2 = state.add_input(input.clone());

        assert_eq!(seq1, 0);
        assert_eq!(seq2, 1);
    }

    #[test]
    fn test_predicted_states_storage() {
        let mut state = ClientPredictionState::default();

        let snapshot = StateSnapshot {
            position: Vec2::new(100.0, 100.0),
            velocity: Vec2::ZERO,
            angle: 0.0,
            sequence: 5,
            timestamp: 0.1,
        };

        state.store_predicted_state(snapshot.clone());

        let retrieved = state.get_predicted_state(5);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().position, Vec2::new(100.0, 100.0));
    }
}
