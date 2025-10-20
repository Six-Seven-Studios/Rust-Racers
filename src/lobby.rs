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
    pub name: String,
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
        Text2d::new(format!("Name: {}", lobby_state.name)),
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