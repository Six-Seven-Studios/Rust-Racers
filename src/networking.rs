use bevy::prelude::*;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CarPosition {
    pub player_id: u32,
    pub x: f32,
    pub y: f32,
    pub angle: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkMessage {
    Position(CarPosition),
    AllPositions(Vec<CarPosition>),
}

#[derive(Component)]
pub struct NetworkedCar {
    pub player_id: u32,
}

#[derive(Component)]
pub struct LocalPlayer {
    pub player_id: u32,
}

#[derive(Resource)]
pub struct NetworkClient {
    pub stream: Arc<Mutex<Option<TcpStream>>>,
    pub player_id: Option<u32>,
    pub last_connection_attempt: Option<Instant>,
    pub connection_attempted: bool,
    pub last_position_send: Option<Instant>,
}

impl Default for NetworkClient {
    fn default() -> Self {
        Self {
            stream: Arc::new(Mutex::new(None)),
            player_id: None,
            last_connection_attempt: None,
            connection_attempted: false,
            last_position_send: None,
        }
    }
}

#[derive(Resource)]
pub struct NetworkServer {
    pub car_positions: Arc<Mutex<HashMap<u32, CarPosition>>>,
}

impl Default for NetworkServer {
    fn default() -> Self {
        Self {
            car_positions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<NetworkClient>()
            .init_resource::<NetworkServer>()
            .add_systems(Update, (
                connect_to_server.run_if(in_state(crate::GameState::Playing)),
            ));
    }
}

fn connect_to_server(
    mut network_client: ResMut<NetworkClient>,
    mut local_player: Query<&mut LocalPlayer>,
) {
    if network_client.connection_attempted {
        return;
    }

    let is_connected = {
        if let Ok(stream_guard) = network_client.stream.lock() {
            stream_guard.is_some()
        } else {
            false
        }
    };

    if is_connected {
        return;
    }

    network_client.connection_attempted = true;
    network_client.last_connection_attempt = Some(Instant::now());

    match TcpStream::connect("127.0.0.1:4000") {
        Ok(stream) => {
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            if let Ok(_) = reader.read_line(&mut line) {
                if line.starts_with("WELCOME PLAYER") {
                    if let Ok(id) = line.split_whitespace().nth(2).unwrap_or("1").parse::<u32>() {
                        network_client.player_id = Some(id);

                        if let Ok(mut player) = local_player.single_mut() {
                            player.player_id = id;
                        }
                    }
                }
            }

            if stream.set_nonblocking(true).is_err() {
                return;
            }

            if let Ok(mut stream_guard) = network_client.stream.lock() {
                *stream_guard = Some(stream);
            }
        }
        Err(_) => {
            // connection failed (no server running)
        }
    }
}