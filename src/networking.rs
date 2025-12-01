use bevy::{prelude::Resource, tasks::IoTaskPool};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use std::{io, thread};

// Single input with sequence number
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputData {
    pub sequence: u64,
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub drift: bool,
    pub easy_drift: bool,
    #[serde(default)]
    pub boost: bool,
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapChoice {
    #[serde(rename = "small")]
    Small,
    #[serde(rename = "big")]
    Big,
}

impl MapChoice {
    pub fn label(self) -> &'static str {
        match self {
            MapChoice::Small => "Small Map",
            MapChoice::Big => "Big Map",
        }
    }

    pub fn path(self) -> &'static str {
        match self {
            MapChoice::Small => "assets/big-map.txt",
            MapChoice::Big => "assets/map2.txt",
        }
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct SelectedMap {
    pub choice: MapChoice,
}

impl Default for SelectedMap {
    fn default() -> Self {
        Self {
            choice: MapChoice::Small,
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum MessageType {
    CreateLobby {
        name: String,
        map: MapChoice,
    },

    JoinLobby {
        name: String,
    },

    LeaveLobby {
        name: String,
    },

    ListLobbies,

    StartLobby {
        name: String,
    },

    CarPosition {
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        angle: f32,
    },

    PlayerInput {
        sequence: u64,
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        drift: bool,
        easy_drift: bool,
        boost: bool,
    },

    // New buffered input message
    PlayerInputBuffer {
        inputs: Vec<InputData>,
    },

    Ping,
}

// Server response messages
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "confirmation")]
    Confirmation { message: String },

    #[serde(rename = "error")]
    Error { message: String },

    #[serde(rename = "active_lobbies")]
    ActiveLobbies { lobbies: Vec<LobbyInfo> },

    #[serde(rename = "game_started")]
    GameStarted { lobby: String, time: u64, map: MapChoice },

    #[serde(rename = "pong")]
    Pong,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LobbyInfo {
    pub name: String,
    pub players: usize,
    pub map: MapChoice,
}

// Lobby state broadcast message
#[derive(Debug, Clone, Deserialize)]
pub struct LobbyStateMessage {
    pub lobby: String,
    pub players: Vec<u32>,
}

// Position message for car positions
#[derive(Debug, Clone, Deserialize)]
pub struct PlayerPositionData {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub angle: f32,
    #[serde(default)]
    pub last_processed_sequence: u64,
    // Array of position snapshots (one per processed input)
    #[serde(default)]
    pub snapshots: Vec<PositionSnapshot>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PositionsMessage {
    pub players: Vec<PlayerPositionData>,
}

pub struct Client {
    socket: UdpSocket,
    server_addr: SocketAddr,
}

impl Client {
    pub fn connect(address: String) -> io::Result<Self> {
        // Parse the server address
        let server_addr: SocketAddr = address.parse().map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid address: {}", e),
            )
        })?;

        // Bind to any available local port
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        println!("Connected to server at {}", address);

        Ok(Self {
            socket,
            server_addr,
        })
    }

    // Get a cloned socket for the listener thread
    pub fn get_socket_clone(&self) -> io::Result<UdpSocket> {
        self.socket.try_clone()
    }

    pub fn clone(&self) -> Self {
        Self {
            socket: self.socket.try_clone().expect("Failed to clone UDP socket"),
            server_addr: self.server_addr,
        }
    }

    pub fn send(&mut self, message: MessageType) -> io::Result<()> {
        let text = serde_json::to_string(&message).unwrap() + "\n";
        self.socket.send_to(text.as_bytes(), self.server_addr)?;
        Ok(())
    }

    pub fn create_lobby(&mut self, name: String, map: MapChoice) -> io::Result<()> {
        self.send(MessageType::CreateLobby { name, map })
    }

    pub fn join_lobby(&mut self, name: String) -> io::Result<()> {
        self.send(MessageType::JoinLobby { name })
    }

    pub fn leave_lobby(&mut self, name: String) -> io::Result<()> {
        self.send(MessageType::LeaveLobby { name })
    }

    /// Asks the server to list active lobbies.
    pub fn list_lobbies(&mut self) -> io::Result<()> {
        self.send(MessageType::ListLobbies)
    }
    pub fn start_lobby(&mut self, name: String) -> io::Result<()> {
        self.send(MessageType::StartLobby { name })
    }

    pub fn send_player_input(
        &mut self,
        sequence: u64,
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        drift: bool,
        easy_drift: bool,
        boost: bool,
    ) -> io::Result<()> {
        self.send(MessageType::PlayerInput {
            sequence,
            forward,
            backward,
            left,
            right,
            drift,
            easy_drift,
            boost,
        })
    }

    pub fn send_player_input_buffer(&mut self, inputs: Vec<InputData>) -> io::Result<()> {
        self.send(MessageType::PlayerInputBuffer { inputs })
    }

    pub fn send_ping(&mut self) -> io::Result<()> {
        self.send(MessageType::Ping)
    }
}

// Enum to represent all possible incoming messages
#[derive(Debug, Clone)]
pub enum IncomingMessage {
    Welcome(u32),
    ServerMessage(ServerMessage),
    LobbyState(LobbyStateMessage),
    Positions(PositionsMessage),
}

// Function to spawn a listener thread that continuously reads from server
pub fn spawn_listener_thread(socket: UdpSocket, sender: Sender<IncomingMessage>) {
    let task_pool = IoTaskPool::get();
    task_pool
        .spawn(async move {
            let mut buf = [0u8; 65536]; // UDP buffer

            loop {
                match socket.recv_from(&mut buf) {
                    Ok((len, _addr)) => {
                        let data = &buf[..len];

                        // Convert bytes to string
                        if let Ok(message_str) = std::str::from_utf8(data) {
                            let trimmed = message_str.trim();
                            if trimmed.is_empty() {
                                continue;
                            }

                            // Try to parse welcome message
                            if trimmed.starts_with("WELCOME PLAYER ") {
                                if let Some(id_str) = trimmed.strip_prefix("WELCOME PLAYER ") {
                                    if let Ok(player_id) = id_str.parse::<u32>() {
                                        let _ = sender.send(IncomingMessage::Welcome(player_id));
                                    }
                                }
                                continue;
                            }

                            // Try parsing as ServerMessage
                            if let Ok(msg) = serde_json::from_str::<ServerMessage>(trimmed) {
                                let _ = sender.send(IncomingMessage::ServerMessage(msg));
                                continue;
                            }

                            // Try parsing as LobbyStateMessage
                            if let Ok(msg) = serde_json::from_str::<LobbyStateMessage>(trimmed) {
                                let _ = sender.send(IncomingMessage::LobbyState(msg));
                                continue;
                            }

                            // Try parsing as PositionsMessage
                            if let Ok(msg) = serde_json::from_str::<PositionsMessage>(trimmed) {
                                let _ = sender.send(IncomingMessage::Positions(msg));
                                continue;
                            }

                            println!("Unknown message from server: {}", trimmed);
                        }
                    }
                    Err(e) => {
                        println!("Error receiving from server: {}", e);
                        break;
                    }
                }
            }
        })
        .detach();
}
