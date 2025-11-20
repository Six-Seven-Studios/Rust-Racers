use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::Instant;

// Single input with sequence number (shared with client)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputData {
    pub sequence: u64,
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub drift: bool,
    pub easy_drift: bool,
}

// Single position snapshot with sequence number
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PositionSnapshot {
    pub sequence: u64,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub angle: f32,
}

// Server doesn't use GameState but lap_system needs it
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Title,
    Lobby,
    Creating,
    Joining,
    Customizing,
    Settings,
    Playing,
    PlayingDemo,
    Victory,
    Credits,
}

// Message types from clients
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum MessageType {
    CreateLobby { name: String },
    JoinLobby { name: String },
    LeaveLobby { name: String },
    ListLobbies,
    StartLobby { name: String },
    PlayerInput {
        sequence: u64,
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        drift: bool,
        easy_drift: bool,
    },
    PlayerInputBuffer {
        inputs: Vec<InputData>,
    },
    Ping,
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

impl ConnectedClients {
    pub fn new(socket: Arc<UdpSocket>) -> Self {
        Self {
            ids: Arc::new(Mutex::new(Vec::new())),
            addrs: Arc::new(Mutex::new(HashMap::new())),
            addr_to_id: Arc::new(Mutex::new(HashMap::new())),
            last_seen: Arc::new(Mutex::new(HashMap::new())),
            socket,
        }
    }
}

// Player input state
#[derive(Clone, Debug)]
pub struct PlayerInput {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub drift: bool,
    pub easy_drift: bool,
}

impl Default for PlayerInput {
    fn default() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,
            drift: false,
            easy_drift: false,
        }
    }
}

// Player state tracked by server
#[derive(Clone, Debug)]
pub struct PlayerState {
    pub x: f32,
    pub y: f32,
    pub velocity: bevy::math::Vec2,
    pub angle: f32,
    pub inputs: PlayerInput,
    pub last_processed_sequence: u64,
    // Queue of pending inputs to process
    pub input_queue: Vec<InputData>,
}

// Lobby structure
#[derive(Clone)]
pub struct Lobby {
    pub players: Arc<Mutex<Vec<u32>>>,
    pub host: u32,
    pub name: String,
    pub started: bool,
    pub states: Arc<Mutex<HashMap<u32, PlayerState>>>,
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

pub type LobbyList = Arc<Mutex<Vec<Lobby>>>;

// Resource wrapper for LobbyList
#[derive(Resource, Clone)]
pub struct Lobbies {
    pub list: LobbyList,
}

// ECS Components for server simulation
#[derive(Component)]
pub struct PlayerId(pub u32);

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

// Component for player input state
#[derive(Component)]
pub struct PlayerInputComponent {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub drift: bool,
    pub easy_drift: bool,
    pub last_processed_sequence: u64,
}

impl Default for PlayerInputComponent {
    fn default() -> Self {
        Self {
            forward: false,
            backward: false,
            left: false,
            right: false,
            drift: false,
            easy_drift: false,
            last_processed_sequence: 0,
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

// Resource for receiving commands in Bevy systems
#[derive(Resource)]
pub struct ServerCommandSender {
    pub sender: Arc<Mutex<std::sync::mpsc::Sender<ServerCommand>>>,
}
