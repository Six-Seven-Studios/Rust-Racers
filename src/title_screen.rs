use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use crate::GameState;
use crate::get_ip::get_local_ip;

#[derive(Component)]
pub struct MainScreenEntity;

#[derive(Component)]
pub struct LobbyScreenEntity;

#[derive(Component)]
pub struct JoinScreenEntity;

#[derive(Component)]
pub struct SettingsScreenEntity;

#[derive(Component)]
pub struct CustomizingScreenEntity;

#[derive(Component)]
pub struct PlayerSlot {
    pub slot_index: usize,
}

#[derive(Component)]
pub struct LobbyCodeText;

#[derive(Component)]
pub struct IpInputText;

#[derive(Resource, Default)]
pub struct LobbyState {
    pub connected_players: Vec<String>,
    pub server_ip: String,
}

#[derive(Resource, Default)]
pub struct IpInputState {
    pub input: String,
}

pub fn check_for_title_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    title_query: Query<Entity, With<MainScreenEntity>>,
    lobby_query: Query<Entity, With<LobbyScreenEntity>>,
    join_query: Query<Entity, With<JoinScreenEntity>>,
    settings_query: Query<Entity, With<SettingsScreenEntity>>,
    customize_query: Query<Entity, With<CustomizingScreenEntity>>,
    mut lobby_state: ResMut<LobbyState>,
    mut ip_input_state: ResMut<IpInputState>,
    mut network_client: ResMut<crate::networking::NetworkClient>,
) {

    match *current_state.get() {
        GameState::Title => {
            if input.just_pressed(KeyCode::Digit1){
                next_state.set(GameState::Lobby);
                destroy_screen(&mut commands, &title_query);

                // Initialize lobby with host player and get server IP
                lobby_state.connected_players.clear();
                lobby_state.connected_players.push("Host (You)".to_string());
                if let Ok(ip) = get_local_ip() {
                    lobby_state.server_ip = ip;
                } else {
                    lobby_state.server_ip = "0.0.0.0".to_string();
                }

                setup_lobby(commands, asset_server, &lobby_state);
            }
            else if input.just_pressed(KeyCode::Digit2){
                next_state.set(GameState::Joining);
                destroy_screen(&mut commands, &title_query);

                // Initialize IP input with default value
                ip_input_state.input = "127.0.0.1".to_string();

                setup_join(commands, asset_server);

            }
            else if input.just_pressed(KeyCode::Digit3){
                next_state.set(GameState::Customizing);
                destroy_screen(&mut commands, &title_query);
                setup_customizing(commands, asset_server);
            }
            else if input.just_pressed(KeyCode::Escape){
                next_state.set(GameState::Settings);
                destroy_screen(&mut commands, &title_query);
                setup_settings(commands, asset_server);
            }
        }
        GameState::Lobby => {
            if input.just_pressed(KeyCode::Escape){
                next_state.set(GameState::Title);
                destroy_screen(&mut commands, &lobby_query);
                setup_title_screen(commands, asset_server);
            }
            else if input.just_pressed(KeyCode::Digit1){
                next_state.set(GameState::Playing);
                destroy_screen(&mut commands, &lobby_query);
            }
        }
        GameState::Joining => {
            if input.just_pressed(KeyCode::Escape){
                next_state.set(GameState::Title);
                destroy_screen(&mut commands, &join_query);
                setup_title_screen(commands, asset_server);
            }
            else if input.just_pressed(KeyCode::Enter){
                // Store the IP address and connect
                network_client.target_ip = Some(ip_input_state.input.clone());
                network_client.connection_attempted = false;

                // Go to lobby instead of directly to playing
                next_state.set(GameState::Lobby);
                destroy_screen(&mut commands, &join_query);

                // Initialize lobby state for client
                lobby_state.connected_players.clear();
                lobby_state.connected_players.push("Connecting...".to_string());
                lobby_state.server_ip = ip_input_state.input.clone();

                setup_lobby(commands, asset_server, &lobby_state);
            }
        }
        GameState::Customizing => {
            if input.just_pressed(KeyCode::Escape){
                next_state.set(GameState::Title);
                destroy_screen(&mut commands, &customize_query);
                setup_title_screen(commands, asset_server);
            }
        }
        GameState::Settings => {
            if input.just_pressed(KeyCode::Escape){
                next_state.set(GameState::Title);
                destroy_screen(&mut commands, &settings_query);
                setup_title_screen(commands, asset_server);
            }
        }
        _ => {
            return;
        }
    }

}

pub fn setup_title_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/settingsGear.png")),
        Transform {
            translation: Vec3::new(-570., 300., 1.),
            ..default()
        },
        MainScreenEntity
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/keyEsc.png")),
        Transform {
            translation: Vec3::new(-570., 220., 1.),
            ..default()
        },
        MainScreenEntity
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/slantedButton.png")),
        Transform {
            translation: Vec3::new(0., -100., 1.),
            ..default()
        },
        MainScreenEntity
    ));
    commands.spawn((
        Text2d::new("CREATE"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., -100., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        MainScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/key1.png")),
        Transform {
            translation: Vec3::new(-250., -100., 1.),
            ..default()
        },
        MainScreenEntity
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/slantedButton.png")),
        Transform {
            translation: Vec3::new(0., -200., 1.),
            ..default()
        },
        MainScreenEntity
    ));
    commands.spawn((
        Text2d::new("JOIN"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., -200., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        MainScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/key2.png")),
        Transform {
            translation: Vec3::new(-250., -200., 1.),
            ..default()
        },
        MainScreenEntity
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/slantedButton.png")),
        Transform {
            translation: Vec3::new(0., -300., 1.),
            ..default()
        },
        MainScreenEntity
    ));
    commands.spawn((
        Text2d::new("CUSTOMIZE"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., -300., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        MainScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/key3.png")),
        Transform {
            translation: Vec3::new(-250., -300., 1.),
            ..default()
        },
        MainScreenEntity
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/rustRacersLogo.png")),
        Transform {
            translation: Vec3::new(0., 100., 1.),
            ..default()
        },
        MainScreenEntity
    ));

}

fn setup_lobby(
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

fn setup_join(
    mut commands: Commands,
    asset_server: Res<AssetServer>){
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/backArrow.png")),
        Transform {
            translation: Vec3::new(-570., 300., 1.),
            ..default()
        },
        JoinScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/keyEsc.png")),
        Transform {
            translation: Vec3::new(-570., 220., 1.),
            ..default()
        },
        JoinScreenEntity
    ));
    commands.spawn((
        Text2d::new("Join A Lobby"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., 300., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        JoinScreenEntity,
    ));
    commands.spawn((
        Text2d::new("Input IP Address:"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., 100., 1.),
            ..default()
        },
        TextFont {
            font_size: 35.0,
            ..default()
        },
        JoinScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/lobbyInput.png")),
        Transform {
            translation: Vec3::new(0., 0., 1.),
            ..default()
        },
        JoinScreenEntity
    ));
    commands.spawn((
        Text2d::new("127.0.0.1"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., 0., 1.),
            ..default()
        },
        TextFont {
            font_size: 40.0,
            ..default()
        },
        JoinScreenEntity,
        IpInputText,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/slantedButton.png")),
        Transform {
            translation: Vec3::new(0., -300., 1.),
            ..default()
        },
        JoinScreenEntity
    ));
    commands.spawn((
        Text2d::new("JOIN!"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., -300., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        JoinScreenEntity,
    ));
    commands.spawn((
        Text2d::new("Press ENTER"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(-350., -300., 1.),
            ..default()
        },
        TextFont {
            font_size: 30.0,
            ..default()
        },
        JoinScreenEntity,
    ));
}
fn setup_settings(
    mut commands: Commands,
    asset_server: Res<AssetServer>){
    commands.spawn((
        Text2d::new("Welcome to Settings"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., 0., 1.),
            ..default()
        },
        SettingsScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/backArrow.png")),
        Transform {
            translation: Vec3::new(-570., 300., 1.),
            ..default()
        },
        SettingsScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/keyEsc.png")),
        Transform {
            translation: Vec3::new(-570., 220., 1.),
            ..default()
        },
        SettingsScreenEntity
    ));
}
fn setup_customizing(
    mut commands: Commands,
    asset_server: Res<AssetServer>){
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/backArrow.png")),
        Transform {
            translation: Vec3::new(-570., 300., 1.),
            ..default()
        },
        CustomizingScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/keyEsc.png")),
        Transform {
            translation: Vec3::new(-570., 220., 1.),
            ..default()
        },
        CustomizingScreenEntity
    ));
    commands.spawn((
        Text2d::new("Customize"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., 300., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        CustomizingScreenEntity,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/keyA.png")),
        Transform {
            translation: Vec3::new(-150., 0., 1.),
            ..default()
        },
        CustomizingScreenEntity
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/keyD.png")),
        Transform {
            translation: Vec3::new(150., 0., 1.),
            ..default()
        },
        CustomizingScreenEntity
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("car.png")),
        Transform {
            translation: Vec3::new(0., 0., 1.),
            ..default()
        },
        CustomizingScreenEntity
    ));
}

pub fn destroy_screen<CurrentScreen: Component>(
    commands: &mut Commands,
    query: &Query<Entity, With<CurrentScreen>>,
) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

pub fn update_lobby_players(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut lobby_state: ResMut<LobbyState>,
    connected_clients: Res<crate::server::ConnectedClients>,
    network_client: Res<crate::networking::NetworkClient>,
    current_state: Res<State<GameState>>,
    existing_slots: Query<Entity, With<PlayerSlot>>,
) {
    if *current_state.get() != GameState::Lobby {
        return;
    }

    // Get the list of connected players
    let mut new_players = Vec::new();

    // If we're the host (no target_ip), show host + connected clients
    if network_client.target_ip.is_none() {
        new_players.push("Host (You)".to_string());

        if let Ok(client_ids) = connected_clients.client_ids.lock() {
            for client_id in client_ids.iter() {
                new_players.push(format!("Player {}", client_id));
            }
        }
    } else {
        // If we're a client, show the existing lobby state
        // The client will see their own view (could be enhanced later)
        if lobby_state.connected_players.is_empty() || lobby_state.connected_players[0] == "Connecting..." {
            // Still connecting, keep the connecting message
            return;
        }
        return;
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

pub fn handle_ip_input(
    input: Res<ButtonInput<KeyCode>>,
    mut ip_input_state: ResMut<IpInputState>,
    mut text_query: Query<&mut Text2d, With<IpInputText>>,
    current_state: Res<State<GameState>>,
) {
    if *current_state.get() != GameState::Joining {
        return;
    }

    // Don't handle input if Enter or Escape is pressed (let the state transition handler deal with it)
    if input.just_pressed(KeyCode::Enter) || input.just_pressed(KeyCode::Escape) {
        return;
    }

    let mut changed = false;

    // Handle backspace
    if input.just_pressed(KeyCode::Backspace) {
        ip_input_state.input.pop();
        changed = true;
    }

    // Handle period/dot
    if input.just_pressed(KeyCode::Period) && ip_input_state.input.len() < 15 {
        ip_input_state.input.push('.');
        changed = true;
    }

    // Handle digit input (exclude Digit1 and Digit2 which conflict with join/escape)
    let digit_keys = [
        (KeyCode::Digit0, '0'), (KeyCode::Digit1, '1'), (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'), (KeyCode::Digit4, '4'), (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'), (KeyCode::Digit7, '7'), (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
    ];

    for (key, digit) in digit_keys {
        if input.just_pressed(key) && ip_input_state.input.len() < 15 {
            ip_input_state.input.push(digit);
            changed = true;
            break;
        }
    }

    // Update the text display if changed
    if changed {
        if let Ok(mut text) = text_query.single_mut() {
            if ip_input_state.input.is_empty() {
                **text = "...".to_string();
            } else {
                **text = ip_input_state.input.clone();
            }
        }
    }
}
