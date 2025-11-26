use crate::GameState;
use crate::drift_settings::DriftSettings;
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

use crate::lobby::{LobbyState, setup_lobby};
use crate::networking_plugin::{NetworkClient, MessageSender, connect_to_server};
use crate::game_logic::CpuDifficulty;

#[derive(Component)]
pub struct MainScreenEntity;

#[derive(Component)]
pub struct JoinScreenEntity;

#[derive(Component)]
pub struct CreateScreenEntity;

#[derive(Component)]
pub struct SettingsScreenEntity;

#[derive(Component)]
pub struct EasyDriftLabel;

#[derive(Component)]
pub struct CustomizingScreenEntity;

#[derive(Component)]
pub struct TitleScreenAudio;

#[derive(Component)]
pub struct LobbyNameInput;

#[derive(Component)]
pub struct ServerIpInput;

#[derive(Component)]
pub struct CpuDifficultyText;

#[derive(Component)]
pub struct LobbyListContainer;

#[derive(Component)]
pub struct LobbyRow;

#[derive(Component)]
pub struct JoinButton {
    pub lobby_name: String,
}

#[derive(Resource)]
pub struct ServerAddress {
    pub address: String,
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

use bevy::ecs::system::SystemParam;

// bundling all your screen queries into one struct to save space
#[derive(SystemParam)]
pub struct MenuScreens<'w, 's> {
    pub main: Query<'w, 's, Entity, With<MainScreenEntity>>,
    pub lobby: Query<'w, 's, Entity, With<crate::lobby::LobbyScreenEntity>>,
    pub join: Query<'w, 's, Entity, With<JoinScreenEntity>>,
    pub settings: Query<'w, 's, Entity, With<SettingsScreenEntity>>,
    pub customize: Query<'w, 's, Entity, With<CustomizingScreenEntity>>,
}

pub fn check_for_title_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // added a menu screen to trim down this function
    screens: MenuScreens, 
    mut network_client: ResMut<NetworkClient>,
    message_sender: Res<MessageSender>,
    // Without<CpuDifficultyText> here
    mut lobby_name_query: Query<&mut Text2d, (With<LobbyNameInput>, Without<ServerIpInput>, Without<CpuDifficultyText>)>,
    // Without<CpuDifficultyText> here
    mut server_ip_query: Query<&mut Text2d, (With<ServerIpInput>, Without<LobbyNameInput>, Without<CpuDifficultyText>)>,
    server_address: Res<ServerAddress>,     
    mut cpu_difficulty: ResMut<CpuDifficulty>,
    // Without<LobbyNameInput> and Without<ServerIpInput> here
    mut difficulty_text_query: Query<&mut Text2d, (With<CpuDifficultyText>, Without<LobbyNameInput>, Without<ServerIpInput>)>
) {
    match *current_state.get() {
        GameState::Title => {
            // Use a local variable to track typing mode that persists across frames
            use bevy::prelude::Local;
            static mut TYPING_MODE: bool = false;

            // Toggle typing mode with Tab
            if input.just_pressed(KeyCode::Tab) {
                unsafe {
                    TYPING_MODE = !TYPING_MODE;
                    println!(
                        "IP typing mode: {}",
                        if TYPING_MODE {
                            "ON"
                        } else {
                            "OFF (use 1/2/3/4)"
                        }
                    );
                }
            }

            let is_typing_ip = unsafe { TYPING_MODE };

            // Only handle text input when in typing mode
            if is_typing_ip {
                for key in input.get_just_pressed() {
                    if let Ok(mut text) = server_ip_query.get_single_mut() {
                        match key {
                            KeyCode::Tab => {
                                // Tab handled above for toggle, skip here
                            }
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
            }

            // Only trigger menu actions if NOT typing in IP field
            if !is_typing_ip && input.just_pressed(KeyCode::Digit1) {
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

                // Transition to creating lobby
                next_state.set(GameState::Creating);
                destroy_screen(&mut commands, &screens.main);

                setup_create_lobby(commands, asset_server);
            } else if !is_typing_ip && input.just_pressed(KeyCode::Digit2) {
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

                // Send list lobby message
                if let Some(client) = &mut network_client.client {
                    if let Err(e) = client.list_lobbies() {
                        println!("Failed to list lobbies: {}", e);
                        return;
                    }
                }

                next_state.set(GameState::Joining);
                destroy_screen(&mut commands, &screens.main);

                setup_join_lobby(commands, asset_server);
            } else if !is_typing_ip && input.just_pressed(KeyCode::Digit3) {
                next_state.set(GameState::Customizing);
                destroy_screen(&mut commands, &screens.main);
                setup_customizing(commands, asset_server);
            } else if input.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Settings);
                destroy_screen(&mut commands, &screens.main);
                setup_settings(commands, asset_server, *cpu_difficulty);
            }
            // Theta* DEMO
            else if !is_typing_ip && input.just_pressed(KeyCode::Digit4) {
                next_state.set(GameState::PlayingDemo);
                destroy_screen(&mut commands, &screens.main);
            }
        }
        GameState::Customizing => {
            if input.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Title);
                destroy_screen(&mut commands, &screens.customize);
                setup_title_screen(commands, asset_server, server_address);
            }
        }
        GameState::Settings => {
            if input.just_pressed(KeyCode::KeyE) {
                drift_settings.toggle();
            } else if input.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Title);
                destroy_screen(&mut commands, &screens.settings);
                setup_title_screen(commands, asset_server, server_address);
            } else if input.just_pressed(KeyCode::ArrowLeft) || input.just_pressed(KeyCode::KeyA) {
                // cycle difficulty
                *cpu_difficulty = cpu_difficulty.prev();
                if let Ok(mut text) = difficulty_text_query.single_mut() {
                    text.0 = format!("CPU Difficulty: {}", cpu_difficulty.as_str());
                }
            } else if input.just_pressed(KeyCode::ArrowRight) || input.just_pressed(KeyCode::KeyD) {
                // cycle difficulty up
                *cpu_difficulty = cpu_difficulty.next();
                if let Ok(mut text) = difficulty_text_query.single_mut() {
                    text.0 = format!("CPU Difficulty: {}", cpu_difficulty.as_str());
                }
            }
        }
        _ => {
            return;
        }
    }
}

pub fn check_for_lobby_input(
    input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    lobby_query: Query<Entity, With<crate::lobby::LobbyScreenEntity>>,
    create_query: Query<Entity, With<CreateScreenEntity>>,
    join_query: Query<Entity, With<JoinScreenEntity>>,
    mut lobby_state: ResMut<LobbyState>,
    mut network_client: ResMut<NetworkClient>,
    message_sender: Res<MessageSender>,
    mut lobby_name_query: Query<&mut Text2d, (With<LobbyNameInput>, Without<ServerIpInput>)>,
    mut server_ip_query: Query<&mut Text2d, (With<ServerIpInput>, Without<LobbyNameInput>)>,
    server_address: Res<ServerAddress>,
    mut buttons: Query<(&Interaction, &JoinButton), (Changed<Interaction>, With<Button>)>,
) {
    match *current_state.get() {
        GameState::Lobby => {
            if input.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Title);

                let lobby_name = lobby_state.name.clone();

                if let Some(client) = &mut network_client.client {
                    if let Err(e) = client.leave_lobby(lobby_name.clone()) {
                        println!("Failed to leave lobby: {}", e);
                        return;
                    }
                }

                destroy_screen(&mut commands, &lobby_query);
                setup_title_screen(commands, asset_server, server_address);
            } else if input.just_pressed(KeyCode::Digit1) {
                // Send start lobby message to server
                if let Some(client) = network_client.client.as_mut() {
                    let lobby_name = lobby_state.name.clone();
                    if let Err(e) = client.start_lobby(lobby_name) {
                        println!("Failed to send start lobby message: {}", e);
                    } else {
                        println!("Sent start lobby request to server");
                    }
                }
            }
        }
        GameState::Creating => {
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

            if input.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Title);
                destroy_screen(&mut commands, &create_query);
                setup_title_screen(commands, asset_server, server_address);
            } else if input.just_pressed(KeyCode::Enter) {
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
                    if let Err(e) = client.create_lobby(lobby_name.clone()) {
                        println!("Failed to create lobby: {}", e);
                        return;
                    }
                }

                // Transition to lobby screen
                next_state.set(GameState::Lobby);
                destroy_screen(&mut commands, &create_query);

                lobby_state.connected_players.clear();
                if let Some(player_id) = &mut network_client.player_id {
                    lobby_state
                        .connected_players
                        .push(format!("Player {} (You)", player_id));
                } else {
                    lobby_state
                        .connected_players
                        .push("Connecting...".to_string());
                }
                lobby_state.name = lobby_name;

                setup_lobby(&mut commands, asset_server.clone(), &lobby_state);
            }
        }
        GameState::Joining => {
            if input.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Title);
                destroy_screen(&mut commands, &join_query);
                setup_title_screen(commands, asset_server, server_address);
                return;
            }

            for (interaction, join_btn) in buttons {
                if *interaction == Interaction::Pressed {
                    // Connect to server if not already connected
                    let server_addr = format!("{}:4000", server_address.address);
                    if network_client.client.is_none() {
                        match connect_to_server(&mut network_client, &message_sender, &server_addr)
                        {
                            Ok(_) => println!("Connected to server!"),
                            Err(e) => {
                                println!("Failed to connect to server: {}", e);
                                return;
                            }
                        }
                    }

                    // Send join lobby message
                    if let Some(client) = &mut network_client.client {
                        if let Err(e) = client.join_lobby(join_btn.lobby_name.clone()) {
                            println!("Failed to join lobby: {}", e);
                            return;
                        }
                    }

                    // Transition to lobby screen
                    next_state.set(GameState::Lobby);
                    destroy_screen(&mut commands, &join_query);
                    lobby_state.connected_players.clear();
                    lobby_state
                        .connected_players
                        .push("Connecting...".to_string());
                    lobby_state.name = join_btn.lobby_name.clone();
                    setup_lobby(&mut commands, asset_server.clone(), &lobby_state);
                }
            }
        }
        _ => {
            return;
        }
    }
}

// To pause audio

pub fn pause(keyboard_input: Res<ButtonInput<KeyCode>>, music_controller: Query<&AudioSink>) {
    let Ok(sink) = music_controller.single() else {
        return;
    };

    if keyboard_input.just_pressed(KeyCode::KeyL) {
        sink.toggle_playback();
    }
}

pub fn setup_title_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    server_address: Res<ServerAddress>,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load("title_screen/RustRacersTitleScreenAudio.ogg")),
        PlaybackSettings::LOOP,
        TitleScreenAudio,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/settingsGear.png")),
        Transform {
            translation: Vec3::new(-570., 300., 1.),
            ..default()
        },
        MainScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/keyEsc.png")),
        Transform {
            translation: Vec3::new(-570., 220., 1.),
            ..default()
        },
        MainScreenEntity,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/slantedButton.png")),
        Transform {
            translation: Vec3::new(0., -100., 1.),
            ..default()
        },
        MainScreenEntity,
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
        MainScreenEntity,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/slantedButton.png")),
        Transform {
            translation: Vec3::new(0., -200., 1.),
            ..default()
        },
        MainScreenEntity,
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
        MainScreenEntity,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/slantedButton.png")),
        Transform {
            translation: Vec3::new(0., -300., 1.),
            ..default()
        },
        MainScreenEntity,
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
        MainScreenEntity,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/rustRacersLogo.png")),
        Transform {
            translation: Vec3::new(0., 100., 1.),
            ..default()
        },
        MainScreenEntity,
    ));

    // Server IP input (top-right)
    commands.spawn((
        Text2d::new("Server IP:"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(220., 300., 1.),
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
            translation: Vec3::new(500., 300., 1.),
            scale: Vec3::new(0.6, 0.6, 1.0),
            ..default()
        },
        MainScreenEntity,
    ));
    commands.spawn((
        Text2d::new(server_address.address.clone()),
        TextColor(Color::srgb(0.5, 0.5, 0.5)), // Gray placeholder color
        Transform {
            translation: Vec3::new(500., 300., 1.),
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
        MainScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/key4.png")),
        Transform {
            translation: Vec3::new(400., -300., 1.),
            ..default()
        },
        MainScreenEntity,
    ));
}

fn setup_create_lobby(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/backArrow.png")),
        Transform {
            translation: Vec3::new(-570., 300., 1.),
            ..default()
        },
        CreateScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/keyEsc.png")),
        Transform {
            translation: Vec3::new(-570., 220., 1.),
            ..default()
        },
        CreateScreenEntity,
    ));
    commands.spawn((
        Text2d::new("Create A Lobby"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., 300., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        CreateScreenEntity,
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
        CreateScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/lobbyInput.png")),
        Transform {
            translation: Vec3::new(0., 0., 1.),
            ..default()
        },
        CreateScreenEntity,
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
        CreateScreenEntity,
        LobbyNameInput,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/slantedButton.png")),
        Transform {
            translation: Vec3::new(0., -300., 1.),
            ..default()
        },
        CreateScreenEntity,
    ));
    commands.spawn((
        Text2d::new("CREATE!"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., -300., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        CreateScreenEntity,
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
        CreateScreenEntity,
    ));
}

fn setup_join_lobby(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Root full-screen UI node
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(245, 245, 245)),
            JoinScreenEntity,
        ))
        .with_children(|root| {
            // Panel
            root.spawn((
                Node {
                    width: Val::Px(800.0),
                    height: Val::Px(500.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(12.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                BorderColor(Color::BLACK),
                BorderRadius::all(Val::Px(12.0)),
                JoinScreenEntity,
            ))
            .with_children(|panel| {
                // Title
                panel.spawn((
                    Text::new("Join a Lobby"),
                    TextFont {
                        font_size: 40.0,
                        ..default()
                    },
                    TextColor(Color::BLACK),
                    JoinScreenEntity,
                ));

                // Column headers
                panel
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(32.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        JoinScreenEntity,
                    ))
                    .with_children(|hdr| {
                        hdr.spawn((
                            Text::new("Lobby"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::BLACK),
                            JoinScreenEntity,
                        ));
                        hdr.spawn((
                            Text::new("Players"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::BLACK),
                            JoinScreenEntity,
                        ));
                        hdr.spawn((
                            Text::new(""),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::BLACK),
                            JoinScreenEntity,
                        ));
                    });

                // Scroll container
                panel
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            overflow: Overflow::clip_y(), // scrollable vertically
                            ..default()
                        },
                        BackgroundColor(Color::srgb_u8(252, 252, 252)),
                        BorderColor(Color::srgb_u8(220, 220, 220)),
                        BorderRadius::all(Val::Px(8.0)),
                        LobbyListContainer,
                        JoinScreenEntity,
                    ))
                    .with_children(|_rows| {
                        // rows will be populated later
                    });

                // Footer hint
                panel.spawn((
                    Text::new("Press ESC to go back"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb_u8(90, 90, 90)),
                    JoinScreenEntity,
                ));
            });
        });
}

fn setup_settings(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cpu_difficulty: CpuDifficulty,
){
    // Title text
    commands.spawn((
        Text2d::new("Settings"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., 80., 1.),
            ..default()
        },
        SettingsScreenEntity,
    ));

    // CPU difficulty display
    commands.spawn((
        Text2d::new(format!("CPU Difficulty: {}", cpu_difficulty.as_str())),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., 20., 1.),
            ..default()
        },
        TextFont {
            font_size: 40.0,
            ..default()
        },
        SettingsScreenEntity,
        CpuDifficultyText,
    ));

    // Hint text
    commands.spawn((
        Text2d::new("Use A/D or Left/Right to change"),
        TextColor(Color::srgb_u8(120, 120, 120)),
        Transform {
            translation: Vec3::new(0., -30., 1.),
            ..default()
        },
        TextFont {
            font_size: 24.0,
            ..default()
        },
        SettingsScreenEntity,
    ));

    // Back arrow and ESC key legend
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
        SettingsScreenEntity,
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
        CustomizingScreenEntity,
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
        CustomizingScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/keyD.png")),
        Transform {
            translation: Vec3::new(150., 0., 1.),
            ..default()
        },
        CustomizingScreenEntity,
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("car.png")),
        Transform {
            translation: Vec3::new(0., 0., 1.),
            ..default()
        },
        CustomizingScreenEntity,
    ));
}

pub fn update_easy_drift_label(
    drift_settings: Res<DriftSettings>,
    mut label_query: Query<&mut Text2d, With<EasyDriftLabel>>,
) {
    if !drift_settings.is_changed() {
        return;
    }

    if let Ok(mut text) = label_query.get_single_mut() {
        text.0 = format!("Easy Drift Mode: {}", drift_settings.mode_label());
    }
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
