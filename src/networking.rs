use serde::{Serialize, Deserialize};
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;
use std::sync::mpsc::{self, Sender, Receiver};

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum MessageType {
    CreateLobby { name: String },

    JoinLobby { name: String },

    LeaveLobby { name: String },

    ListLobbies,

    StartLobby { name: String },

    CarPosition { x: f32, y: f32, vx: f32, vy: f32, angle: f32 },

    PlayerInput {
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        drift: bool,
    },
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
    GameStarted { lobby: String },
}

#[derive(Debug, Clone, Deserialize)]
pub struct LobbyInfo {
    pub name: String,
    pub players: usize,
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
    pub input_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PositionsMessage {
    pub players: Vec<PlayerPositionData>,
}

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn connect(address: String) -> io::Result<Self> {
        let stream = TcpStream::connect(&address)?;
        // Removed read timeout, we'll handle reading in a separate thread
        println!("Connected to server at {}", address);
        Ok(Self { stream })
    }

    // Get a cloned stream for the listener thread
    pub fn get_stream_clone(&self) -> io::Result<TcpStream> {
        self.stream.try_clone()
    }

    pub fn send(&mut self, message: MessageType) -> io::Result<()> {
        let text = serde_json::to_string(&message).unwrap() + "\n";
        self.stream.write_all(text.as_bytes())?;
        self.stream.flush()?;
        Ok(())
    }

    pub fn create_lobby(&mut self, name: String) -> io::Result<()> {
        self.send(MessageType::CreateLobby { name })
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

    pub fn send_car_position(&mut self, x: f32, y: f32, vx: f32, vy: f32, angle: f32) -> io::Result<()> {
        self.send(MessageType::CarPosition { x, y, vx, vy, angle })
    }

    pub fn send_player_input(&mut self, forward: bool, backward: bool, left: bool, right: bool, drift: bool) -> io::Result<()> {
        self.send(MessageType::PlayerInput { forward, backward, left, right, drift })
    }

    pub fn try_read_message(&mut self) -> io::Result<Option<String>> {
        let mut line = String::new();
        match BufReader::new(&self.stream).read_line(&mut line) {
            Ok(0) | Ok(_) if line.trim().is_empty() => Ok(None),
            Ok(_) => Ok(Some(line)),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(e),
        }
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
pub fn spawn_listener_thread(stream: TcpStream, sender: Sender<IncomingMessage>) {
    std::thread::spawn(move || {
        let mut reader = BufReader::new(stream);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => {
                    println!("Server disconnected");
                    break;
                }
                Ok(_) => {
                    let trimmed = line.trim();
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
                Err(e) => {
                    println!("Error reading from server: {}", e);
                    break;
                }
            }
        }
    });
}