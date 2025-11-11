use bevy::prelude::*;
use crate::game_logic::{Velocity, Orientation, physics::{PhysicsInput, apply_physics}, GameMap, TILE_SIZE};
use crate::prediction::state_snapshot::StateSnapshot;
use crate::prediction::input_buffer::{InputBuffer, TimestampedInput};

/// Threshold for position error before triggering reconciliation (in pixels)
pub const RECONCILIATION_THRESHOLD: f32 = 5.0;

/// Threshold for velocity error before triggering reconciliation (in pixels/second)
pub const VELOCITY_THRESHOLD: f32 = 50.0;

/// Reconciliation engine that compares predicted state with server state
///
/// When the server sends authoritative state, this engine:
/// 1. Compares it with what we predicted at that sequence number
/// 2. If error is above threshold, re-simulates from server state
/// 3. Returns the corrected state and whether correction was needed
pub struct ReconciliationEngine;

impl ReconciliationEngine {
    /// Reconcile client prediction with server state
    ///
    /// Returns:
    /// - The corrected state (either re-simulated or predicted)
    /// - Whether a correction was needed
    /// - The error magnitude (for debugging/logging)
    pub fn reconcile(
        server_state: &StateSnapshot,
        input_buffer: &InputBuffer,
        game_map: &GameMap,
        delta: f32,
    ) -> (StateSnapshot, bool, f32) {
        // Get all inputs that came after the server's acknowledged input
        let pending_inputs = input_buffer.get_from_sequence(server_state.sequence + 1);

        if pending_inputs.is_empty() {
            // No pending inputs, server state is current
            return (server_state.clone(), false, 0.0);
        }

        // Re-simulate from server state using pending inputs
        let re_simulated_state = Self::re_simulate(
            server_state,
            &pending_inputs,
            game_map,
            delta,
        );

        // For now, we don't have the predicted state to compare against
        // The client_prediction module will handle this comparison
        (re_simulated_state, false, 0.0)
    }

    /// Re-simulate physics from a given state using a sequence of inputs
    ///
    /// This is the core reconciliation algorithm:
    /// - Start from the server's authoritative state
    /// - Re-apply all inputs that the server hasn't processed yet
    /// - Get the "corrected" predicted position
    pub fn re_simulate(
        start_state: &StateSnapshot,
        inputs: &[TimestampedInput],
        game_map: &GameMap,
        delta: f32,
    ) -> StateSnapshot {
        let mut position = start_state.position;
        let mut velocity = Velocity::from(start_state.velocity);
        let mut orientation = Orientation::new(start_state.angle);

        for input in inputs {
            // Get terrain modifiers for current position
            let tile = game_map.get_tile(position.x, position.y, TILE_SIZE as f32);

            // Apply physics with the same deterministic function as the server
            apply_physics(
                &mut position,
                &mut velocity,
                &mut orientation,
                &input.input,
                delta,
                tile.speed_modifier,
                tile.friction_modifier,
                tile.turn_modifier,
                tile.decel_modifier,
            );
        }

        // Create snapshot of the re-simulated state
        let final_sequence = inputs.last()
            .map(|i| i.sequence)
            .unwrap_or(start_state.sequence);

        let final_timestamp = inputs.last()
            .map(|i| i.timestamp)
            .unwrap_or(start_state.timestamp);

        StateSnapshot {
            position,
            velocity: velocity.velocity,
            angle: orientation.angle,
            sequence: final_sequence,
            timestamp: final_timestamp,
        }
    }

    /// Check if prediction error exceeds threshold and needs correction
    pub fn needs_correction(
        predicted: &StateSnapshot,
        server: &StateSnapshot,
    ) -> (bool, f32) {
        let position_error = predicted.distance_to(server);
        let velocity_error = predicted.velocity_difference(server);

        let needs_correction = position_error > RECONCILIATION_THRESHOLD
            || velocity_error > VELOCITY_THRESHOLD;

        (needs_correction, position_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_correction_small_error() {
        let predicted = StateSnapshot {
            position: Vec2::new(100.0, 100.0),
            velocity: Vec2::new(10.0, 0.0),
            angle: 0.0,
            sequence: 5,
            timestamp: 0.1,
        };

        let server = StateSnapshot {
            position: Vec2::new(102.0, 100.0),
            velocity: Vec2::new(10.0, 0.0),
            angle: 0.0,
            sequence: 5,
            timestamp: 0.1,
        };

        let (needs_correction, error) = ReconciliationEngine::needs_correction(&predicted, &server);
        assert!(!needs_correction);
        assert_eq!(error, 2.0);
    }

    #[test]
    fn test_needs_correction_large_error() {
        let predicted = StateSnapshot {
            position: Vec2::new(100.0, 100.0),
            velocity: Vec2::new(10.0, 0.0),
            angle: 0.0,
            sequence: 5,
            timestamp: 0.1,
        };

        let server = StateSnapshot {
            position: Vec2::new(110.0, 100.0),
            velocity: Vec2::new(10.0, 0.0),
            angle: 0.0,
            sequence: 5,
            timestamp: 0.1,
        };

        let (needs_correction, error) = ReconciliationEngine::needs_correction(&predicted, &server);
        assert!(needs_correction);
        assert_eq!(error, 10.0);
    }
}
