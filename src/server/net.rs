use bevy::tasks::IoTaskPool;
use serde_json::json;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::game_logic::{START_ORIENTATION, START_POSITIONS, START_POSITIONS_MAP2, TILE_SIZE};
use crate::networking::MapChoice;
use crate::lobby_management::*;
use crate::types::*;
use crate::game_logic::{GameMap, load_map_from_file};
use crate::game_logic::theta_grid::ThetaGrid;

/// Spawn the UDP listener task that handles incoming client messages
pub fn server_listener(
    connected_clients: ConnectedClients,
    lobbies: LobbyList,
    cmd_sender: Arc<Mutex<std::sync::mpsc::Sender<ServerCommand>>>,
) {
    let task_pool = IoTaskPool::get();
    task_pool
        .spawn(async move {
            let mut next_id: u32 = 1;
            let mut buf = [0u8; 65536]; // UDP buffer

            loop {
                match connected_clients.socket.recv_from(&mut buf) {
                    Ok((len, addr)) => {
                        let data = &buf[..len];

                        // Convert bytes to string
                        if let Ok(message_str) = std::str::from_utf8(data) {
                            let trimmed = message_str.trim();
                            if trimmed.is_empty() {
                                continue;
                            }

                            // Get or assign client ID
                            let client_id = {
                                let mut addr_to_id = connected_clients.addr_to_id.lock().unwrap();

                                if let Some(&id) = addr_to_id.get(&addr) {
                                    // Update last seen time
                                    if let Ok(mut last_seen) = connected_clients.last_seen.lock() {
                                        last_seen.insert(id, Instant::now());
                                    }
                                    id
                                } else {
                                    // New client
                                    let id = next_id;
                                    next_id += 1;

                                    addr_to_id.insert(addr, id);

                                    // Add to other maps
                                    if let Ok(mut addrs) = connected_clients.addrs.lock() {
                                        addrs.insert(id, addr);
                                    }
                                    if let Ok(mut ids) = connected_clients.ids.lock() {
                                        ids.push(id);
                                    }
                                    if let Ok(mut last_seen) = connected_clients.last_seen.lock() {
                                        last_seen.insert(id, Instant::now());
                                    }

                                    println!("New client {} from {}", id, addr);

                                    // Send welcome message
                                    let welcome = format!("WELCOME PLAYER {}\n", id);
                                    let _ =
                                        connected_clients.socket.send_to(welcome.as_bytes(), addr);

                                    id
                                }
                            };

                            // Parse and handle message
                            match serde_json::from_str::<MessageType>(trimmed) {
                                Ok(message) => {
                                    if let Err(e) = handle_client_message(
                                        client_id,
                                        message,
                                        &connected_clients,
                                        &lobbies,
                                        &cmd_sender,
                                    ) {
                                        eprintln!(
                                            "handle_client_message error for {}: {}",
                                            client_id, e
                                        );
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "JSON parse error from {}: {}; raw={}",
                                        client_id, e, trimmed
                                    );
                                    let _ = send_to_client(
                                        client_id,
                                        &connected_clients,
                                        &json!({
                                            "type": "error",
                                            "message": "invalid_json"
                                        }),
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("recv_from error: {}", e);
                    }
                }
            }
        })
        .detach();
}

/// Handle incoming message from a client
fn handle_client_message(
    id: u32,
    message: MessageType,
    connected_clients: &ConnectedClients,
    lobbies: &LobbyList,
    cmd_sender: &Arc<Mutex<std::sync::mpsc::Sender<ServerCommand>>>,
) -> io::Result<()> {
    match message {
        MessageType::CreateLobby { name, map } => {
            let mut guard = lobbies.lock().unwrap();

            // Check if lobby already exists
            if guard.iter().any(|l| l.name == name) {
                let _ = send_to_client(
                    id,
                    connected_clients,
                    &json!({
                        "type": "error",
                        "message": format!("Lobby '{}' already exists", name)
                    }),
                );
                return Ok(());
            }

            // Create new lobby
            let mut new_lobby = Lobby::default();
            new_lobby.name = name.clone();
            new_lobby.host = id;
            new_lobby.players.lock().unwrap().push(id);
            // add game_map to new_lobby's data based on selection
            let game_map = load_map_from_file(map.path());
            println!(
                "Server loaded map ({:?}): {}x{}",
                map,
                game_map.width,
                game_map.height
            );
            new_lobby.map_choice = map;
            new_lobby.map = game_map;
            let grid_size = match map {
                MapChoice::Small => (100, 100),
                MapChoice::Big => (125, 125),
            };
            new_lobby.theta_grid = ThetaGrid::create_theta_grid_with_size(
                &new_lobby.map,
                TILE_SIZE as f32,
                grid_size.0,
                grid_size.1,
            );

            guard.push(new_lobby);

            let lobby_index = guard.len() - 1;
            drop(guard);

            broadcast_lobby_state(connected_clients, lobbies, lobby_index);
            broadcast_active_lobbies(connected_clients, lobbies);

            let _ = send_to_client(
                id,
                connected_clients,
                &json!({
                    "type": "confirmation",
                    "message": format!("You have created the lobby '{}'", name)
                }),
            );

            Ok(())
        }

        MessageType::JoinLobby { name } => {
            let mut guard = lobbies.lock().unwrap();

            let lobby_index_opt = guard.iter().position(|l| l.name == name);

            if let Some(lobby_index) = lobby_index_opt {
                let lobby = &mut guard[lobby_index];

                if lobby.started {
                    let _ = send_to_client(
                        id,
                        connected_clients,
                        &json!({
                            "type": "error",
                            "message": "Lobby has already started"
                        }),
                    );
                    return Ok(());
                }

                let mut players = lobby.players.lock().unwrap();
                if !players.contains(&id) {
                    players.push(id);
                }
                drop(players);
                drop(guard);

                broadcast_lobby_state(connected_clients, lobbies, lobby_index);
                broadcast_active_lobbies(connected_clients, lobbies);

                let _ = send_to_client(
                    id,
                    connected_clients,
                    &json!({
                        "type": "confirmation",
                        "message": format!("You have joined the lobby '{}'", name)
                    }),
                );
            } else {
                let _ = send_to_client(
                    id,
                    connected_clients,
                    &json!({
                        "type": "error",
                        "message": format!("Lobby '{}' does not exist", name)
                    }),
                );
            }

            Ok(())
        }

        MessageType::LeaveLobby { name } => {
            let mut guard = lobbies.lock().unwrap();
            let lobby_index_opt = guard.iter().position(|l| l.name == name);

            if let Some(lobby_index) = lobby_index_opt {
                let lobby = &mut guard[lobby_index];
                let mut players = lobby.players.lock().unwrap();

                players.retain(|&pid| pid != id);

                if players.is_empty() {
                    drop(players);
                    guard.remove(lobby_index);
                    drop(guard);

                    broadcast_active_lobbies(connected_clients, lobbies);
                } else {
                    if lobby.host == id {
                        if let Some(&new_host) = players.first() {
                            lobby.host = new_host;
                        }
                    }
                    drop(players);
                    drop(guard);

                    broadcast_lobby_state(connected_clients, lobbies, lobby_index);
                    broadcast_active_lobbies(connected_clients, lobbies);
                }

                let _ = send_to_client(
                    id,
                    connected_clients,
                    &json!({
                        "type": "confirmation",
                        "message": format!("You have left the lobby '{}'", name)
                    }),
                );
            } else {
                let _ = send_to_client(
                    id,
                    connected_clients,
                    &json!({
                        "type": "error",
                        "message": format!("Lobby '{}' does not exist", name)
                    }),
                );
            }

            Ok(())
        }

        MessageType::ListLobbies => {
            broadcast_active_lobbies(connected_clients, lobbies);
            Ok(())
        }

        MessageType::StartLobby { name } => {
            let mut guard = lobbies.lock().unwrap();
            let lobby_index_opt = guard.iter().position(|l| l.name == name);

            if let Some(lobby_index) = lobby_index_opt {
                let lobby = &mut guard[lobby_index];

                if lobby.host != id {
                    let _ = send_to_client(
                        id,
                        connected_clients,
                        &json!({
                            "type": "error",
                            "message": "Only the host can start the lobby"
                        }),
                    );
                    return Ok(());
                }

                if lobby.started {
                    let _ = send_to_client(
                        id,
                        connected_clients,
                        &json!({
                            "type": "error",
                            "message": "Lobby has already started"
                        }),
                    );
                    return Ok(());
                }

                lobby.started = true;

                let players: Vec<u32> = lobby.players.lock().unwrap().clone();

                // Initialize all players to fixed grid spawn positions
                {
                    let start_positions = match lobby.map_choice {
                        MapChoice::Small => &START_POSITIONS,
                        MapChoice::Big => &START_POSITIONS_MAP2,
                    };
                    let mut states = lobby.states.lock().unwrap();
                    for (idx, player_id) in players.iter().enumerate() {
                        if let Some((spawn_x, spawn_y)) = start_positions.get(idx) {
                            states.insert(
                                *player_id,
                                PlayerState {
                                    x: *spawn_x,
                                    y: *spawn_y,
                                    velocity: bevy::math::Vec2::ZERO,
                                    angle: START_ORIENTATION,
                                    inputs: PlayerInput::default(),
                                    last_processed_sequence: 0,
                                    boost_remaining: 0.0,
                                    input_queue: Vec::new(),
                                },
                            );
                        }
                    }
                }

                let map_choice = lobby.map_choice;
                drop(guard);
                broadcast_game_start(connected_clients, &players, &name, map_choice);

                // Spawn commands for each player
                let sender = cmd_sender.lock().unwrap();
                let start_positions = match map_choice {
                    MapChoice::Small => &START_POSITIONS,
                    MapChoice::Big => &START_POSITIONS_MAP2,
                };
                for (idx, player_id) in players.iter().enumerate() {
                    if let Some((spawn_x, spawn_y)) = start_positions.get(idx) {
                        let _ = sender.send(ServerCommand::SpawnPlayer {
                            player_id: *player_id,
                            lobby_name: name.clone(),
                            x: *spawn_x,
                            y: *spawn_y,
                        });
                    }
                }

                // Spawn AI cars to fill empty slots (up to 4 total)
                let num_players = players.len();
                let num_ai = start_positions.len().saturating_sub(num_players);
                for i in 0..num_ai {
                    let ai_id = 1000 + i as u32;
                    let slot_index = num_players + i;
                    if let Some((spawn_x, spawn_y)) = start_positions.get(slot_index) {
                        let _ = sender.send(ServerCommand::SpawnAI {
                            ai_id,
                            lobby_name: name.clone(),
                            x: *spawn_x,
                            y: *spawn_y,
                            angle: START_ORIENTATION,
                        });
                    }
                }

                let _ = send_to_client(
                    id,
                    connected_clients,
                    &json!({
                        "type": "confirmation",
                        "message": format!("You have started the lobby '{}'", name)
                    }),
                );
            } else {
                let _ = send_to_client(
                    id,
                    connected_clients,
                    &json!({
                        "type": "error",
                        "message": format!("Lobby '{}' does not exist", name)
                    }),
                );
            }

            Ok(())
        }

        MessageType::PlayerInput {
            sequence,
            forward,
            backward,
            left,
            right,
            drift,
            easy_drift,
            boost,
        } => handle_player_input(
            id,
            sequence,
            forward,
            backward,
            left,
            right,
            drift,
            easy_drift,
            boost,
            connected_clients,
            lobbies,
        ),

        MessageType::PlayerInputBuffer { inputs } => {
            handle_player_input_buffer(id, inputs, connected_clients, lobbies)
        }

        MessageType::Ping => {
            // Send Pong response to client
            let _ = send_to_client(
                id,
                connected_clients,
                &json!({
                    "type": "pong",
                }),
            );
            Ok(())
        }
    }
}

/// Handle player input message (legacy single input)
fn handle_player_input(
    id: u32,
    sequence: u64,
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    drift: bool,
    easy_drift: bool,
    boost: bool,
    _connected_clients: &ConnectedClients,
    lobbies: &LobbyList,
) -> io::Result<()> {
    // Find the lobby that the player is in
    let lobby_index_opt: Option<usize> = {
        let guard = lobbies.lock().unwrap();
        guard
            .iter()
            .position(|lobby| lobby.players.lock().unwrap().contains(&id))
    };

    if let Some(lobby_index) = lobby_index_opt {
        let guard = lobbies.lock().unwrap();
        let lobby = &guard[lobby_index];
        let mut states = lobby.states.lock().unwrap();

        if let Some(player_state) = states.get_mut(&id) {
            // Ignore old/duplicate inputs
            if sequence <= player_state.last_processed_sequence {
                return Ok(());
            }

            // Store client's sequence number as last processed
            player_state.last_processed_sequence = sequence;
            player_state.inputs = PlayerInput {
                forward,
                backward,
                left,
                right,
                drift,
                easy_drift,
                boost,
            };
        } else {
            panic!("Player {} does not have a current state", id);
        }
    }

    Ok(())
}

/// Handle buffered player inputs
fn handle_player_input_buffer(
    id: u32,
    inputs: Vec<InputData>,
    _connected_clients: &ConnectedClients,
    lobbies: &LobbyList,
) -> io::Result<()> {
    // Find the lobby that the player is in
    let lobby_index_opt: Option<usize> = {
        let guard = lobbies.lock().unwrap();
        guard
            .iter()
            .position(|lobby| lobby.players.lock().unwrap().contains(&id))
    };

    if let Some(lobby_index) = lobby_index_opt {
        let guard = lobbies.lock().unwrap();
        let lobby = &guard[lobby_index];
        let mut states = lobby.states.lock().unwrap();

        if let Some(player_state) = states.get_mut(&id) {
            // Add all inputs to the queue, filtering out old/duplicate ones
            for input in inputs {
                // Only add inputs newer than what we've already processed
                if input.sequence > player_state.last_processed_sequence {
                    player_state.input_queue.push(input);
                }
            }

            // Sort by sequence to ensure correct order
            player_state.input_queue.sort_by_key(|input| input.sequence);
        } else {
            panic!("Player {} does not have a current state", id);
        }
    }

    Ok(())
}

/// Send a message to a specific client
pub fn send_to_client(
    id: u32,
    connected_clients: &ConnectedClients,
    val: &serde_json::Value,
) -> io::Result<()> {
    let payload = val.to_string() + "\n";
    if let Some(addr) = connected_clients.addrs.lock().unwrap().get(&id).copied() {
        connected_clients.socket.send_to(payload.as_bytes(), addr)?;
    }
    Ok(())
}
