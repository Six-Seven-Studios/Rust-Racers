use bevy::prelude::*;
use crate::GameState;

#[derive(Component)]
pub struct LobbyScreenEntity;

#[derive(Component)]
pub struct PlayerSlot {
    pub slot_index: usize,
}

#[derive(Component)]
pub struct LobbyCodeText;

#[derive(Resource, Default)]
pub struct LobbyState {
    pub connected_players: Vec<String>,
    pub server_ip: String,
}

pub fn setup_lobby(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    lobby_state: &LobbyState,
){
    commands.spawn((
        Text2d::new("Lobby"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., 300., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        LobbyScreenEntity,
    ));
    commands.spawn((
        Text2d::new(format!("IP: {}", lobby_state.server_ip)),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(450., 300., 1.),
            ..default()
        },
        TextFont {
            font_size: 30.0,
            ..default()
        },
        LobbyScreenEntity,
        LobbyCodeText,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/backArrow.png")),
        Transform {
            translation: Vec3::new(-570., 300., 1.),
            ..default()
        },
        LobbyScreenEntity
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/keyEsc.png")),
        Transform {
            translation: Vec3::new(-570., 220., 1.),
            ..default()
        },
        LobbyScreenEntity
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/slantedButton.png")),
        Transform {
            translation: Vec3::new(0., -300., 1.),
            ..default()
        },
        LobbyScreenEntity
    ));
    commands.spawn((
        Text2d::new("GO!"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., -300., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        LobbyScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/key1.png")),
        Transform {
            translation: Vec3::new(-250., -300., 1.),
            ..default()
        },
        LobbyScreenEntity
    ));

    // Spawn player slots dynamically based on connected players
    let player_icons = [
        "player-icons/human1.png",
        "player-icons/human2.png",
        "player-icons/human3.png",
        "player-icons/human4.png",
    ];

    for (i, player_name) in lobby_state.connected_players.iter().enumerate().take(4) {
        let y_pos = 150. - (i as f32 * 100.);

        // Nameplate
        commands.spawn((
            Sprite::from_image(asset_server.load("title_screen/namePlate.png")),
            Transform {
                translation: Vec3::new(25., y_pos, 1.),
                ..default()
            },
            LobbyScreenEntity,
            PlayerSlot { slot_index: i },
        ));

        // Player icon
        commands.spawn((
            Sprite::from_image(asset_server.load(player_icons[i])),
            Transform {
                translation: Vec3::new(-225., y_pos, 1.),
                ..default()
            },
            LobbyScreenEntity,
            PlayerSlot { slot_index: i },
        ));

        // Player name
        commands.spawn((
            Text2d::new(player_name.clone()),
            TextColor(Color::BLACK),
            Transform {
                translation: Vec3::new(0., y_pos, 1.),
                ..default()
            },
            TextFont {
                font_size: 40.0,
                ..default()
            },
            LobbyScreenEntity,
            PlayerSlot { slot_index: i },
        ));
    }
}

pub fn update_lobby_players(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut lobby_state: ResMut<LobbyState>,
    connected_clients: Res<crate::server::ConnectedClients>,
    network_server: Res<crate::networking::NetworkServer>,
    network_client: Res<crate::networking::NetworkClient>,
    current_state: Res<State<GameState>>,
    existing_slots: Query<Entity, With<PlayerSlot>>,
) {
    if *current_state.get() != GameState::Lobby {
        return;
    }

    let mut new_players = Vec::new();
    let my_player_id = network_client.player_id;

    // Check if we have synchronized lobby state from server
    if let Ok(server_lobby) = network_server.lobby_state.lock() {
        if let Some(ref sync_lobby) = *server_lobby {
            // Use synchronized lobby state (for clients)
            for &player_id in &sync_lobby.player_ids {
                let player_number = player_id + 1; // Display as 1-indexed
                let player_name = if Some(player_id) == my_player_id {
                    format!("Player {} (You)", player_number)
                } else {
                    format!("Player {}", player_number)
                };
                new_players.push(player_name);
            }
        } else if network_client.target_ip.is_none() {
            // We're the host and haven't received sync yet - show host view
            if let Ok(client_ids) = connected_clients.client_ids.lock() {
                // Host is always Player 1
                new_players.push("Player 1 (You)".to_string());

                // Add connected clients as Player 2, 3, 4...
                for client_id in client_ids.iter() {
                    let player_number = client_id + 1;
                    new_players.push(format!("Player {}", player_number));
                }
            }
        } else {
            // Client waiting for lobby sync
            if lobby_state.connected_players.is_empty() || lobby_state.connected_players[0] == "Connecting..." {
                return; // Keep showing "Connecting..."
            }
        }
    }

    // Only update if the player list has changed
    if new_players != lobby_state.connected_players && !new_players.is_empty() {
        // Remove old player slot entities
        for entity in existing_slots.iter() {
            commands.entity(entity).despawn();
        }

        // Update the lobby state
        lobby_state.connected_players = new_players;

        // Spawn new player slots
        let player_icons = [
            "player-icons/human1.png",
            "player-icons/human2.png",
            "player-icons/human3.png",
            "player-icons/human4.png",
        ];

        for (i, player_name) in lobby_state.connected_players.iter().enumerate().take(4) {
            let y_pos = 150. - (i as f32 * 100.);

            // Nameplate
            commands.spawn((
                Sprite::from_image(asset_server.load("title_screen/namePlate.png")),
                Transform {
                    translation: Vec3::new(25., y_pos, 1.),
                    ..default()
                },
                LobbyScreenEntity,
                PlayerSlot { slot_index: i },
            ));

            // Player icon
            commands.spawn((
                Sprite::from_image(asset_server.load(player_icons[i])),
                Transform {
                    translation: Vec3::new(-225., y_pos, 1.),
                    ..default()
                },
                LobbyScreenEntity,
                PlayerSlot { slot_index: i },
            ));

            // Player name
            commands.spawn((
                Text2d::new(player_name.clone()),
                TextColor(Color::BLACK),
                Transform {
                    translation: Vec3::new(0., y_pos, 1.),
                    ..default()
                },
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                LobbyScreenEntity,
                PlayerSlot { slot_index: i },
            ));
        }
    }
}
