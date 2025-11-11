use bevy::prelude::*;
use bevy::input::ButtonInput;
use crate::game_logic::{Car, Velocity, Orientation, PlayerControlled, CAR_SIZE, LapCounter, GameMap};
use crate::networking_plugin::{NetworkClient, PlayerPositions};
use crate::prediction::{
    ClientPredictionState, StateSnapshot, ReconciliationEngine,
    SmoothCorrection, ENABLE_PREDICTION,
};

#[derive(Component)]
pub struct NetworkPlayer {
    pub player_id: u32,
}

pub fn send_keyboard_input(
    mut network_client: ResMut<NetworkClient>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Some(client) = network_client.client.as_mut() else { return };

    let forward = input.pressed(KeyCode::KeyW);
    let backward = input.pressed(KeyCode::KeyS);
    let left = input.pressed(KeyCode::KeyA);
    let right = input.pressed(KeyCode::KeyD);
    let drift = input.pressed(KeyCode::Space);

    let _ = client.send_player_input(forward, backward, left, right, drift);
}

pub fn get_car_positions(
    network_client: Res<NetworkClient>,
    mut network_cars: Query<(&NetworkPlayer, &mut Transform, &mut Velocity, &mut Orientation)>,
    mut player_car: Query<
        (Entity, &mut Transform, &mut Velocity, &mut Orientation),
        (With<PlayerControlled>, Without<NetworkPlayer>)
    >,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    player_positions: Res<PlayerPositions>,
    mut prediction_state: ResMut<ClientPredictionState>,
    game_map: Res<GameMap>,
    time: Res<Time>,
) {
    if network_client.client.is_none() { return }
    let my_id = network_client.player_id;

    // Process all positions from the resource
    for (id, player_pos) in &player_positions.positions {
        // Update our own player car with reconciliation
        if Some(*id) == my_id {
            if ENABLE_PREDICTION {
                // Reconcile with server state
                if let Ok((entity, mut transform, mut velocity, mut orientation)) = player_car.get_single_mut() {
                    reconcile_with_server(
                        &mut transform,
                        &mut velocity,
                        &mut orientation,
                        player_pos,
                        &mut prediction_state,
                        &game_map,
                        time.delta_secs(),
                        &mut commands,
                        entity,
                    );
                }
            } else {
                // No prediction: directly apply server state
                if let Ok((_entity, mut transform, mut velocity, mut orientation)) = player_car.get_single_mut() {
                    transform.translation = Vec3::new(player_pos.x, player_pos.y, transform.translation.z);
                    transform.rotation = Quat::from_rotation_z(player_pos.angle);
                    velocity.velocity = Vec2::new(player_pos.vx, player_pos.vy);
                    orientation.angle = player_pos.angle;
                }
            }
            continue;
        }

        // Update other players
        compensate_lag(
            &mut network_cars,
            *id,
            player_pos.x,
            player_pos.y,
            player_pos.vx,
            player_pos.vy,
            player_pos.angle,
            &mut commands,
            &asset_server,
            &mut texture_atlases
        );
    }
}

/// Reconcile client prediction with authoritative server state
///
/// This is where the magic happens:
/// 1. Create a snapshot from server data
/// 2. Re-simulate from server state using pending inputs
/// 3. Compare with what we predicted
/// 4. If error is significant, apply smooth correction
fn reconcile_with_server(
    transform: &mut Transform,
    velocity: &mut Velocity,
    orientation: &mut Orientation,
    server_data: &crate::networking::PlayerPositionData,
    prediction_state: &mut ClientPredictionState,
    game_map: &GameMap,
    delta: f32,
    commands: &mut Commands,
    entity: Entity,
) {
    // Create server state snapshot
    let server_state = StateSnapshot::from_server_data(
        server_data.x,
        server_data.y,
        server_data.vx,
        server_data.vy,
        server_data.angle,
        server_data.input_count,
        prediction_state.game_time,
    );

    // Store last server state
    prediction_state.last_server_state = Some(server_state.clone());

    // Get our predicted state at the same sequence number
    let predicted_state = prediction_state.get_predicted_state(server_data.input_count);

    // Check if we need correction
    let needs_correction = if let Some(predicted) = predicted_state {
        let (needs_corr, error) = ReconciliationEngine::needs_correction(predicted, &server_state);

        if needs_corr {
            println!("[Prediction] Error detected: {:.2} pixels at sequence {}", error, server_data.input_count);
        }

        needs_corr
    } else {
        // No predicted state to compare, use server state
        false
    };

    if needs_correction {
        // Re-simulate from server state using pending inputs
        let (corrected_state, _was_corrected, _error) = ReconciliationEngine::reconcile(
            &server_state,
            &prediction_state.input_buffer,
            game_map,
            delta,
        );

        // Calculate the visual error offset (where we were vs where we should be)
        let current_pos = Vec2::new(transform.translation.x, transform.translation.y);
        let error_offset = current_pos - corrected_state.position;

        // Apply corrected physics state immediately
        corrected_state.apply_to_components(transform, velocity, orientation);

        // Add smooth visual correction to hide the snap
        commands.entity(entity).insert(SmoothCorrection::start(error_offset));

        println!("[Prediction] Applied correction with offset: ({:.2}, {:.2})", error_offset.x, error_offset.y);
    } else {
        // Prediction was accurate or no prediction available
        // Re-simulate anyway to handle pending inputs
        let (corrected_state, _, _) = ReconciliationEngine::reconcile(
            &server_state,
            &prediction_state.input_buffer,
            game_map,
            delta,
        );

        corrected_state.apply_to_components(transform, velocity, orientation);
    }

    // Clean up old inputs that have been acknowledged
    prediction_state.input_buffer.clear_before(server_data.input_count.saturating_sub(30));
}

fn compensate_lag(
    network_cars: &mut Query<(&NetworkPlayer, &mut Transform, &mut Velocity, &mut Orientation)>,
    id: u32,
    x: f32, y: f32, vx: f32, vy: f32, angle: f32,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    // Update existing car or spawn new one
    for (net_player, mut transform, mut velocity, mut orientation) in network_cars.iter_mut() {
        if net_player.player_id == id {
            transform.translation = Vec3::new(x, y, transform.translation.z);
            transform.rotation = Quat::from_rotation_z(angle);
            velocity.velocity = Vec2::new(vx, vy);
            orientation.angle = angle;
            return;
        }
    }
    
    // Spawn new car for this player
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
        LapCounter::default(),
    ));
}

