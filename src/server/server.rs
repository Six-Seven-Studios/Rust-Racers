use bevy::prelude::*;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use serde_json::json;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClientMsg {
    CreateLobby { name: String },

    JoinLobby { name: String },

    LeaveLobby { name: String },

    ListLobbies,

    StartLobby { name: String },
}

// Track connected clients
pub struct ConnectedClients {
    pub ids: Arc<Mutex<Vec<u32>>>,
    pub streams: Arc<Mutex<HashMap<u32, TcpStream>>>,
}

pub struct Lobby {
    pub players: Arc<Mutex<Vec<u32>>>,
    pub host: u32,
    pub name: String,
    pub started: bool,
}

type LobbyList = Arc<Mutex<Vec<Lobby>>>;

impl Default for ConnectedClients {
    fn default() -> Self {
        Self {
            ids: Arc::new(Mutex::new(Vec::new())),
            streams: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for Lobby {
    fn default() -> Self {
        Self {
            players: Arc::new(Mutex::new(Vec::new())),
            host: 0,
            name: String::from(""),
            started: false,
        }
    }
}

fn server_listener(
    connected_clients: ConnectedClients,
    lobbies: LobbyList,
) {
    thread::spawn(move || {
        let listener = TcpListener::bind(("0.0.0.0", 4000)).expect("Expected to bind to port 4000 successfully");
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

                    // Add to streams map
                    if let Ok(mut client_streams) = connected_clients.streams.lock() {
                        client_streams.insert(id, stream_clone);
                    }

                    // Add to connected clients list
                    if let Ok(mut client_ids) = connected_clients.ids.lock() {
                        client_ids.push(id);
                        println!("Connected clients: {:?}", client_ids);
                    }

                    let connected_clients_clone = ConnectedClients {
                        ids: Arc::clone(&connected_clients.ids),
                        streams: Arc::clone(&connected_clients.streams),
                    };
                    let lobbies_clone = Arc::clone(&lobbies);
                    thread::spawn(move || handle_client(id, s, connected_clients_clone, lobbies_clone));
                }
                Err(e) => eprintln!("Accept error: {e}"),
            }
        }
    });
}

fn handle_client(
    id: u32,
    stream: TcpStream,
    connected_clients: ConnectedClients,
    lobbies: LobbyList,
) {
    let mut reader = BufReader::new(stream);

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // EOF: client disconnected
                disconnect_cleanup(id, &connected_clients, &lobbies);
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() { continue; }

                match serde_json::from_str::<ClientMsg>(trimmed) {
                    Ok(msg) => {
                        if let Err(e) = handle_client_msg(id, msg, &connected_clients, &lobbies) {
                            eprintln!("handle_client_msg error for {id}: {e}");
                        }
                    }
                    Err(e) => {
                        eprintln!("JSON parse error from {id}: {e}; raw={trimmed}");
                        // Optional: reply with an error
                        let _ = send_to_client(id, &connected_clients, &json!({
                            "type":"error", "message":"invalid_json"
                        }));
                    }
                }
            }
            Err(e) => {
                eprintln!("read_line error for {id}: {e}");
                disconnect_cleanup(id, &connected_clients, &lobbies);
                break;
            }
        }
    }
}

fn handle_client_msg(
    id: u32,
    msg: ClientMsg,
    connected_clients: &ConnectedClients,
    lobbies: &LobbyList,
) -> std::io::Result<()> {
    match msg {
        ClientMsg::CreateLobby { name } => {
            let mut guard = lobbies.lock().unwrap();

            // Create new lobby
            let mut lobby = Lobby::default();

            lobby.host = id;
            lobby.name = name.clone();

            guard.push(lobby);

            broadcast_active_lobbies(connected_clients, lobbies);

            Ok(())
        }

        ClientMsg::JoinLobby { name } => {
            let mut guard = lobbies.lock().unwrap();

            // Try to find a lobby that has the same name
            let mut found_lobby_index: Option<usize> = None;

            for (i, lobby) in guard.iter().enumerate() {
                if lobby.name == name {
                    found_lobby_index = Some(i);
                    break;
                }
            }

            if let Some(i) = found_lobby_index {
                // Get the lobby
                let lobby = &mut guard[i];

                // Add player if not already there
                let mut players = lobby.players.lock().unwrap();
                if !players.contains(&id) {
                    players.push(id);
                }
            }

            broadcast_lobby_state(connected_clients, lobbies);

            Ok(())
        }

        ClientMsg::LeaveLobby { name } => {
            let mut guard = lobbies.lock().unwrap();

            // Try to find a lobby that has the same name
            let mut found_lobby_index: Option<usize> = None;

            for (i, lobby) in guard.iter().enumerate() {
                if lobby.name == name {
                    found_lobby_index = Some(i);
                    break;
                }
            }

            if let Some(i) = found_lobby_index {
                // Get the lobby
                let lobby = &mut guard[i];

                let mut players = lobby.players.lock().unwrap();
                players.retain(|player_id| {
                    return *player_id != id;
                });
            }
            
            broadcast_lobby_state(connected_clients, lobbies);

            Ok(())
        }

        ClientMsg::ListLobbies => {
            let snap: Vec<_> = {
                let guard = lobbies.lock().unwrap();
                guard.iter().map(|l| {
                    let count = l.players.lock().unwrap().len();
                    json!({"name": l.name, "players": count})
                }).collect()
            };
            
            send_to_client(id, connected_clients, &json!({
                "type":"active_lobbies",
                "lobbies": snap
            }))
        }

        ClientMsg::StartLobby { name } => {
            let mut guard = lobbies.lock().unwrap();

            // Try to find a lobby that has the same name
            let mut found_lobby_index: Option<usize> = None;

            for (i, lobby) in guard.iter().enumerate() {
                if lobby.name == name {
                    found_lobby_index = Some(i);
                    break;
                }
            }

            if let Some(i) = found_lobby_index {
                // Get the lobby
                let lobby = &mut guard[i];

                // Mark the lobby as started
                lobby.started = true;
            }

            Ok(())
        }
    }
}

fn send_to_client(id: u32, connected_clients: &ConnectedClients, val: &serde_json::Value) -> std::io::Result<()> {
    let payload = val.to_string() + "\n";
    if let Some(mut s) = connected_clients.streams.lock().unwrap().get(&id).and_then(|s| s.try_clone().ok()) {
        s.write_all(payload.as_bytes())?;
        s.flush()?;
    }
    Ok(())
}

fn broadcast_lobby_state(
    connected_clients: &ConnectedClients,
    lobbies: &LobbyList,
) {
    let guard = lobbies.lock().unwrap();

    for lobby in guard.iter() {
        // Snapshot the player IDs
        let players: Vec<u32> = {
            let lobby_guard = lobby.players.lock().unwrap();
            lobby_guard.clone()
        };

        // Continue if players are empty
        if players.is_empty() {
            continue;
        }

        // Build one payload that everyone in this lobby gets
        let payload = json!({ "players": players }).to_string() + "\n";

        // Clone target streams
        let mut targets = Vec::new();
        {
            let streams = connected_clients.streams.lock().unwrap();
            for pid in &players {
                if let Some(s) = streams.get(pid) {
                    if let Ok(clone) = s.try_clone() {
                        targets.push(clone);
                    }
                }
            }
        }

        // Write to everyone
        for mut stream in targets {
            let _ = stream.write_all(payload.as_bytes());
            let _ = stream.flush();
        }
    }
}

fn broadcast_active_lobbies(
    connected_clients: &ConnectedClients,
    lobbies: &LobbyList,
) {
    // Snapshot the lobby list (name + count)
    let lobby_list: Vec<_> = {
        let guard = lobbies.lock().unwrap();
        guard.iter().map(|l| {
            let count = l.players.lock().unwrap().len();
            json!({"name": l.name, "players": count})
        }).collect()
    };

    // Build a single payload to send to everyone.
    let payload = json!({
        "type": "active_lobbies",
        "lobbies": lobby_list
    }).to_string();
    let payload = payload + "\n"; // helps line-oriented clients

    // Clone all streams
    let targets: Vec<TcpStream> = {
        let streams = connected_clients.streams.lock().unwrap();
        streams.values().filter_map(|s| s.try_clone().ok()).collect()
    };

    // Write to everyone
    for mut stream in targets {
        let _ = stream.write_all(payload.as_bytes());
        let _ = stream.flush();
    }
}

fn disconnect_cleanup(id: u32, connected: &ConnectedClients, lobbies: &LobbyList) {
    // remove from streams/ids
    if let Ok(mut m) = connected.streams.lock() { m.remove(&id); }
    if let Ok(mut ids) = connected.ids.lock() { ids.retain(|x| *x != id); }

    // remove from all lobbies
    let mut guard = lobbies.lock().unwrap();
    for lobby in guard.iter_mut() {
        let mut players = lobby.players.lock().unwrap();
        players.retain(|p| *p != id);
    }

    println!("Client {id} disconnected and cleaned up");
}

fn main() {
    let connected_clients = ConnectedClients::default();
    let lobbies: LobbyList = Arc::new(Mutex::new(Vec::new()));
    server_listener(connected_clients, lobbies);

    loop {
        thread::sleep(Duration::from_millis(250));
    }
}