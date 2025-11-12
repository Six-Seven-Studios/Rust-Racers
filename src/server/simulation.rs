use bevy::prelude::*;
use serde_json::json;
use std::collections::HashMap;

use crate::game_logic::{
    CAR_SIZE, TILE_SIZE, FIXED_TIMESTEP,
    GameMap,
    physics::{PhysicsInput, apply_physics},
    Velocity, Orientation,
    handle_collision,
};
use crate::types::*;
use crate::lobby_management::timeout_cleanup;

/// System to apply physics simulation to all active players
/// Uses terrain modifiers, collision detection, and map boundaries
/// Runs at fixed 60 Hz (FIXED_TIMESTEP) in FixedUpdate schedule
pub fn physics_simulation_system(
    mut query: Query<(
        &PlayerId,
        &mut Position,
        &mut Velocity,
        &mut Orientation,
        &PlayerInputComponent,
        &LobbyMember,
    )>,
    lobbies: Res<Lobbies>,
    game_map: Res<GameMap>,
) {

    // Check which lobbies have started
    let started_lobbies: Vec<String> = {
        let guard = lobbies.list.lock().unwrap();
        guard.iter()
            .filter(|l| l.started)
            .map(|l| l.name.clone())
            .collect()
    };

    // Collect all player data for collision detection
    let player_data: Vec<(Vec3, Vec2)> = query.iter()
        .filter(|(_, _, _, _, _, lobby_member)| started_lobbies.contains(&lobby_member.lobby_name))
        .map(|(_, pos, vel, _, _, _)| (Vec3::new(pos.x, pos.y, 0.0), **vel))
        .collect();

    for (player_id, mut pos, mut vel, mut orient, input, lobby_member) in query.iter_mut() {
        // Only simulate physics for players in started lobbies
        if !started_lobbies.contains(&lobby_member.lobby_name) {
            continue;
        }

        // Get current tile and terrain modifiers
        let tile = game_map.get_tile(pos.x, pos.y, TILE_SIZE as f32);
        let fric_mod = tile.friction_modifier;
        let speed_mod = tile.speed_modifier;
        let turn_mod = tile.turn_modifier;
        let decel_mod = tile.decel_modifier;

        // Create physics input from player input component
        let physics_input = PhysicsInput {
            forward: input.forward,
            backward: input.backward,
            left: input.left,
            right: input.right,
            drift: input.drift,
        };

        // Create a Vec2 position for physics calculation
        let mut position_vec = Vec2::new(pos.x, pos.y);

        // Apply shared physics logic with fixed timestep (deterministic)
        apply_physics(
            &mut position_vec,
            &mut vel,
            &mut orient,
            &physics_input,
            FIXED_TIMESTEP,
            speed_mod,
            fric_mod,
            turn_mod,
            decel_mod,
        );

        // Apply map boundaries
        let half_width = game_map.width / 2.0;
        let half_height = game_map.height / 2.0;
        let car_half_size = (CAR_SIZE as f32) / 2.0;

        position_vec.x = position_vec.x.clamp(-half_width + car_half_size, half_width - car_half_size);
        position_vec.y = position_vec.y.clamp(-half_height + car_half_size, half_height - car_half_size);

        let new_position_3d = position_vec.extend(0.0);
        let current_position_2d = Vec2::new(pos.x, pos.y);

        // Convert player_data Vec to iterator of (position, velocity) pairs
        let other_players_iter = player_data.iter().map(|(p, v)| (p.truncate(), *v));

        let should_update = handle_collision(
            new_position_3d,
            current_position_2d,
            &mut **vel,
            &game_map,
            other_players_iter,
        );

        // Update position if collision allows it
        if should_update {
            pos.x = position_vec.x;
            pos.y = position_vec.y;
        }
    }
}

/// System to broadcast game state to all clients
pub fn broadcast_state_system(
    query: Query<(
        &PlayerId,
        &Position,
        &Velocity,
        &Orientation,
        &PlayerInputComponent,
        &LobbyMember,
    )>,
    connected_clients: Res<ConnectedClients>,
    lobbies: Res<Lobbies>,
) {
    // Group players by lobby
    let mut lobby_players: HashMap<String, Vec<(u32, &Position, &Velocity, &Orientation, &PlayerInputComponent)>> = HashMap::new();

    for (player_id, pos, vel, orient, input, lobby_member) in query.iter() {
        lobby_players
            .entry(lobby_member.lobby_name.clone())
            .or_insert_with(Vec::new)
            .push((player_id.0, pos, vel, orient, input));
    }

    // Broadcast state for each started lobby
    let guard = lobbies.list.lock().unwrap();
    for lobby in guard.iter() {
        if !lobby.started {
            continue;
        }

        if let Some(players_data) = lobby_players.get(&lobby.name) {
            // Build positions payload
            let positions_json: Vec<_> = players_data.iter().map(|(id, pos, vel, orient, input)| {
                json!({
                    "id": id,
                    "x": pos.x,
                    "y": pos.y,
                    "vx": vel.x,
                    "vy": vel.y,
                    "angle": orient.angle,
                    "last_processed_sequence": input.last_processed_sequence
                })
            }).collect();

            let payload = json!({
                "type": "game_state_update",
                "players": positions_json
            }).to_string() + "\n";

            // Get player IDs in this lobby
            let lobby_player_ids: Vec<u32> = lobby.players.lock().unwrap().clone();

            // Send to all players in lobby
            let addrs = connected_clients.addrs.lock().unwrap();
            for pid in &lobby_player_ids {
                if let Some(addr) = addrs.get(pid) {
                    let _ = connected_clients.socket.send_to(payload.as_bytes(), addr);
                }
            }
        }
    }
}

/// System to update player input components from the lobby states
pub fn sync_input_from_lobbies_system(
    mut query: Query<(&PlayerId, &mut PlayerInputComponent, &LobbyMember)>,
    lobbies: Res<Lobbies>,
) {
    let guard = lobbies.list.lock().unwrap();

    for (player_id, mut input_component, lobby_member) in query.iter_mut() {
        // Find the lobby this player belongs to
        if let Some(lobby) = guard.iter().find(|l| l.name == lobby_member.lobby_name) {
            let states = lobby.states.lock().unwrap();
            if let Some(state) = states.get(&player_id.0) {
                // Sync input from lobby state to component
                input_component.forward = state.inputs.forward;
                input_component.backward = state.inputs.backward;
                input_component.left = state.inputs.left;
                input_component.right = state.inputs.right;
                input_component.drift = state.inputs.drift;
                input_component.last_processed_sequence = state.last_processed_sequence;
            }
        }
    }
}

/// System to process commands from networking threads (spawn/despawn players)
pub fn process_server_commands_system(
    mut commands: Commands,
    receiver: Res<ServerCommandReceiver>,
    mut player_entities: ResMut<PlayerEntities>,
) {
    // Process all pending commands
    let recv = receiver.receiver.lock().unwrap();
    while let Ok(command) = recv.try_recv() {
        match command {
            ServerCommand::SpawnPlayer { player_id, lobby_name, x, y } => {
                println!("Spawning player {} in lobby {}", player_id, lobby_name);

                let entity = commands.spawn((
                    PlayerId(player_id),
                    Position { x, y },
                    Velocity::new(),
                    Orientation::new(0.0),
                    PlayerInputComponent::default(),
                    LobbyMember { lobby_name },
                )).id();

                player_entities.map.insert(player_id, entity);
            }
            ServerCommand::DespawnPlayer { player_id } => {
                if let Some(entity) = player_entities.map.remove(&player_id) {
                    println!("Despawning player {}", player_id);
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

/// System to check for timed out clients and disconnect them
pub fn timeout_cleanup_system(
    connected_clients: Res<ConnectedClients>,
    lobbies: Res<Lobbies>,
) {
    const TIMEOUT_SECONDS: u64 = 10;
    timeout_cleanup(&connected_clients, &lobbies.list, TIMEOUT_SECONDS);
}
