use bevy::prelude::*;
use serde_json::json;
use std::collections::HashMap;

use crate::game_logic::{
    AIControlled, CAR_SIZE, GameMap, Orientation, SERVER_TIMESTEP, START_ORIENTATION, TILE_SIZE,
    Velocity, handle_collision,
    physics::{PhysicsInput, apply_physics},
    theta::{ThetaCheckpointList, theta_star_pursuit, ThetaCommand},
    theta_grid::ThetaGrid,
};
use crate::lobby_management::timeout_cleanup;
use crate::types::*;

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
) {

    // Check which lobbies have started
    let started_lobbies: Vec<String> = {
        let guard = lobbies.list.lock().unwrap();
        guard
            .iter()
            .filter(|l| l.started)
            .map(|l| l.name.clone())
            .collect()
    };

    // Snapshot positions/velocities for collision checks without aliasing the query
    let player_snapshots: Vec<(u32, String, Vec2, Vec2)> = query
        .iter()
        .map(|(player_id, pos, vel, _, _, lobby_member)| {
            (
                player_id.0,
                lobby_member.lobby_name.clone(),
                Vec2::new(pos.x, pos.y),
                vel.velocity,
            )
        })
        .collect();

    // Process each player
    for (player_id, mut pos, mut vel, mut orient, mut input_component, lobby_member) in
        query.iter_mut()
    {
        // Only simulate physics for players in started lobbies
        if !started_lobbies.contains(&lobby_member.lobby_name) {
            continue;
        }

        // Find the lobby to access input queue
        let guard = lobbies.list.lock().unwrap();
        let lobby_opt = guard.iter().find(|l| l.name == lobby_member.lobby_name);

        if let Some(lobby) = lobby_opt {
            let mut states = lobby.states.lock().unwrap();
            
            // set the map to this lobby's map
            let game_map = &lobby.map;

            if let Some(player_state) = states.get_mut(&player_id.0) {
                // Process all inputs in the queue
                let inputs_to_process: Vec<InputData> =
                    player_state.input_queue.drain(..).collect();

                for input_data in inputs_to_process {
                    // Refresh boost timer when client reports a pickup
                    if input_data.boost && player_state.boost_remaining <= 0.0 {
                        player_state.boost_remaining = 5.0;
                    }
                    if player_state.boost_remaining > 0.0 {
                        player_state.boost_remaining = (player_state.boost_remaining
                            - crate::game_logic::CLIENT_TIMESTEP)
                            .max(0.0);
                    }

                    let prev_pos = Vec2::new(pos.x, pos.y);

                    // Get current tile and terrain modifiers
                    let tile = game_map.get_tile(pos.x, pos.y, TILE_SIZE as f32);

                    // Create physics input
                    let physics_input = PhysicsInput {
                        forward: input_data.forward,
                        backward: input_data.backward,
                        left: input_data.left,
                        right: input_data.right,
                        drift: input_data.drift,
                        easy_drift: input_data.easy_drift,
                        boost: player_state.boost_remaining > 0.0,
                    };

                    // Apply physics for this input (using CLIENT_TIMESTEP since inputs are at 60 Hz)
                    let mut position_vec = Vec2::new(pos.x, pos.y);
                    apply_physics(
                        &mut position_vec,
                        &mut vel,
                        &mut orient,
                        &physics_input,
                        crate::game_logic::CLIENT_TIMESTEP, // Use client timestep for each input
                        tile.speed_modifier,
                        tile.friction_modifier,
                        tile.turn_modifier,
                        tile.decel_modifier,
                    );

                    // Apply map boundaries
                    let half_width = game_map.width / 2.0;
                    let half_height = game_map.height / 2.0;
                    let car_half_size = (CAR_SIZE as f32) / 2.0;

                    position_vec.x = position_vec
                        .x
                        .clamp(-half_width + car_half_size, half_width - car_half_size);
                    position_vec.y = position_vec
                        .y
                        .clamp(-half_height + car_half_size, half_height - car_half_size);

                    // Resolve collisions against walls/other cars (same lobby only)
                    let other_cars_iter = player_snapshots
                        .iter()
                        .filter(|(other_id, lobby_name, _, _)| {
                            *other_id != player_id.0 && *lobby_name == lobby_member.lobby_name
                        })
                        .map(|(_, _, other_pos, other_vel)| (*other_pos, *other_vel));
                    let should_update = handle_collision(
                        position_vec.extend(0.0),
                        prev_pos,
                        &mut vel.velocity,
                        &game_map,
                        other_cars_iter,
                    );

                    // Update position
                    if should_update {
                        pos.x = position_vec.x;
                        pos.y = position_vec.y;
                    }

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
                        easy_drift: input_data.easy_drift,
                        boost: player_state.boost_remaining > 0.0,
                    };
                }

                // Update input component with latest state
                input_component.last_processed_sequence = player_state.last_processed_sequence;
                input_component.forward = player_state.inputs.forward;
                input_component.backward = player_state.inputs.backward;
                input_component.left = player_state.inputs.left;
                input_component.right = player_state.inputs.right;
                input_component.drift = player_state.inputs.drift;
                input_component.easy_drift = player_state.inputs.easy_drift;
                input_component.boost = player_state.inputs.boost;
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
    let mut lobby_players: HashMap<
        String,
        Vec<(
            u32,
            &Position,
            &Velocity,
            &Orientation,
            &PlayerInputComponent,
        )>,
    > = HashMap::new();

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
            let positions_json: Vec<_> = players_data
                .iter()
                .map(|(id, pos, vel, orient, input)| {
                    json!({
                        "id": id,
                        "x": pos.x,
                        "y": pos.y,
                        "vx": vel.x,
                        "vy": vel.y,
                        "angle": orient.angle,
                        "last_processed_sequence": input.last_processed_sequence
                    })
                })
                .collect();

            let payload = json!({
                "type": "game_state_update",
                "players": positions_json
            })
            .to_string()
                + "\n";

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
            ServerCommand::SpawnPlayer {
                player_id,
                lobby_name,
                x,
                y,
            } => {
                println!("Spawning player {} in lobby {}", player_id, lobby_name);

                let entity = commands
                    .spawn((
                        PlayerId(player_id),
                        Position { x, y },
                        Velocity::new(),
                        Orientation::new(START_ORIENTATION),
                        PlayerInputComponent::default(),
                        LobbyMember { lobby_name },
                    ))
                    .id();

                player_entities.map.insert(player_id, entity);
            }
            ServerCommand::SpawnAI {
                ai_id,
                lobby_name,
                x,
                y,
                angle,
            } => {
                println!("Spawning AI {} in lobby {}", ai_id, lobby_name);

                // Load checkpoints for map 1
                let mut checkpoint_list = ThetaCheckpointList::new(Vec::new());
                checkpoint_list = checkpoint_list.load_checkpoint_list(1);

                let entity = commands
                    .spawn((
                        PlayerId(ai_id),
                        Position { x, y },
                        Velocity::new(),
                        Orientation::new(angle),
                        PlayerInputComponent::default(),
                        LobbyMember { lobby_name },
                        AIControlled,
                        checkpoint_list,
                    ))
                    .id();

                player_entities.map.insert(ai_id, entity);
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
    timeout_cleanup(
        &connected_clients,
        &lobbies.list,
        TIMEOUT_SECONDS,
        &cmd_sender.sender,
    );
}

/// System to move AI cars using Theta* pathfinding
pub fn ai_movement_system(
    game_map: Res<GameMap>,
    theta_grid:
    Res<ThetaGrid>,
    lobbies: Res<Lobbies>,
    mut ai_cars: Query<
        (
            &mut Position,
            &mut Velocity,
            &mut Orientation,
            &mut ThetaCheckpointList,
            &LobbyMember,
        ),
        With<AIControlled>,
    >,
    other_cars: Query<(&Position, &Velocity), Without<AIControlled>>,
) {
    // Check which lobbies have started
    let started_lobbies: Vec<String> = {
        let guard = lobbies.list.lock().unwrap();
        guard
            .iter()
            .filter(|l| l.started)
            .map(|l| l.name.clone())
            .collect()
    };

    // AI physics constants (same as player)
    const ACCEL_RATE: f32 = 400.0;
    const TURNING_RATE: f32 = 3.0;
    const PLAYER_SPEED: f32 = 300.0;

    let deltat = SERVER_TIMESTEP;
    let accel = ACCEL_RATE * deltat;

    for (mut pos, mut velocity, mut orientation, mut theta_checkpoint_list, lobby_member) in
        ai_cars.iter_mut()
    {
        // Only simulate AI in started lobbies
        if !started_lobbies.contains(&lobby_member.lobby_name) {
            continue;
        }

        // Find the lobby to access input queue (same thing as above basically)
        let game_map = {
            let guard = lobbies.list.lock().unwrap();
            let lobby_opt = guard.iter().find(|l| l.name == lobby_member.lobby_name);
            
            if let Some(lobby) = lobby_opt {
                lobby.map.clone() 
            } else {
                GameMap::default()
            }
        };

        // Get the current tile
        let tile = game_map.get_tile(pos.x, pos.y, TILE_SIZE as f32);

        // Get terrain modifiers
        let fric_mod = tile.friction_modifier;
        let speed_mod = tile.speed_modifier;
        let turn_mod = tile.turn_modifier;
        let decel_mod = tile.decel_modifier;

        // Get command from Theta* pathfinding
        let command = theta_star_pursuit(
            (tile.x_coordinate, tile.y_coordinate),
            orientation.angle,
            &mut theta_checkpoint_list,
            &theta_grid,
        );

        // COPIED FROM src/car.rs
        // Execute the command
        match command {
            ThetaCommand::TurnLeft => {
                orientation.angle += TURNING_RATE * deltat * turn_mod;
            }
            ThetaCommand::TurnRight => {
                orientation.angle -= TURNING_RATE * deltat * turn_mod;
            }
            ThetaCommand::Forward => {
                let forward = orientation.forward_vector() * accel;
                **velocity += forward;
                **velocity = velocity.clamp_length_max(PLAYER_SPEED * speed_mod);
            }
            ThetaCommand::Reverse => {
                let backward = -orientation.forward_vector() * (accel / 2.0);
                **velocity += backward;
                **velocity = velocity.clamp_length_max(PLAYER_SPEED * (speed_mod / 2.0));
            }
            ThetaCommand::Stop => {
                if velocity.length() > 0.0 {
                    let backward = -orientation.forward_vector() * (accel / 2.0);
                    **velocity += backward;
                    **velocity = velocity.clamp_length_max(PLAYER_SPEED * (speed_mod / 2.0));
                } else {
                    **velocity = Vec2::ZERO;
                }
            }
        }

        // Apply friction when not accelerating forward or reversing
        if !matches!(command, ThetaCommand::Forward | ThetaCommand::Reverse) {
            let decel_rate = decel_mod * fric_mod * deltat;
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

        // Update position
        let change_x = velocity.x * deltat;
        let change_y = velocity.y * deltat;

        let new_x = (pos.x + change_x).clamp(
            -game_map.width / 2.0 + (CAR_SIZE as f32) / 2.0,
            game_map.width / 2.0 - (CAR_SIZE as f32) / 2.0,
        );
        let new_y = (pos.y + change_y).clamp(
            -game_map.height / 2.0 + (CAR_SIZE as f32) / 2.0,
            game_map.height / 2.0 - (CAR_SIZE as f32) / 2.0,
        );

        // Handle collision detection
        let new_pos_vec = Vec2::new(new_x, new_y);
        let current_pos = Vec2::new(pos.x, pos.y);
        let other_cars_iter = other_cars
            .iter()
            .map(|(p, v)| (Vec2::new(p.x, p.y), v.velocity));

        let should_update = handle_collision(
            new_pos_vec.extend(0.0),
            current_pos,
            &mut velocity.velocity,
            &game_map,
            other_cars_iter,
        );

        if should_update {
            pos.x = new_x;
            pos.y = new_y;
        }
    }
}
