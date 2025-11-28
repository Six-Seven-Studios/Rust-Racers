use crate::GameState;
use crate::lobby::{LobbyInfo, LobbyList, LobbyListDirty, LobbyState, setup_lobby};
use crate::networking::{
    Client, IncomingMessage, PlayerPositionData, ServerMessage, spawn_listener_thread,
};
use crate::title_screen::destroy_screen;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// Resource to hold the client connection
#[derive(Resource)]
pub struct NetworkClient {
    pub client: Option<Client>,
    pub player_id: Option<u32>,
    pub current_lobby: Option<String>,
}

impl Default for NetworkClient {
    fn default() -> Self {
        Self {
            client: None,
            player_id: None,
            current_lobby: None,
        }
    }
}

#[derive(Debug, Clone, Resource)]
pub struct Latency {
    now: Arc<Mutex<Instant>>,
    average_latency: Arc<Mutex<f32>>,
    count: Arc<Mutex<u32>>,
}

impl Default for Latency {
    fn default() -> Self {
        Self {
            now: Arc::new(Mutex::new(Instant::now())),
            average_latency: Arc::new(Mutex::new(0.0)),
            count: Arc::new(Mutex::new(0)),
        }
    }
}

// Resource to hold the message receiver
#[derive(Resource)]
pub struct MessageReceiver {
    pub receiver: Mutex<Receiver<IncomingMessage>>,
}

// Resource to hold player positions received from server
#[derive(Resource, Default)]
pub struct PlayerPositions {
    pub positions: HashMap<u32, PlayerPositionData>,
}

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        // Create the message channel
        let (sender, receiver) = mpsc::channel();

        app.insert_resource(NetworkClient::default())
            .insert_resource(MessageReceiver {
                receiver: Mutex::new(receiver),
            })
            .insert_resource(MessageSender { sender })
            .insert_resource(PlayerPositions::default())
            .insert_resource(Latency::default())
            .add_systems(Update, process_network_messages)
            .add_systems(
                Update,
                ping_server_system.run_if(on_timer(Duration::from_secs(5))),
            );
    }
}

// Resource to hold the sender
#[derive(Resource, Clone)]
pub struct MessageSender {
    pub sender: std::sync::mpsc::Sender<IncomingMessage>,
}

// System to process incoming network messages
fn process_network_messages(
    receiver: Res<MessageReceiver>,
    mut network_client: ResMut<NetworkClient>,
    mut lobby_state: ResMut<LobbyState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    lobby_query: Query<Entity, With<crate::lobby::LobbyScreenEntity>>,
    mut player_positions: ResMut<PlayerPositions>,
    asset_server: Res<AssetServer>,
    mut list: ResMut<LobbyList>,
    mut dirty: ResMut<LobbyListDirty>,
    latency: Res<Latency>,
) {
    // Lock the receiver to access it
    let rx = receiver.receiver.lock().unwrap();

    // Process all pending messages
    while let Ok(message) = rx.try_recv() {
        match message {
            IncomingMessage::Welcome(player_id) => {
                println!("Connected as Player {}", player_id);
                network_client.player_id = Some(player_id);
            }

            IncomingMessage::ServerMessage(msg) => {
                match msg {
                    ServerMessage::Confirmation { message } => {
                        println!("Server: {}", message);
                    }
                    ServerMessage::Error { message } => {
                        println!("Error: {}", message);
                    }
                    ServerMessage::ActiveLobbies { lobbies } => {
                        list.0.clear();
                        println!("Active lobbies:");
                        for lobby in lobbies {
                            println!("  {} ({} players)", lobby.name, lobby.players);
                            list.0.push(LobbyInfo {
                                name: lobby.name,
                                players: lobby.players,
                                capacity: 4,
                            });
                        }
                        dirty.0 = true;
                    }
                    ServerMessage::GameStarted { lobby, time } => {
                        println!("Game started for lobby: {}", lobby);

                        // Destroy lobby screen entities
                        for entity in lobby_query.iter() {
                            commands.entity(entity).despawn();
                        }

                        let average_latency = latency.average_latency.lock().unwrap();

                        // Sleep to mimick the synchronized start - account for latency delays
                        thread::sleep(Duration::from_millis(time - *average_latency as u64));

                        // Transition to Playing state
                        next_state.set(GameState::Playing);
                    }
                    ServerMessage::Pong => {
                        let now = Instant::now();
                        let mut time = latency.now.lock().unwrap();
                        let new_latency = now.duration_since(*time).as_millis() / 2;

                        let mut count = latency.count.lock().unwrap();
                        *count += 1;

                        let mut average_latency = latency.average_latency.lock().unwrap();
                        *average_latency += (new_latency as f32 - *average_latency) / *count as f32;

                        println!("Received Pong");
                    }
                }
            }

            IncomingMessage::LobbyState(state) => {
                println!(
                    "Lobby state update: {} - {:?} players",
                    state.lobby, state.players
                );

                // Update the lobby state resource
                lobby_state.name = state.lobby.clone();
                network_client.current_lobby = Some(state.lobby);

                // Convert player IDs to display names
                lobby_state.connected_players.clear();
                for (i, player_id) in state.players.iter().enumerate() {
                    let is_you = Some(*player_id) == network_client.player_id;
                    let name = if is_you {
                        format!("Player {} (You)", player_id)
                    } else {
                        format!("Player {}", player_id)
                    };
                    lobby_state.connected_players.push(name);
                }

                destroy_screen(&mut commands, &lobby_query);
                setup_lobby(&mut commands, asset_server.clone(), &lobby_state);
            }

            IncomingMessage::Positions(pos_msg) => {
                // Update player positions
                for player_pos in pos_msg.players {
                    player_positions.positions.insert(player_pos.id, player_pos);
                }
            }
        }
    }
}

// Helper function to connect to server
pub fn connect_to_server(
    network_client: &mut NetworkClient,
    sender: &MessageSender,
    address: &str,
) -> Result<(), String> {
    let client =
        Client::connect(address.to_string()).map_err(|e| format!("Failed to connect: {}", e))?;

    // Clone the socket for the listener thread
    let socket_clone = client
        .get_socket_clone()
        .map_err(|e| format!("Failed to clone socket: {}", e))?;

    // Spawn the listener thread
    spawn_listener_thread(socket_clone, sender.sender.clone());

    network_client.client = Some(client);

    Ok(())
}

// Function to ping the server
pub fn ping_server_system(mut network_client: ResMut<NetworkClient>, latency: Res<Latency>) {
    if network_client.client.is_none() {
        println!("No connection to server");
        return;
    }

    let mut time = latency.now.lock().unwrap();
    *time = Instant::now();

    if let Some(client) = &mut network_client.client {
        if let Err(e) = client.send_ping() {
            println!("Failed to send ping: {}", e);
            return;
        } else {
            println!("Pinged server");
        }
    }
}
