use bevy::prelude::*;
use serde_json::json;
use std::collections::HashMap;

use crate::game_logic::{
    CAR_SIZE, TILE_SIZE, SERVER_TIMESTEP,
    GameMap,
    physics::{PhysicsInput, apply_physics},
    Velocity, Orientation,
    handle_collision,
};
use crate::types::*;
use crate::lobby_management::timeout_cleanup;

/// System to apply physics simulation to all active players
/// Processes all buffered inputs and generates position snapshots
/// Runs at 20 Hz
pub fn physics_simulation_system(
    mut query: Query<(
        &PlayerId,
        &mut Position,
        &mut Velocity,
        &mut Orientation,
        &mut PlayerInputComponent,
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

    // Process each player
    for (player_id, mut pos, mut vel, mut orient, mut input_component, lobby_member) in query.iter_mut() {
        // Only simulate physics for players in started lobbies
        if !started_lobbies.contains(&lobby_member.lobby_name) {
            continue;
        }

        // Find the lobby to access input queue
        let guard = lobbies.list.lock().unwrap();
        let lobby_opt = guard.iter().find(|l| l.name == lobby_member.lobby_name);

        if let Some(lobby) = lobby_opt {
            let mut states = lobby.states.lock().unwrap();

            if let Some(player_state) = states.get_mut(&player_id.0) {
                // Process all inputs in the queue
                let inputs_to_process: Vec<InputData> = player_state.input_queue.drain(..).collect();

                for input_data in inputs_to_process {
                    // Get current tile and terrain modifiers
                    let tile = game_map.get_tile(pos.x, pos.y, TILE_SIZE as f32);

                    // Create physics input
                    let physics_input = PhysicsInput {
                        forward: input_data.forward,
                        backward: input_data.backward,
                        left: input_data.left,
                        right: input_data.right,
                        drift: input_data.drift,
                    };

                    // Apply physics for this input (using CLIENT_TIMESTEP since inputs are at 60 Hz)
                    let mut position_vec = Vec2::new(pos.x, pos.y);
                    apply_physics(
                        &mut position_vec,
                        &mut vel,
                        &mut orient,
                        &physics_input,
                        crate::game_logic::CLIENT_TIMESTEP,  // Use client timestep for each input
                        tile.speed_modifier,
                        tile.friction_modifier,
                        tile.turn_modifier,
                        tile.decel_modifier,
                    );

                    // Apply map boundaries
                    let half_width = game_map.width / 2.0;
                    let half_height = game_map.height / 2.0;
                    let car_half_size = (CAR_SIZE as f32) / 2.0;

                    position_vec.x = position_vec.x.clamp(-half_width + car_half_size, half_width - car_half_size);
                    position_vec.y = position_vec.y.clamp(-half_height + car_half_size, half_height - car_half_size);

                    // Update position
                    pos.x = position_vec.x;
                    pos.y = position_vec.y;

                    // Update player state with processed input
                    player_state.last_processed_sequence = input_data.sequence;
                    player_state.x = pos.x;
                    player_state.y = pos.y;
                    player_state.velocity = vel.velocity;
                    player_state.angle = orient.angle;
                    player_state.inputs = PlayerInput {
                        forward: input_data.forward,
                        backward: input_data.backward,
                        left: input_data.left,
                        right: input_data.right,
                        drift: input_data.drift,
                    };
                }

                // Update input component with latest state
                input_component.last_processed_sequence = player_state.last_processed_sequence;
                input_component.forward = player_state.inputs.forward;
                input_component.backward = player_state.inputs.backward;
                input_component.left = player_state.inputs.left;
                input_component.right = player_state.inputs.right;
                input_component.drift = player_state.inputs.drift;
            }
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
    cmd_sender: Res<ServerCommandSender>,
) {
    const TIMEOUT_SECONDS: u64 = 10;
    timeout_cleanup(&connected_clients, &lobbies.list, TIMEOUT_SECONDS, &cmd_sender.sender);
}
