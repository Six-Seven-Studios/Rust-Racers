use bevy::prelude::*;
use crate::game_logic::{ACCEL_RATE, FRICTION, TURNING_RATE, PLAYER_SPEED, LATERAL_FRICTION, Velocity, Orientation};

/// Input state for physics simulation
#[derive(Clone, Default)]
pub struct PhysicsInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub drift: bool,
}

/// Apply physics simulation to a single entity
/// This is the core physics logic shared between client and server
pub fn apply_physics(
    position: &mut Vec2,
    velocity: &mut Velocity,
    orientation: &mut Orientation,
    input: &PhysicsInput,
    delta: f32,
    speed_modifier: f32,
    friction_modifier: f32,
    turn_modifier: f32,
    decel_modifier: f32,
) {
    let accel = ACCEL_RATE * delta;

    // Apply turning
    if input.left {
        orientation.angle += TURNING_RATE * delta * turn_modifier;
    }
    if input.right {
        orientation.angle -= TURNING_RATE * delta * turn_modifier;
    }

    // Calculate forward vector
    let forward = orientation.forward_vector();

    // Apply forward acceleration
    if input.forward {
        let forward_accel = forward * accel;
        **velocity += forward_accel;

        // Clamp to max speed
        **velocity = velocity.clamp_length_max(PLAYER_SPEED * speed_modifier);
    }

    // Apply backward acceleration (slower)
    if input.backward {
        let backward_accel = -forward * (accel / 2.0);
        **velocity += backward_accel;
        **velocity = velocity.clamp_length_max(PLAYER_SPEED * (speed_modifier / 2.0));
    }

    // Apply friction when not accelerating
    if !input.forward && !input.backward {
        let decel_rate = decel_modifier * friction_modifier * delta;
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

    // Apply lateral friction when not drifting (client-specific feature)
    if !input.drift && velocity.length() > 0.01 {
        let right = Vec2::new(-forward.y, forward.x);

        let forward_speed = velocity.dot(forward);
        let lateral_speed = velocity.dot(right);

        let damping = (1.0 - LATERAL_FRICTION * delta).max(0.0);
        let new_lateral_speed = lateral_speed * damping;

        **velocity = forward * forward_speed + right * new_lateral_speed;
    }

    // Update position
    *position += **velocity * delta;
}
