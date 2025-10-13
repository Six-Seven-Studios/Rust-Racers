use bevy::prelude::*;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::get_ip::get_local_ip;
use crate::networking::{CarPosition, NetworkMessage};
use serde_json;

// Resource to track connected clients
#[derive(Resource)]
pub struct ConnectedClients {
    pub client_ids: Arc<Mutex<Vec<u32>>>,
}

impl Default for ConnectedClients {
    fn default() -> Self {
        Self {
            client_ids: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

// Plugin that starts a background TCP listener
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ConnectedClients>()
            .add_systems(Startup, server_listener);
    }
}

fn server_listener(connected_clients: Res<ConnectedClients>) {
    let connected_clients_clone = Arc::clone(&connected_clients.client_ids);
    thread::spawn(move || {
        let listener = TcpListener::bind(("0.0.0.0", 4000)).expect("Expected to bind to port 4000 successfully");

        if let Ok(ip) = get_local_ip() {
            println!("Listening on {}:4000", ip);
        } else {
            println!("Listening on 0.0.0.0:4000");
        }

        let clients: Arc<Mutex<HashMap<u32, TcpStream>>> = Arc::new(Mutex::new(HashMap::new()));
        let car_positions: Arc<Mutex<HashMap<u32, CarPosition>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut next_id: u32 = 1;

        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    let id = next_id;
                    next_id += 1;

                    let line = format!("WELCOME PLAYER {}\n", id);
                    let _ = s.write_all(line.as_bytes());
                    let _ = s.flush();
                    println!("Greeted client with id={id}");

                    let stream_clone = s.try_clone().expect("Failed to clone stream");

                    if let Ok(mut clients_guard) = clients.lock() {
                        clients_guard.insert(id, stream_clone);
                    }

                    // Add to connected clients list
                    if let Ok(mut client_ids) = connected_clients_clone.lock() {
                        client_ids.push(id);
                        println!("Connected clients: {:?}", client_ids);
                    }

                    // Broadcast updated lobby state to all clients
                    broadcast_lobby_state(&clients, &connected_clients_clone);

                    // send existing player positions to new client
                    if let Ok(positions_guard) = car_positions.lock() {
                        if !positions_guard.is_empty() {
                            let all_positions: Vec<CarPosition> = positions_guard.values().cloned().collect();
                            let message = NetworkMessage::AllPositions(all_positions);
                            if let Ok(serialized) = serde_json::to_string(&message) {
                                let _ = writeln!(s, "{}", serialized);
                                let _ = s.flush();
                            }
                        }
                    }

                    let clients_ref = Arc::clone(&clients);
                    let positions_ref = Arc::clone(&car_positions);
                    let connected_clients_ref = Arc::clone(&connected_clients_clone);
                    thread::spawn(move || {
                        handle_client(s, id, clients_ref, positions_ref, connected_clients_ref);
                    });
                }
                Err(e) => eprintln!("Accept error: {e}"),
            }
        }
    });
}

fn handle_client(
    stream: TcpStream,
    client_id: u32,
    clients: Arc<Mutex<HashMap<u32, TcpStream>>>,
    positions: Arc<Mutex<HashMap<u32, CarPosition>>>,
    connected_clients: Arc<Mutex<Vec<u32>>>,
) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                println!("Client {} disconnected", client_id);
                if let Ok(mut clients_guard) = clients.lock() {
                    clients_guard.remove(&client_id);
                }
                if let Ok(mut positions_guard) = positions.lock() {
                    positions_guard.remove(&client_id);
                }
                // Remove from connected clients list
                if let Ok(mut client_ids) = connected_clients.lock() {
                    client_ids.retain(|&id| id != client_id);
                    println!("Connected clients: {:?}", client_ids);
                }

                // Broadcast updated lobby state to remaining clients
                broadcast_lobby_state(&clients, &connected_clients);

                break;
            }
            Ok(_) => {
                if let Ok(message) = serde_json::from_str::<NetworkMessage>(line.trim()) {
                    match message {
                        NetworkMessage::Position(car_pos) => {
                            if let Ok(mut positions_guard) = positions.lock() {
                                positions_guard.insert(client_id, car_pos);
                            }

                            broadcast_positions(&clients, &positions);
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from client {}: {}", client_id, e);
                break;
            }
        }
    }
}

fn broadcast_positions(
    clients: &Arc<Mutex<HashMap<u32, TcpStream>>>,
    positions: &Arc<Mutex<HashMap<u32, CarPosition>>>,
) {
    if let (Ok(positions_guard), Ok(mut clients_guard)) = (positions.lock(), clients.lock()) {
        let all_positions: Vec<CarPosition> = positions_guard.values().cloned().collect();
        let message = NetworkMessage::AllPositions(all_positions);

        if let Ok(serialized) = serde_json::to_string(&message) {
            let mut disconnected_clients = Vec::new();

            for (client_id, stream) in clients_guard.iter_mut() {
                if let Err(_) = writeln!(stream, "{}", serialized) {
                    disconnected_clients.push(*client_id);
                }
            }

            for client_id in disconnected_clients {
                clients_guard.remove(&client_id);
            }
        }
    }
}

fn broadcast_lobby_state(
    clients: &Arc<Mutex<HashMap<u32, TcpStream>>>,
    connected_clients: &Arc<Mutex<Vec<u32>>>,
) {
    if let (Ok(client_ids), Ok(mut clients_guard)) = (connected_clients.lock(), clients.lock()) {
        // Host is player 0, others are 1, 2, 3...
        let mut player_ids = vec![0]; // Host is always player 0
        player_ids.extend(client_ids.iter().copied());

        let lobby_state = crate::networking::LobbyState { player_ids: player_ids.clone() };
        let message = NetworkMessage::LobbySync(lobby_state);

        println!("Broadcasting lobby state: {:?}", player_ids);

        if let Ok(serialized) = serde_json::to_string(&message) {
            let mut disconnected_clients = Vec::new();

            for (client_id, stream) in clients_guard.iter_mut() {
                if let Err(_) = writeln!(stream, "{}", serialized) {
                    disconnected_clients.push(*client_id);
                }
            }

            for client_id in disconnected_clients {
                clients_guard.remove(&client_id);
            }
        }
    }
}