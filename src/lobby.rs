use bevy::prelude::*;
use crate::GameState;

#[derive(Component)]
pub struct LobbyScreenEntity;

#[derive(Component)]
pub struct PlayerSlot {
    pub slot_index: usize,
}

#[derive(Component)]
pub struct PlayerNameText {
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
            PlayerNameText { slot_index: i },
        ));
    }
}

// System to update lobby UI when LobbyState changes
pub fn update_lobby_display(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    lobby_state: Res<LobbyState>,
    mut name_text_query: Query<(&mut Text2d, &PlayerNameText)>,
    slot_query: Query<(Entity, &PlayerSlot), With<LobbyScreenEntity>>,
) {
    // Only run when LobbyState changes
    if !lobby_state.is_changed() {
        return;
    }

    let player_icons = [
        "player-icons/human1.png",
        "player-icons/human2.png",
        "player-icons/human3.png",
        "player-icons/human4.png",
    ];

    // Update existing player name texts
    for (mut text, player_name_text) in name_text_query.iter_mut() {
        if let Some(player_name) = lobby_state.connected_players.get(player_name_text.slot_index) {
            text.0 = player_name.clone();
        }
    }

    // Count how many slots currently exist
    let existing_slots = slot_query.iter().map(|(_, slot)| slot.slot_index).max().map(|i| i + 1).unwrap_or(0);
    let needed_slots = lobby_state.connected_players.len();

    // Spawn new slots if need more
    if needed_slots > existing_slots {
        for i in existing_slots..needed_slots.min(4) {
            let y_pos = 150. - (i as f32 * 100.);
            let player_name = &lobby_state.connected_players[i];

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
                PlayerNameText { slot_index: i },
            ));
        }
    }
}