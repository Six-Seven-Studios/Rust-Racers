use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use bevy::tasks::IoTaskPool;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use serde_json::json;
use serde::Deserialize;
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum MessageType {
    CreateLobby { name: String },

    JoinLobby { name: String },

    LeaveLobby { name: String },

    ListLobbies,

    StartLobby { name: String },

    PlayerInput {
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        drift: bool,
    },

    Ping
}

// Track connected clients
#[derive(Clone, Resource)]
pub struct ConnectedClients {
    pub ids: Arc<Mutex<Vec<u32>>>,
    pub addrs: Arc<Mutex<HashMap<u32, SocketAddr>>>,
    pub addr_to_id: Arc<Mutex<HashMap<SocketAddr, u32>>>,
    pub last_seen: Arc<Mutex<HashMap<u32, Instant>>>,
    pub socket: Arc<UdpSocket>,
}

#[derive(Clone, Debug)]
pub struct PlayerInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub drift: bool,
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub angle: f32,
    pub inputs: PlayerInput,
    pub input_count: u64,
}

#[derive(Clone)]
pub struct Lobby {
    pub players: Arc<Mutex<Vec<u32>>>,
    pub host: u32,
    pub name: String,
    pub started: bool,
    pub states: Arc<Mutex<HashMap<u32, PlayerState>>>,
}

type LobbyList = Arc<Mutex<Vec<Lobby>>>;

/// Resource wrapper for LobbyList
#[derive(Resource, Clone)]
pub struct Lobbies {
    pub list: LobbyList,
}

impl ConnectedClients {
    fn new(socket: Arc<UdpSocket>) -> Self {
        Self {
            ids: Arc::new(Mutex::new(Vec::new())),
            addrs: Arc::new(Mutex::new(HashMap::new())),
            addr_to_id: Arc::new(Mutex::new(HashMap::new())),
            last_seen: Arc::new(Mutex::new(HashMap::new())),
            socket,
        }
    }
}

impl Default for PlayerInput { 
    fn default() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,
            drift: false,
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
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// Marks an entity as a player with a specific ID
#[derive(Component)]
pub struct PlayerId(pub u32);

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Velocity {
    pub vx: f32,
    pub vy: f32,
}

#[derive(Component)]
pub struct Orientation {
    pub angle: f32,
}

// Component for player input state
#[derive(Component)]
pub struct PlayerInputComponent {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub drift: bool,
    pub input_count: u64,
}

impl Default for PlayerInputComponent {
    fn default() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,
            drift: false,
            input_count: 0,
        }
    }
}

// Marks which lobby a player entity belongs to
#[derive(Component)]
pub struct LobbyMember {
    pub lobby_name: String,
}

// Map player IDs to their entity in the ECS
#[derive(Resource, Default)]
pub struct PlayerEntities {
    pub map: HashMap<u32, Entity>,
}

// Commands from networking threads to Bevy systems
#[derive(Debug, Clone)]
pub enum ServerCommand {
    SpawnPlayer {
        player_id: u32,
        lobby_name: String,
        x: f32,
        y: f32,
    },
    DespawnPlayer {
        player_id: u32,
    },
}

// Resource for receiving commands in Bevy systems
#[derive(Resource)]
pub struct ServerCommandReceiver {
    pub receiver: Arc<Mutex<std::sync::mpsc::Receiver<ServerCommand>>>,
}

fn server_listener(
    connected_clients: ConnectedClients,
    lobbies: LobbyList,
    cmd_sender: Arc<Mutex<std::sync::mpsc::Sender<ServerCommand>>>,
) {
    let task_pool = IoTaskPool::get();
    task_pool.spawn(async move {
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
                                let _ = connected_clients.socket.send_to(welcome.as_bytes(), addr);

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
                                    &cmd_sender
                                ) {
                                    eprintln!("handle_client_message error for {}: {}", client_id, e);
                                }
                            }
                            Err(e) => {
                                eprintln!("JSON parse error from {}: {}; raw={}", client_id, e, trimmed);
                                let _ = send_to_client(client_id, &connected_clients, &json!({
                                    "type": "error",
                                    "message": "invalid_json"
                                }));
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("recv_from error: {}", e);
                }
            }
        }
    }).detach();
}

fn handle_client_message(
    id: u32,
    message: MessageType,
    connected_clients: &ConnectedClients,
    lobbies: &LobbyList,
    cmd_sender: &Arc<Mutex<std::sync::mpsc::Sender<ServerCommand>>>,
) -> std::io::Result<()> {
    match message {
        MessageType::CreateLobby { name } => {
            {
                let mut guard = lobbies.lock().unwrap();

                 // Try to find a lobby that has the same name
                 let mut found_lobby_index: Option<usize> = None;

                 for (i, lobby) in guard.iter().enumerate() {
                     if lobby.name == name {
                         found_lobby_index = Some(i);
                         break;
                     }
                 }
 
                // Cannot have two lobbies with the same name
                if let Some(_i) = found_lobby_index {
                    return send_to_client(id, connected_clients, &json!({
                        "type": "error",
                        "message": format!("A lobby named '{}' already exists", name)
                    }));
                } 
                else {
                    // Create new lobby
                    let mut lobby = Lobby::default();

                    lobby.host = id;
                    lobby.name = name.clone();

                    {
                        let mut players = lobby.players.lock().unwrap();
                        players.push(id);
                    }

                    guard.push(lobby);

                    let _ = send_to_client(id, connected_clients, &json!({
                        "type": "confirmation",
                        "message": format!("You have created the lobby '{}'", name)
                    }));
                }
            }

            broadcast_active_lobbies(connected_clients, lobbies);

            Ok(())
        }

        MessageType::JoinLobby { name } => {
            let lobby_index:usize;
            {
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

                    lobby_index = i;

                    // Cannot join a lobby that already started
                    if lobby.started {
                        return send_to_client(id, connected_clients, &json!({
                            "type": "error",
                            "message": format!("Cannot join '{}' because it has already started", name)
                        }));
                    }
                    // Cannot join a lobby that already has 4 players
                    else if lobby.players.lock().unwrap().len() == 4 {
                        return send_to_client(id, connected_clients, &json!({
                            "type": "error",
                            "message": format!("Cannot join '{}' because it is full", name)
                        }));
                    }
                    else {
                        // Add player if not already there
                        let mut players = lobby.players.lock().unwrap();
                        if !players.contains(&id) {
                            players.push(id);
                        }

                        let _ = send_to_client(id, connected_clients, &json!({
                            "type": "confirmation",
                            "message": format!("You have joined the lobby '{}'", name)
                        }));
                    }
                }
                else {
                    // Send an error if lobby does not exist
                    return send_to_client(id, connected_clients, &json!({
                        "type": "error",
                        "message": format!("Cannot join '{}' because it does not exist", name)
                    }));
                }
            }

            broadcast_lobby_state(connected_clients, lobbies, lobby_index);
            broadcast_active_lobbies(connected_clients, lobbies);

            Ok(())
        }

        MessageType::LeaveLobby { name } => {
            let mut lobby_exists = true;
            let mut lobby_index = 0;
            {
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

                    lobby_index = i;

                    // If the lobby already started send an error
                    if lobby.started {
                        return send_to_client(id, connected_clients, &json!({
                            "type": "error",
                            "message": format!("Cannot leave '{}' because it has already started", name)
                        }));
                    }
                    else {
                        // Remove the player
                        let mut players = lobby.players.lock().unwrap();
                        players.retain(|player_id| {
                            return *player_id != id;
                        });

                        let _ = send_to_client(id, connected_clients, &json!({
                            "type": "confirmation",
                            "message": format!("You have left the lobby '{}'", name)
                        }));

                        // Remove lobby if there are no players
                        if players.len() == 0 {
                            lobby_exists = false;
                        }
                        // If the host leaves, set the new host to be the next player 
                        else if lobby.host == id {
                            if let Some(first_player) = players.get(0) {
                                lobby.host = *first_player;
                            }
                            else {
                                println!("Should never get here because players should have at least one element");
                            }
                        }
                    }
                }
                else {
                    // If the lobby cannot be found send an error
                    let _ = send_to_client(id, connected_clients, &json!({
                        "type": "error",
                        "message": format!("Cannot join '{}' because it does not exist", name)
                    }));
                }

                // Remove the lobby if necessary
                if !lobby_exists {
                    guard.remove(lobby_index);
                }
            }
            
            // Broadcast state if the lobby still exists
            if lobby_exists {
                broadcast_lobby_state(connected_clients, lobbies, lobby_index);
            }

            broadcast_active_lobbies(connected_clients, lobbies);

            Ok(())
        }

        MessageType::ListLobbies => {
            // Only send lobbies that are not started
            let lobby_list: Vec<_> = {
                let guard = lobbies.lock().unwrap();
                guard.iter().filter(|l| !l.started).map(|l| {
                    let count = l.players.lock().unwrap().len();
                    json!({
                        "name": l.name,
                        "players": count
                    })
                }).collect()
            };
            
            send_to_client(id, connected_clients, &json!({
                "type": "active_lobbies",
                "lobbies": lobby_list
            }))
        }

        MessageType::StartLobby { name } => {
            {
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

                    // The host is the only one who can start the game
                    if lobby.host != id {
                        return send_to_client(id, connected_clients, &json!({
                            "type": "error",
                            "message": format!("You are not the host of '{}', so you cannot start the game.", name)
                        }))
                    }
                    else {
                        // Mark the lobby as started
                        lobby.started = true;

                        // Get the players list to broadcast to
                        let players: Vec<u32> = lobby.players.lock().unwrap().clone();
                        
                        // Initialize all players to varied spawn positions
                        {
                             let mut states = lobby.states.lock().unwrap();
                             for player_id in &players {
                                 states.insert(*player_id, PlayerState {
                                     x: 2752.0 + (*player_id as f32) * 100.0,
                                     y: 960.0,
                                    vx: 0.0,
                                    vy: 0.0,
                                    angle: 0.0,
                                    inputs: PlayerInput::default(),
                                    input_count: 0,
                                });
                            }
                        }

                        // Broadcast game started to all players in the lobby
                        drop(guard); // Release the lock before broadcasting
                        broadcast_game_start(connected_clients, &players, &name);

                        // Spawn commands for each player
                        let sender = cmd_sender.lock().unwrap();
                        for player_id in &players {
                            let spawn_x = 2752.0 + (*player_id as f32) * 100.0;
                            let spawn_y = 960.0;

                            let _ = sender.send(ServerCommand::SpawnPlayer {
                                player_id: *player_id,
                                lobby_name: name.clone(),
                                x: spawn_x,
                                y: spawn_y,
                            });
                        }


                        let _ = send_to_client(id, connected_clients, &json!({
                            "type": "confirmation",
                            "message": format!("You have started the lobby '{}'", name)
                        }));

                        broadcast_active_lobbies(connected_clients, lobbies);
                        return Ok(());
                    }
                }
            }

            broadcast_active_lobbies(connected_clients, lobbies);

            Ok(())
        }

        // Need to check if the lobby is active or not
        MessageType::PlayerInput { forward, backward, left, right, drift } => {
            handle_player_input(id, forward, backward, left, right, drift, connected_clients, lobbies)
        }

        // This just maintains the client connection
        MessageType::Ping => {
            Ok(())
        }
    }
}

fn send_to_client(id: u32, connected_clients: &ConnectedClients, val: &serde_json::Value) -> std::io::Result<()> {
    let payload = val.to_string() + "\n";
    if let Some(addr) = connected_clients.addrs.lock().unwrap().get(&id).copied() {
        connected_clients.socket.send_to(payload.as_bytes(), addr)?;
    }
    Ok(())
}

fn broadcast_lobby_state(
    connected_clients: &ConnectedClients,
    lobbies: &LobbyList,
    lobby_index: usize,
) {
    let guard = lobbies.lock().unwrap();

    let lobby = if let Some(i) = guard.get(lobby_index) {
        i
    } else {
        println!("Lobby does not exist");
        return;
    };

    // Snapshot the player IDs
    let players: Vec<u32> = {
        let lobby_guard = lobby.players.lock().unwrap();
        lobby_guard.clone()
    };

    // Build one payload that everyone in this lobby gets
    let payload = json!({
        "lobby": lobby.name.clone(),
        "players": players
    }).to_string() + "\n";

    // Get target addresses and send
    let addrs = connected_clients.addrs.lock().unwrap();
    for pid in &players {
        if let Some(addr) = addrs.get(pid) {
            let _ = connected_clients.socket.send_to(payload.as_bytes(), addr);
        }
    }
}

fn broadcast_active_lobbies(
    connected_clients: &ConnectedClients,
    lobbies: &LobbyList,
) {
    // Only send lobbies that are not started
    let lobby_list: Vec<_> = {
        let guard = lobbies.lock().unwrap();
        guard.iter().filter(|l| !l.started).map(|l| {
            let count = l.players.lock().unwrap().len();
            json!({
                "name": l.name,
                "players": count
            })
        }).collect()
    };

    // Build a single payload to send to everyone.
    let payload = json!({
        "type": "active_lobbies",
        "lobbies": lobby_list
    }).to_string() + "\n";

    // Send to all connected clients
    let addrs = connected_clients.addrs.lock().unwrap();
    for addr in addrs.values() {
        let _ = connected_clients.socket.send_to(payload.as_bytes(), addr);
    }
}

fn handle_player_input(
    id: u32,
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    drift: bool,
    _connected_clients: &ConnectedClients,
    lobbies: &LobbyList,
) -> std::io::Result<()> {
    // Find the lobby that the player is in
    let lobby_index_opt: Option<usize> = {
        let guard = lobbies.lock().unwrap();
        guard.iter().position(|lobby| {
            lobby.players.lock().unwrap().contains(&id)
        })
    };

    if let Some(lobby_index) = lobby_index_opt {
        // Update this player's inputs in the lobby
        let guard = lobbies.lock().unwrap();
        let lobby = &guard[lobby_index];
        let mut states = lobby.states.lock().unwrap();

        // Update the player's state
        if let Some(player_state) = states.get_mut(&id) {
            // TODO: simulate the input
            player_state.input_count += 1;
            player_state.inputs = PlayerInput {
                forward,
                backward,
                left,
                right,
                drift,
            };
        } else {
            // This shouldn't happen because we should initialize state at race start.
            panic!("Player {} does not have a current state", id);
        }
    }

    Ok(())
}

// Broadcast game start to all players in a lobby
fn broadcast_game_start(
    connected_clients: &ConnectedClients,
    players: &[u32],
    lobby_name: &str,
) {
    // Build the game started payload
    let payload = json!({
        "type": "game_started",
        "lobby": lobby_name
    }).to_string() + "\n";

    // Send to all players in the lobby
    let addrs = connected_clients.addrs.lock().unwrap();
    for pid in players {
        if let Some(addr) = addrs.get(pid) {
            let _ = connected_clients.socket.send_to(payload.as_bytes(), addr);
        }
    }
}

fn disconnect_cleanup(id: u32, connected: &ConnectedClients, lobbies: &LobbyList) {
    // Get the address before removing
    let addr = connected.addrs.lock().unwrap().get(&id).copied();

    // Remove from all maps
    if let Ok(mut m) = connected.addrs.lock() { m.remove(&id); }
    if let Ok(mut ids) = connected.ids.lock() { ids.retain(|x| *x != id); }
    if let Ok(mut last_seen) = connected.last_seen.lock() { last_seen.remove(&id); }
    if let Some(addr) = addr {
        if let Ok(mut addr_to_id) = connected.addr_to_id.lock() {
            addr_to_id.remove(&addr);
        }
    }

    let mut empty_lobbies: Vec<usize> = Vec::new();

    // Remove from all lobbies
    let mut guard = lobbies.lock().unwrap();
    for (i, lobby) in guard.iter_mut().enumerate() {
        let mut players = lobby.players.lock().unwrap();
        players.retain(|p| *p != id);

        // Mark the empty lobbies as to be removed
        if players.len() == 0 {
            empty_lobbies.push(i);
        } else if lobby.host == id {
            if let Some(first_player) = players.get(0) {
                lobby.host = *first_player;
            } else {
                println!("Should never get here because players should have at least one element");
            }
        }
    }

    // Remove the empty lobbies
    for index in empty_lobbies.iter().rev() {
        guard.remove(*index);
    }

    println!("Client {} disconnected and cleaned up", id);
}

fn get_local_ip() -> Result<String, Box<dyn std::error::Error>> {
    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip().to_string())
}

// System to apply physics simulation to all active players
fn physics_simulation_system(
    mut query: Query<(
        &PlayerId,
        &mut Position,
        &mut Velocity,
        &mut Orientation,
        &PlayerInputComponent,
        &LobbyMember,
    )>,
    time: Res<Time>,
    lobbies: Res<Lobbies>,
) {
    const ACCEL_RATE: f32 = 700.0;
    const FRICTION: f32 = 0.95;
    const TURNING_RATE: f32 = 3.0;
    const MAX_SPEED: f32 = 350.0;

    let delta = time.delta_secs();
    let accel = ACCEL_RATE * delta;

    // Check which lobbies have started
    let started_lobbies: Vec<String> = {
        let guard = lobbies.list.lock().unwrap();
        guard.iter()
            .filter(|l| l.started)
            .map(|l| l.name.clone())
            .collect()
    };

    for (_player_id, mut pos, mut vel, mut orient, input, lobby_member) in query.iter_mut() {
        // Only simulate physics for players in started lobbies
        if !started_lobbies.contains(&lobby_member.lobby_name) {
            continue;
        }

        // Apply turning
        if input.left {
            orient.angle += TURNING_RATE * delta;
        }
        if input.right {
            orient.angle -= TURNING_RATE * delta;
        }

        // Calculate forward vector
        let forward_x = orient.angle.cos();
        let forward_y = orient.angle.sin();

        // Apply acceleration
        if input.forward {
            vel.vx += forward_x * accel;
            vel.vy += forward_y * accel;

            // Clamp to max speed
            let speed = (vel.vx * vel.vx + vel.vy * vel.vy).sqrt();
            if speed > MAX_SPEED {
                vel.vx = (vel.vx / speed) * MAX_SPEED;
                vel.vy = (vel.vy / speed) * MAX_SPEED;
            }
        }

        // Apply backward acceleration (slower)
        if input.backward {
            vel.vx -= forward_x * (accel / 2.0);
            vel.vy -= forward_y * (accel / 2.0);

            let speed = (vel.vx * vel.vx + vel.vy * vel.vy).sqrt();
            if speed > MAX_SPEED / 2.0 {
                vel.vx = (vel.vx / speed) * (MAX_SPEED / 2.0);
                vel.vy = (vel.vy / speed) * (MAX_SPEED / 2.0);
            }
        }

        // Apply friction when not accelerating
        if !input.forward && !input.backward {
            vel.vx *= FRICTION;
            vel.vy *= FRICTION;

            // Stop if moving very slowly
            if vel.vx.abs() < 0.1 {
                vel.vx = 0.0;
            }
            if vel.vy.abs() < 0.1 {
                vel.vy = 0.0;
            }
        }

        // Update position
        pos.x += vel.vx * delta;
        pos.y += vel.vy * delta;
    }
}

// System to broadcast game state to all clients
fn broadcast_state_system(
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
                    "vx": vel.vx,
                    "vy": vel.vy,
                    "angle": orient.angle,
                    "input_count": input.input_count
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

// System to update player input components from the lobby states
fn sync_input_from_lobbies_system(
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
                input_component.input_count = state.input_count;
            }
        }
    }
}

// System to process commands from networking threads (spawn/despawn players)
fn process_server_commands_system(
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
                    Velocity { vx: 0.0, vy: 0.0 },
                    Orientation { angle: 0.0 },
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

// System to check for timed out clients and disconnect them
fn timeout_cleanup_system(
    connected_clients: Res<ConnectedClients>,
    lobbies: Res<Lobbies>,
) {
    const TIMEOUT_SECONDS: u64 = 10;

    let now = Instant::now();
    let mut timed_out_clients = Vec::new();

    // Find clients that haven't sent anything in TIMEOUT_SECONDS
    {
        let last_seen = connected_clients.last_seen.lock().unwrap();
        for (id, last_time) in last_seen.iter() {
            if now.duration_since(*last_time).as_secs() > TIMEOUT_SECONDS {
                timed_out_clients.push(*id);
            }
        }
    }

    // Disconnect timed out clients
    for id in timed_out_clients {
        println!("Client {} timed out", id);
        disconnect_cleanup(id, &connected_clients, &lobbies.list);
    }
}

fn main() {
    // Display the local IP address
    match get_local_ip() {
        Ok(ip) => println!("Server running on {}:4000", ip),
        Err(e) => println!("Server running on 0.0.0.0:4000 (Could not determine local IP: {})", e),
    }

    // Bind UDP socket
    let socket = UdpSocket::bind("0.0.0.0:4000").expect("Failed to bind UDP socket to port 4000");
    println!("UDP server listening on 0.0.0.0:4000");
    let socket = Arc::new(socket);

    // Set up shared resources for networking
    let connected_clients = ConnectedClients::new(Arc::clone(&socket));
    let lobbies: LobbyList = Arc::new(Mutex::new(Vec::new()));

    // Create command channel for networking threads to communicate with Bevy
    let (cmd_sender, cmd_receiver) = std::sync::mpsc::channel::<ServerCommand>();
    let cmd_sender = Arc::new(Mutex::new(cmd_sender));
    let cmd_receiver = Arc::new(Mutex::new(cmd_receiver));

    // Initialize Bevy's task pools
    bevy::tasks::IoTaskPool::get_or_init(|| bevy::tasks::TaskPool::new());

    // Clone for the listener thread
    let connected_clients_clone = ConnectedClients {
        ids: Arc::clone(&connected_clients.ids),
        addrs: Arc::clone(&connected_clients.addrs),
        addr_to_id: Arc::clone(&connected_clients.addr_to_id),
        last_seen: Arc::clone(&connected_clients.last_seen),
        socket: Arc::clone(&socket),
    };
    let lobbies_clone = Arc::clone(&lobbies);

    // Start the UDP listener in a separate thread
    server_listener(connected_clients_clone, lobbies_clone, Arc::clone(&cmd_sender));

    // Create headless server
    App::new()
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
            Duration::from_millis(16),
        )))
        .insert_resource(connected_clients)
        .insert_resource(Lobbies { list: lobbies })
        .insert_resource(PlayerEntities::default())
        .insert_resource(ServerCommandReceiver { receiver: cmd_receiver })
        .add_systems(Update, (
            process_server_commands_system,
            sync_input_from_lobbies_system,
            physics_simulation_system,
            timeout_cleanup_system,
        ).chain())
        .add_systems(FixedUpdate, broadcast_state_system)
        .run();
}