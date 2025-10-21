use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use crate::GameState;

use bevy::{color::palettes::basic::*, input_focus::InputFocus, prelude::*};
use crate::lobby::{LobbyState, setup_lobby};
use crate::networking_plugin::{NetworkClient, MessageSender, connect_to_server};

#[derive(Component)]
pub struct MainScreenEntity;

#[derive(Component)]
pub struct JoinScreenEntity;

#[derive(Component)]
pub struct SettingsScreenEntity;

#[derive(Component)]
pub struct CustomizingScreenEntity;

#[derive(Component)]
pub struct LobbyNameInput;

#[derive(Component)]
pub struct ServerIpInput;

#[derive(Resource)]
pub struct ServerAddress {
    pub address: String,
}

#[derive(Resource)]
pub struct TypingMode {
    pub is_typing: bool,
}

// System to sync the server IP input text with the ServerAddress resource
pub fn sync_server_address(
    server_ip_query: Query<&Text2d, (With<ServerIpInput>, Changed<Text2d>)>,
    mut server_address: ResMut<ServerAddress>,
) {
    if let Ok(text) = server_ip_query.get_single() {
        server_address.address = text.0.trim().to_string();
    }
}

pub fn check_for_title_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    title_query: Query<Entity, With<MainScreenEntity>>,
    lobby_query: Query<Entity, With<crate::lobby::LobbyScreenEntity>>,
    join_query: Query<Entity, With<JoinScreenEntity>>,
    settings_query: Query<Entity, With<SettingsScreenEntity>>,
    customize_query: Query<Entity, With<CustomizingScreenEntity>>,
    mut lobby_state: ResMut<LobbyState>,
    mut network_client: ResMut<NetworkClient>,
    message_sender: Res<MessageSender>,
    mut lobby_name_query: Query<&mut Text2d, (With<LobbyNameInput>, Without<ServerIpInput>)>,
    mut server_ip_query: Query<&mut Text2d, (With<ServerIpInput>, Without<LobbyNameInput>)>,
    server_address: Res<ServerAddress>,
) {

    match *current_state.get() {
        GameState::Title => {
            // Check if user is typing in the IP field (not default "hi")
            let is_typing_ip = if let Ok(text) = server_ip_query.get_single() {
                !text.0.is_empty() && text.0 != "hi"
            } else {
                false
            };

            // Handle text input for server IP
            for key in input.get_just_pressed() {
                if let Ok(mut text) = server_ip_query.get_single_mut() {
                    match key {
                        KeyCode::Backspace => {
                            text.0.pop();
                        }
                        KeyCode::Period => {
                            text.0.push('.');
                        }
                        KeyCode::Semicolon => {
                            text.0.push(':');
                        }
                        _ => {
                            if let Some(character) = key_to_char(key) {
                                if text.0.len() < 25 {
                                    text.0.push(character);
                                }
                            }
                        }
                    }
                }
            }

            // Only trigger menu actions if NOT typing in IP field
            if !is_typing_ip && input.just_pressed(KeyCode::Digit1){
                let server_addr = format!("{}:4000", server_address.address);

                // Connect to server and create lobby
                if network_client.client.is_none() {
                    match connect_to_server(&mut network_client, &message_sender, &server_addr) {
                        Ok(_) => println!("Connected to server!"),
                        Err(e) => {
                            println!("Failed to connect to server: {}", e);
                            return;
                        }
                    }
                }

                // Send create lobby message
                let lobby_name = "LOBBY 1".to_string();
                if let Some(client) = &mut network_client.client {
                    if let Err(e) = client.create_lobby(lobby_name.clone()) {
                        println!("Failed to create lobby: {}", e);
                        return;
                    }
                }

                // Transition to lobby screen
                next_state.set(GameState::Lobby);
                destroy_screen(&mut commands, &title_query);

                lobby_state.connected_players.clear();
                lobby_state.connected_players.push("Connecting...".to_string());
                lobby_state.name = lobby_name;

                setup_lobby(commands, asset_server, &lobby_state);
            }
            else if !is_typing_ip && input.just_pressed(KeyCode::Digit2){
                next_state.set(GameState::Joining);
                destroy_screen(&mut commands, &title_query);

                setup_join(commands, asset_server);
            }
            else if !is_typing_ip && input.just_pressed(KeyCode::Digit3){
                next_state.set(GameState::Customizing);
                destroy_screen(&mut commands, &title_query);
                setup_customizing(commands, asset_server);
            }
            else if input.just_pressed(KeyCode::Escape){
                next_state.set(GameState::Settings);
                destroy_screen(&mut commands, &title_query);
                setup_settings(commands, asset_server);
            }
            // Theta* DEMO
            else if !is_typing_ip && input.just_pressed(KeyCode::Digit4){
                next_state.set(GameState::PlayingDemo);
                destroy_screen(&mut commands, &title_query);
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
            // Handle text input for lobby name
            for key in input.get_just_pressed() {
                if let Ok(mut text) = lobby_name_query.get_single_mut() {
                    match key {
                        KeyCode::Backspace => {
                            text.0.pop();
                        }
                        KeyCode::Space => {
                            text.0.push(' ');
                        }
                        _ => {
                            if let Some(character) = key_to_char(key) {
                                if text.0.len() < 20 {
                                    text.0.push(character);
                                }
                            }
                        }
                    }
                }
            }

            if input.just_pressed(KeyCode::Escape){
                next_state.set(GameState::Title);
                destroy_screen(&mut commands, &join_query);
                setup_title_screen(commands, asset_server);
            }
            else if input.just_pressed(KeyCode::Enter){
                // Get the lobby name from the input
                let lobby_name = if let Ok(text) = lobby_name_query.get_single() {
                    text.0.trim().to_string()
                } else {
                    String::new()
                };

                if lobby_name.is_empty() {
                    println!("Please enter a lobby name!");
                    return;
                }

                // Connect to server if not already connected
                let server_addr = format!("{}:4000", server_address.address);
                if network_client.client.is_none() {
                    match connect_to_server(&mut network_client, &message_sender, &server_addr) {
                        Ok(_) => println!("Connected to server!"),
                        Err(e) => {
                            println!("Failed to connect to server: {}", e);
                            return;
                        }
                    }
                }

                // Send join lobby message
                if let Some(client) = &mut network_client.client {
                    if let Err(e) = client.join_lobby(lobby_name.clone()) {
                        println!("Failed to join lobby: {}", e);
                        return;
                    }
                }

                // Transition to lobby screen
                next_state.set(GameState::Lobby);
                destroy_screen(&mut commands, &join_query);

                lobby_state.connected_players.clear();
                lobby_state.connected_players.push("Connecting...".to_string());
                lobby_state.name = lobby_name;

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

    // Server IP input (top-right)
    commands.spawn((
        Text2d::new("Server IP:"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(290., 300., 1.),
            ..default()
        },
        TextFont {
            font_size: 25.0,
            ..default()
        },
        MainScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/lobbyInput.png")),
        Transform {
            translation: Vec3::new(450., 300., 1.),
            scale: Vec3::new(0.6, 0.6, 1.0),
            ..default()
        },
        MainScreenEntity
    ));
    commands.spawn((
        Text2d::new("hi"),
        TextColor(Color::srgb(0.5, 0.5, 0.5)),  // Gray placeholder color
        Transform {
            translation: Vec3::new(450., 300., 1.),
            ..default()
        },
        TextFont {
            font_size: 25.0,
            ..default()
        },
        MainScreenEntity,
        ServerIpInput,
    ));

    // Theta* DEMO (Remove later)
    commands.spawn((
        Sprite::from_image(asset_server.load("temp-art/theta-demo.png")),
        Transform {
            translation: Vec3::new(400., -200., 1.),
            ..default()
        },
        MainScreenEntity
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/key4.png")),
        Transform {
            translation: Vec3::new(400., -300., 1.),
            ..default()
        },
        MainScreenEntity
    ));
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
        Text2d::new("Input Lobby Name:"),
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
        Text2d::new(""),
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
        LobbyNameInput,
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

// Helper function to convert KeyCode to character
fn key_to_char(key: &KeyCode) -> Option<char> {
    match key {
        KeyCode::KeyA => Some('A'),
        KeyCode::KeyB => Some('B'),
        KeyCode::KeyC => Some('C'),
        KeyCode::KeyD => Some('D'),
        KeyCode::KeyE => Some('E'),
        KeyCode::KeyF => Some('F'),
        KeyCode::KeyG => Some('G'),
        KeyCode::KeyH => Some('H'),
        KeyCode::KeyI => Some('I'),
        KeyCode::KeyJ => Some('J'),
        KeyCode::KeyK => Some('K'),
        KeyCode::KeyL => Some('L'),
        KeyCode::KeyM => Some('M'),
        KeyCode::KeyN => Some('N'),
        KeyCode::KeyO => Some('O'),
        KeyCode::KeyP => Some('P'),
        KeyCode::KeyQ => Some('Q'),
        KeyCode::KeyR => Some('R'),
        KeyCode::KeyS => Some('S'),
        KeyCode::KeyT => Some('T'),
        KeyCode::KeyU => Some('U'),
        KeyCode::KeyV => Some('V'),
        KeyCode::KeyW => Some('W'),
        KeyCode::KeyX => Some('X'),
        KeyCode::KeyY => Some('Y'),
        KeyCode::KeyZ => Some('Z'),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        _ => None,
    }
}