use crate::game_logic::{
    ACCEL_RATE, EASY_DRIFT_LATERAL_FRICTION, EASY_DRIFT_SPEED_BONUS, EASY_DRIFT_TURN_MULTIPLIER,
    LATERAL_FRICTION, Orientation, PLAYER_SPEED, TURNING_RATE, Velocity,
};
use bevy::prelude::*;

/// Input state for physics simulation
#[derive(Clone, Default)]
pub struct PhysicsInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub drift: bool,
    pub easy_drift: bool,
    pub boost: bool,
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
    let mut speed_modifier = speed_modifier;
    let mut friction_modifier = friction_modifier;
    let mut turn_modifier = turn_modifier;
    let drift_turn_scale = if input.drift && input.easy_drift {
        EASY_DRIFT_TURN_MULTIPLIER
    } else {
        1.0
    };
    let drift_speed_bonus = if input.drift && input.easy_drift {
        EASY_DRIFT_SPEED_BONUS
    } else {
        1.0
    };

    if input.boost {
        // Increase top speed and tweak turning/friction while boosted
        speed_modifier *= 3.0;
        friction_modifier = 10.0;
        turn_modifier *= 1.5;
    }

    // Apply turning
    if input.left {
        orientation.angle += TURNING_RATE * delta * turn_modifier * drift_turn_scale;
    }
    if input.right {
        orientation.angle -= TURNING_RATE * delta * turn_modifier * drift_turn_scale;
    }

    // Calculate forward vector
    let forward = orientation.forward_vector();

    // Apply forward acceleration
    if input.forward {
        let forward_accel = forward * accel;
        **velocity += forward_accel;

        // Clamp to max speed
        **velocity = velocity.clamp_length_max(PLAYER_SPEED * speed_modifier * drift_speed_bonus);
    }

    // Apply backward acceleration (slower)
    if input.backward {
        let backward_accel = -forward * (accel / 2.0);
        **velocity += backward_accel;
        **velocity =
            velocity.clamp_length_max(PLAYER_SPEED * (speed_modifier / 2.0) * drift_speed_bonus);
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
    let apply_lateral_control = !input.drift || input.easy_drift;
    if apply_lateral_control && velocity.length() > 0.01 {
        let right = Vec2::new(-forward.y, forward.x);

        let forward_speed = velocity.dot(forward);
        let lateral_speed = velocity.dot(right);

        let damping_strength = if input.drift && input.easy_drift {
            EASY_DRIFT_LATERAL_FRICTION
        } else {
            LATERAL_FRICTION
        };
        let damping = (1.0 - damping_strength * delta).max(0.0);
        let new_lateral_speed = lateral_speed * damping;

        **velocity = forward * forward_speed + right * new_lateral_speed;
    }

    // Update position
    *position += **velocity * delta;
}
