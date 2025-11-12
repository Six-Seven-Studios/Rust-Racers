use bevy::prelude::*;
use bevy::input::ButtonInput;
use crate::game_logic::{Car, Velocity, Orientation, PlayerControlled, CAR_SIZE, LapCounter, FIXED_TIMESTEP, TILE_SIZE, GameMap, apply_physics};
use crate::networking_plugin::{NetworkClient, PlayerPositions};
use crate::client_prediction::PredictionBuffer;

#[derive(Component)]
pub struct NetworkPlayer {
    pub player_id: u32,
}

// Buffers two consecutive server states for smooth client-side interpolation
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

pub fn get_car_positions(
    network_client: Res<NetworkClient>,
    mut network_cars: Query<(&NetworkPlayer, &mut InterpolationBuffer)>,
    mut player_car: Query<(&mut Transform, &mut Velocity, &mut Orientation, &mut PredictionBuffer), (With<PlayerControlled>, Without<NetworkPlayer>)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    player_positions: Res<PlayerPositions>,
    time: Res<Time>,
    game_map: Res<GameMap>,
) {
    if network_client.client.is_none() { return }
    let my_id = network_client.player_id;
    let current_time = time.elapsed_secs();

    // Process all positions from the resource
    for (id, player_pos) in &player_positions.positions {
        // Reconcile our own player with server state (client-side prediction)
        if Some(*id) == my_id {
            if let Ok((mut transform, mut velocity, mut orientation, mut buffer)) = player_car.single_mut() {
                let last_ack_sequence = player_pos.last_processed_sequence;

                // Step 1: Remove acknowledged inputs from buffer
                buffer.states.retain(|state| state.sequence > last_ack_sequence);

                // Step 2: Create local variables starting from server's authoritative state
                // We don't mutate the player's actual components until we have the final result
                let mut replay_pos = Vec2::new(player_pos.x, player_pos.y);
                let mut replay_vel = Velocity::from(Vec2::new(player_pos.vx, player_pos.vy));
                let mut replay_orient = Orientation::new(player_pos.angle);

                // Step 3: Replay all unacknowledged inputs using local variables
                if !buffer.states.is_empty() {
                    for predicted_state in &buffer.states {
                        let tile = game_map.get_tile(replay_pos.x, replay_pos.y, TILE_SIZE as f32);

                        apply_physics(
                            &mut replay_pos,
                            &mut replay_vel,
                            &mut replay_orient,
                            &predicted_state.input,
                            FIXED_TIMESTEP,
                            tile.speed_modifier,
                            tile.friction_modifier,
                            tile.turn_modifier,
                            tile.decel_modifier,
                        );
                    }
                }

                // Step 4: Update everything at once with final replayed values
                // This is the only place we modify the player's actual state
                transform.translation = replay_pos.extend(transform.translation.z);
                transform.rotation = Quat::from_rotation_z(replay_orient.angle);
                velocity.velocity = replay_vel.velocity;
                orientation.angle = replay_orient.angle;
            }
            continue;
        }

        // Buffer states for networked cars
        buffer_networked_car(
            &mut network_cars,
            *id,
            player_pos.x,
            player_pos.y,
            player_pos.vx,
            player_pos.vy,
            player_pos.angle,
            current_time,
            &mut commands,
            &asset_server,
            &mut texture_atlases,
        );
    }
}

pub fn interpolate_networked_cars(
    mut network_cars: Query<(&InterpolationBuffer, &mut Transform, &mut Orientation, &mut Velocity), With<NetworkPlayer>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    // Render delay: Set to ~1.5x your server tick rate (e.g., 80ms tick → 120ms delay)
    const RENDER_DELAY: f32 = 0.080;
    let render_time = current_time - RENDER_DELAY;

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

fn buffer_networked_car(
    network_cars: &mut Query<(&NetworkPlayer, &mut InterpolationBuffer)>,
    id: u32,
    x: f32, y: f32, vx: f32, vy: f32, angle: f32,
    timestamp: f32,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    // Try to find existing car and buffer the new state
    for (net_player, mut buffer) in network_cars.iter_mut() {
        if net_player.player_id == id {
            buffer.push_state(x, y, angle, vx, vy, timestamp);
            return;
        }
    }

    // Spawn new car for this player with interpolation buffer
    let car_layout = TextureAtlasLayout::from_grid(UVec2::splat(CAR_SIZE), 2, 2, None, None);
    commands.spawn((
        Sprite::from_atlas_image(
            asset_server.load("red-car.png"),
            TextureAtlas { layout: texture_atlases.add(car_layout), index: 0 },
        ),
        Transform::from_xyz(x, y, 10.).with_rotation(Quat::from_rotation_z(angle)),
        Velocity::from(Vec2::new(vx, vy)),
        Orientation::new(angle),
        Car,
        NetworkPlayer { player_id: id },
        InterpolationBuffer::new(x, y, angle, vx, vy, timestamp),
        LapCounter::default(),
    ));
}

