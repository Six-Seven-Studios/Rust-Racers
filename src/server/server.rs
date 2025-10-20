use bevy::prelude::*;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write, Result};
use std::net::{TcpListener, TcpStream, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;
use serde_json;

// Track connected clients
#[derive(Resource, Clone)]
pub struct ConnectedClients {
    pub ids: Arc<Mutex<Vec<u32>>>,
    pub streams: Arc<Mutex<HashMap<u32, TcpStream>>>,
    pub ips: Arc<Mutex<HashMap<SocketAddr, u32>>>,
}

impl Default for ConnectedClients {
    fn default() -> Self {
        Self {
            ids: Arc::new(Mutex::new(Vec::new())),
            streams: Arc::new(Mutex::new(HashMap::new())),
            ips: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

fn server_listener(connected_clients: ConnectedClients) {
    thread::spawn(move || {
        let listener = TcpListener::bind(("0.0.0.0", 4000)).expect("Expected to bind to port 4000 successfully");
        let mut next_id: u32 = 1;

        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    let id = next_id;
                    next_id += 1;

                    let player_ip = s.peer_addr().unwrap();

                    let line = format!("WELCOME PLAYER {}\n", id);
                    let _ = s.write_all(line.as_bytes());
                    let _ = s.flush();
                    println!("Greeted client with id={id}");

                    let stream_clone = s.try_clone().expect("Failed to clone stream");

                    // Add to streams map
                    if let Ok(mut client_streams) = connected_clients.streams.lock() {
                        client_streams.insert(id, stream_clone);
                    }

                    // Add to connected clients list
                    if let Ok(mut client_ids) = connected_clients.ids.lock() {
                        client_ids.push(id);
                        println!("Connected clients: {:?}", client_ids);
                    }

                    if let Ok(mut client_ips) = connected_clients.ips.lock() {
                        client_ips.insert(player_ip, id);
                    }
                }
                Err(e) => eprintln!("Accept error: {e}"),
            }
        }
    });
}

fn main() {
    let connected_clients = ConnectedClients::default();
    server_listener(connected_clients);

    loop {
        
    }
}