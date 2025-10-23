use bevy::prelude::*;
use std::sync::mpsc::{self, Receiver};
use std::sync::Mutex;
use std::collections::HashMap;
use crate::networking::{Client, IncomingMessage, ServerMessage, PlayerPositionData, spawn_listener_thread};
use crate::lobby::{LobbyState, setup_lobby};
use crate::GameState;
use crate::title_screen::destroy_screen;

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

        app
            .insert_resource(NetworkClient::default())
            .insert_resource(MessageReceiver { receiver: Mutex::new(receiver) })
            .insert_resource(MessageSender { sender })
            .insert_resource(PlayerPositions::default())
            .add_systems(Update, process_network_messages);
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
                        println!("Active lobbies:");
                        for lobby in lobbies {
                            println!("  {} ({} players)", lobby.name, lobby.players);
                        }
                    }
                    ServerMessage::GameStarted { lobby } => {
                        println!("Game started for lobby: {}", lobby);

                        // Destroy lobby screen entities
                        for entity in lobby_query.iter() {
                            commands.entity(entity).despawn();
                        }

                        // Transition to Playing state
                        next_state.set(GameState::Playing);
                    }
                }
            }

            IncomingMessage::LobbyState(state) => {
                println!("Lobby state update: {} - {:?} players", state.lobby, state.players);

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
    let client = Client::connect(address.to_string())
        .map_err(|e| format!("Failed to connect: {}", e))?;

    // Clone the stream for the listener thread
    let stream_clone = client.get_stream_clone()
        .map_err(|e| format!("Failed to clone stream: {}", e))?;

    // Spawn the listener thread
    spawn_listener_thread(stream_clone, sender.sender.clone());

    network_client.client = Some(client);

    Ok(())
}
