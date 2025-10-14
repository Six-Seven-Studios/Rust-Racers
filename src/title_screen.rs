use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use crate::GameState;

use bevy::{color::palettes::basic::*, input_focus::InputFocus, prelude::*};
use crate::get_ip::get_local_ip;
use crate::lobby::{LobbyState, setup_lobby};

#[derive(Component)]
pub struct MainScreenEntity;

#[derive(Component)]
pub struct JoinScreenEntity;

#[derive(Component)]
pub struct SettingsScreenEntity;

#[derive(Component)]
pub struct CustomizingScreenEntity;

#[derive(Component)]
pub struct IpInputText;

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
    lobby_query: Query<Entity, With<crate::lobby::LobbyScreenEntity>>,
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

                lobby_state.connected_players.clear();
                lobby_state.connected_players.push("Player 1 (You)".to_string());
                if let Ok(ip) = get_local_ip() {
                    lobby_state.server_ip = ip;
                } else {
                    lobby_state.server_ip = "0.0.0.0".to_string();
                }

                network_client.player_id = Some(0);

                setup_lobby(commands, asset_server, &lobby_state);
            }
            else if input.just_pressed(KeyCode::Digit2){
                next_state.set(GameState::Joining);
                destroy_screen(&mut commands, &title_query);

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
            // Theta* DEMO
            else if input.just_pressed(KeyCode::Digit4){
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
            if input.just_pressed(KeyCode::Escape){
                next_state.set(GameState::Title);
                destroy_screen(&mut commands, &join_query);
                setup_title_screen(commands, asset_server);
            }
            else if input.just_pressed(KeyCode::Enter){
                network_client.target_ip = Some(ip_input_state.input.clone());
                network_client.connection_attempted = false;

                next_state.set(GameState::Lobby);
                destroy_screen(&mut commands, &join_query);

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

pub fn handle_ip_input(
    input: Res<ButtonInput<KeyCode>>,
    mut ip_input_state: ResMut<IpInputState>,
    mut text_query: Query<&mut Text2d, With<IpInputText>>,
    current_state: Res<State<GameState>>,
) {
    if *current_state.get() != GameState::Joining {
        return;
    }

    if input.just_pressed(KeyCode::Enter) || input.just_pressed(KeyCode::Escape) {
        return;
    }

    let mut changed = false;

    if input.just_pressed(KeyCode::Backspace) {
        ip_input_state.input.pop();
        changed = true;
    }

    if input.just_pressed(KeyCode::Period) && ip_input_state.input.len() < 15 {
        ip_input_state.input.push('.');
        changed = true;
    }

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
