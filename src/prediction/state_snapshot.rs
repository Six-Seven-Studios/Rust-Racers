use bevy::prelude::*;
use crate::game_logic::{Velocity, Orientation};

/// Complete snapshot of a car's physics state at a specific moment
///
/// Used for:
/// - Storing predicted states for comparison with server
/// - Re-simulation starting point during reconciliation
/// - Rollback when prediction errors are detected
#[derive(Clone, Debug)]
pub struct StateSnapshot {
    pub position: Vec2,
    pub velocity: Vec2,
    pub angle: f32,
    pub sequence: u64,      // Input sequence number that produced this state
    pub timestamp: f64,      // Game time when this state was captured
}

impl StateSnapshot {
    /// Create a snapshot from current ECS components
    pub fn from_components(
        transform: &Transform,
        velocity: &Velocity,
        orientation: &Orientation,
        sequence: u64,
        timestamp: f64,
    ) -> Self {
        Self {
            position: Vec2::new(transform.translation.x, transform.translation.y),
            velocity: velocity.velocity,
            angle: orientation.angle,
            sequence,
            timestamp,
        }
    }

    /// Apply this snapshot to ECS components
    pub fn apply_to_components(
        &self,
        transform: &mut Transform,
        velocity: &mut Velocity,
        orientation: &mut Orientation,
    ) {
        transform.translation.x = self.position.x;
        transform.translation.y = self.position.y;
        transform.rotation = Quat::from_rotation_z(self.angle);
        velocity.velocity = self.velocity;
        orientation.angle = self.angle;
    }

    /// Calculate the distance between two states (for error detection)
    pub fn distance_to(&self, other: &StateSnapshot) -> f32 {
        self.position.distance(other.position)
    }

    /// Calculate velocity difference between two states
    pub fn velocity_difference(&self, other: &StateSnapshot) -> f32 {
        (self.velocity - other.velocity).length()
    }

    /// Calculate angle difference between two states (in radians)
    pub fn angle_difference(&self, other: &StateSnapshot) -> f32 {
        let diff = (self.angle - other.angle).abs();
        // Normalize to [-PI, PI]
        if diff > std::f32::consts::PI {
            2.0 * std::f32::consts::PI - diff
        } else {
            diff
        }
    }

    /// Create a snapshot from server position data
    pub fn from_server_data(
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        angle: f32,
        input_count: u64,
        timestamp: f64,
    ) -> Self {
        Self {
            position: Vec2::new(x, y),
            velocity: Vec2::new(vx, vy),
            angle,
            sequence: input_count,
            timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_calculation() {
        let state1 = StateSnapshot {
            position: Vec2::new(0.0, 0.0),
            velocity: Vec2::ZERO,
            angle: 0.0,
            sequence: 0,
            timestamp: 0.0,
        };

        let state2 = StateSnapshot {
            position: Vec2::new(3.0, 4.0),
            velocity: Vec2::ZERO,
            angle: 0.0,
            sequence: 1,
            timestamp: 0.016,
        };

        assert_eq!(state1.distance_to(&state2), 5.0);
    }

    #[test]
    fn test_angle_difference() {
        let state1 = StateSnapshot {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            angle: 0.0,
            sequence: 0,
            timestamp: 0.0,
        };

        let state2 = StateSnapshot {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            angle: std::f32::consts::PI / 2.0,
            sequence: 1,
            timestamp: 0.016,
        };

        let diff = state1.angle_difference(&state2);
        assert!((diff - std::f32::consts::PI / 2.0).abs() < 0.001);
    }
}
