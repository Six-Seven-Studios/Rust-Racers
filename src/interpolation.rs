use bevy::prelude::*;
use crate::game_logic::{Velocity, Orientation};
use crate::multiplayer::NetworkPlayer;

// Resource to track adaptive interpolation delay
#[derive(Resource)]
pub struct InterpolationDelay {
    pub delay: f32,
    packet_intervals: [f32; 10],
    interval_index: usize,
    consecutive_underruns: u32,
    consecutive_overruns: u32,
}

impl Default for InterpolationDelay {
    fn default() -> Self {
        Self {
            delay: 0.030,
            packet_intervals: [0.016; 10],
            interval_index: 0,
            consecutive_underruns: 0,
            consecutive_overruns: 0,
        }
    }
}

impl InterpolationDelay {
    pub fn record_packet_interval(&mut self, interval: f32) {
        self.packet_intervals[self.interval_index] = interval;
        self.interval_index = (self.interval_index + 1) % self.packet_intervals.len();
    }

    pub fn calculate_jitter(&self) -> (f32, f32) {
        let sum: f32 = self.packet_intervals.iter().sum();
        let avg = sum / self.packet_intervals.len() as f32;
        let max = self.packet_intervals.iter().cloned().fold(0.0f32, f32::max);
        (avg, max)
    }

    pub fn adjust_delay(&mut self, alpha: f32) {
        const ADJUST_RATE: f32 = 0.001;
        const MIN_DELAY: f32 = 0.016;
        const MAX_DELAY: f32 = 0.100;

        if alpha >= 0.95 {
            self.consecutive_underruns += 1;
            self.consecutive_overruns = 0;

            if self.consecutive_underruns > 5 {
                self.delay = (self.delay + ADJUST_RATE).min(MAX_DELAY);
                self.consecutive_underruns = 0;
            }
        } else if alpha < 0.3 {
            self.consecutive_overruns += 1;
            self.consecutive_underruns = 0;

            if self.consecutive_overruns > 30 {
                self.delay = (self.delay - ADJUST_RATE).max(MIN_DELAY);
                self.consecutive_overruns = 0;
            }
        } else {
            self.consecutive_underruns = 0;
            self.consecutive_overruns = 0;
        }

        let (avg_interval, _) = self.calculate_jitter();
        let jitter_based_delay = avg_interval * 1.5;

        self.delay = self.delay * 0.95 + jitter_based_delay * 0.05;
        self.delay = self.delay.clamp(MIN_DELAY, MAX_DELAY);
    }
}

// Buffers two consecutive server states
#[derive(Component)]
pub struct InterpolationBuffer {
    pub prev_position: Vec2,
    pub prev_angle: f32,
    pub prev_velocity: Vec2,
    pub prev_timestamp: f32,

    pub curr_position: Vec2,
    pub curr_angle: f32,
    pub curr_velocity: Vec2,
    pub curr_timestamp: f32,

    pub initialized: bool,
}

impl InterpolationBuffer {
    pub fn new(x: f32, y: f32, angle: f32, vx: f32, vy: f32, timestamp: f32) -> Self {
        Self {
            prev_position: Vec2::new(x, y),
            prev_angle: angle,
            prev_velocity: Vec2::new(vx, vy),
            prev_timestamp: timestamp,
            curr_position: Vec2::new(x, y),
            curr_angle: angle,
            curr_velocity: Vec2::new(vx, vy),
            curr_timestamp: timestamp,
            initialized: false,
        }
    }

    pub fn push_state(&mut self, x: f32, y: f32, angle: f32, vx: f32, vy: f32, timestamp: f32) {
        self.prev_position = self.curr_position;
        self.prev_angle = self.curr_angle;
        self.prev_velocity = self.curr_velocity;
        self.prev_timestamp = self.curr_timestamp;

        self.curr_position = Vec2::new(x, y);
        self.curr_angle = angle;
        self.curr_velocity = Vec2::new(vx, vy);
        self.curr_timestamp = timestamp;

        self.initialized = true;
    }
}

// Interpolates networked car positions between server updates
pub fn interpolate_networked_cars(
    mut network_cars: Query<(&InterpolationBuffer, &mut Transform, &mut Orientation, &mut Velocity), With<NetworkPlayer>>,
    time: Res<Time>,
    mut interp_delay: ResMut<InterpolationDelay>,
) {
    let current_time = time.elapsed_secs();

    // Use adaptive render delay (dynamically adjusts based on network conditions)
    let render_delay = interp_delay.delay;
    let render_time = current_time - render_delay;

    for (buffer, mut transform, mut orientation, mut velocity) in network_cars.iter_mut() {
        if !buffer.initialized {
            transform.translation.x = buffer.curr_position.x;
            transform.translation.y = buffer.curr_position.y;
            transform.rotation = Quat::from_rotation_z(buffer.curr_angle);
            orientation.angle = buffer.curr_angle;
            velocity.velocity = buffer.curr_velocity;
            continue;
        }

        let target_duration = buffer.curr_timestamp - buffer.prev_timestamp;
        let time_since_prev = render_time - buffer.prev_timestamp;
        let alpha = if target_duration > 0.0 {
            (time_since_prev / target_duration).clamp(0.0, 1.0)
        } else {
            1.0
        };

        // Adjust delay based on alpha performance
        interp_delay.adjust_delay(alpha);

        let interpolated_pos = hermite_position(
            buffer.prev_position,
            buffer.curr_position,
            buffer.prev_velocity,
            buffer.curr_velocity,
            alpha,
            target_duration
        );

        transform.translation.x = interpolated_pos.x;
        transform.translation.y = interpolated_pos.y;

        let interpolated_angle = interpolate_angle(buffer.prev_angle, buffer.curr_angle, alpha);
        transform.rotation = Quat::from_rotation_z(interpolated_angle);
        orientation.angle = interpolated_angle;

        velocity.velocity = buffer.curr_velocity;
    }
}

// Interpolation methods
fn hermite_position(p0: Vec2, p1: Vec2, v0: Vec2, v1: Vec2, alpha: f32, duration: f32) -> Vec2 {
    let tangent_from = v0 * duration;
    let tangent_to = v1 * duration;

    let t2 = alpha * alpha;
    let t3 = t2 * alpha;

    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + alpha;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;

    p0 * h00 + tangent_from * h10 + p1 * h01 + tangent_to * h11
}

fn interpolate_angle(from: f32, to: f32, alpha: f32) -> f32 {
    use std::f32::consts::PI;

    let mut diff = to - from;
    while diff > PI {
        diff -= 2.0 * PI;
    }
    while diff < -PI {
        diff += 2.0 * PI;
    }

    from + diff * alpha
}
