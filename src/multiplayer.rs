use bevy::prelude::*;
use bevy::input::ButtonInput;
use crate::game_logic::{Car, Velocity, Orientation, PlayerControlled, CAR_SIZE, LapCounter, FIXED_TIMESTEP, TILE_SIZE, GameMap, apply_physics};
use crate::networking_plugin::{NetworkClient, PlayerPositions};
use crate::client_prediction::PredictionBuffer;
use crate::interpolation::{InterpolationDelay, InterpolationBuffer};

#[derive(Component)]
pub struct NetworkPlayer {
    pub player_id: u32,
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
    mut interp_delay: ResMut<InterpolationDelay>,
) {
    if network_client.client.is_none() { return }
    let my_id = network_client.player_id;
    let current_time = time.elapsed_secs();

    // Process all positions from the resource
    for (id, player_pos) in &player_positions.positions {
        // Reconcile our own player with server state
        if Some(*id) == my_id {
            if let Ok((mut transform, mut velocity, mut orientation, mut buffer)) = player_car.single_mut() {
                // Step 1: Use the server sequence number to get the inputs after it
                let last_ack_sequence = player_pos.last_processed_sequence;
                buffer.states.retain(|state| state.sequence > last_ack_sequence);

                // Step 2: Create local variables starting from server's authoritative state
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
            &mut interp_delay,
        );
    }
}

fn buffer_networked_car(
    network_cars: &mut Query<(&NetworkPlayer, &mut InterpolationBuffer)>,
    id: u32,
    x: f32, y: f32, vx: f32, vy: f32, angle: f32,
    timestamp: f32,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    interp_delay: &mut InterpolationDelay,
) {
    // Try to find existing car and buffer the new state
    for (net_player, mut buffer) in network_cars.iter_mut() {
        if net_player.player_id == id {
            // Calculate interval since last update and record it
            let interval = timestamp - buffer.curr_timestamp;
            if interval > 0.0 {
                interp_delay.record_packet_interval(interval);
            }
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

