use bevy::prelude::*;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::get_ip::get_local_ip;
use crate::networking::{CarPosition, NetworkMessage};
use serde_json;

// Plugin that starts a background TCP listener
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, server_listener);
    }
}

fn server_listener() {
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
                    thread::spawn(move || {
                        handle_client(s, id, clients_ref, positions_ref);
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