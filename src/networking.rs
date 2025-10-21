use serde::Serialize;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum MessageType {
    CreateLobby { name: String },

    JoinLobby { name: String },

    LeaveLobby { name: String },

    ListLobbies,

    StartLobby { name: String },

    CarPosition { x: f32, y: f32, vx: f32, vy: f32, angle: f32 },
}

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn connect(address: String) -> io::Result<Self> {
        let stream = TcpStream::connect(&address)?;
        // Sets a read timeout, however this will kill the application so we will have to change it eventually
        // to handle reads better
        stream.set_read_timeout(Some(Duration::from_millis(100)))?;
        println!("Connected to server at {}", address);
        Ok(Self { stream })
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

    pub fn read_message(&mut self) -> io::Result<()> {
        let mut reader = BufReader::new(&self.stream);
        let mut line = String::new();

        reader.read_line(&mut line)?;
        if !line.trim().is_empty() {
            println!("Server says: {}", line.trim());
        }

        Ok(())
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