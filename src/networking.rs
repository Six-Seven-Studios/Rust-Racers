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
            .init_resource::<NetworkServer>();
    }
}