use bevy::input::ButtonInput;
use bevy::prelude::*;
use crate::GameState;

use bevy::{color::palettes::basic::*, input_focus::InputFocus, prelude::*};

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
) {

    match *current_state.get() {
        GameState::Title => {
            if input.just_pressed(KeyCode::Digit1){
                next_state.set(GameState::Lobby);
                destroy_screen(&mut commands, &title_query);
                setup_lobby(commands, asset_server);
            }
            else if input.just_pressed(KeyCode::Digit2){
                next_state.set(GameState::Joining);
                destroy_screen(&mut commands, &title_query);
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
            else if input.just_pressed(KeyCode::Digit1){
                next_state.set(GameState::Playing);
                destroy_screen(&mut commands, &join_query);
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
            translation: Vec3::new(300., -200., 1.),
            ..default()
        },
        MainScreenEntity
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/keys/key4.png")),
        Transform {
            translation: Vec3::new(300., -300., 1.),
            ..default()
        },
        MainScreenEntity
    ));
}

fn setup_lobby(
    mut commands: Commands,
    asset_server: Res<AssetServer>){
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
        Text2d::new("Code: ????"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(500., 300., 1.),
            ..default()
        },
        LobbyScreenEntity,
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

    // Player Icons & Nameplates
    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/namePlate.png")),
        Transform {
            translation: Vec3::new(25., 150., 1.),
            ..default()
        },
        LobbyScreenEntity
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("player-icons/human1.png")),
        Transform {
            translation: Vec3::new(-225., 150., 1.),
            ..default()
        },
        LobbyScreenEntity
    ));
    commands.spawn((
        Text2d::new("6ix7even"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., 150., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        LobbyScreenEntity,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/namePlate.png")),
        Transform {
            translation: Vec3::new(25., 50., 1.),
            ..default()
        },
        LobbyScreenEntity
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("player-icons/human2.png")),
        Transform {
            translation: Vec3::new(-225., 50., 1.),
            ..default()
        },
        LobbyScreenEntity
    ));
    commands.spawn((
        Text2d::new("L.Griffin"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., 50., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        LobbyScreenEntity,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/namePlate.png")),
        Transform {
            translation: Vec3::new(25., -50., 1.),
            ..default()
        },
        LobbyScreenEntity
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("player-icons/human3.png")),
        Transform {
            translation: Vec3::new(-225., -50., 1.),
            ..default()
        },
        LobbyScreenEntity
    ));
    commands.spawn((
        Text2d::new("JohnPork"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., -50., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        LobbyScreenEntity,
    ));

    commands.spawn((
        Sprite::from_image(asset_server.load("title_screen/namePlate.png")),
        Transform {
            translation: Vec3::new(25., -150., 1.),
            ..default()
        },
        LobbyScreenEntity
    ));
    commands.spawn((
        Sprite::from_image(asset_server.load("player-icons/human4.png")),
        Transform {
            translation: Vec3::new(-225., -150., 1.),
            ..default()
        },
        LobbyScreenEntity
    ));
    commands.spawn((
        Text2d::new("N.Farnan"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., -150., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        LobbyScreenEntity,
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
        Sprite::from_image(asset_server.load("title_screen/lobbyInput.png")),
        Transform {
            translation: Vec3::new(0., 0., 1.),
            ..default()
        },
        JoinScreenEntity
    ));
    commands.spawn((
        Text2d::new("Input Code"),
        TextColor(Color::BLACK),
        Transform {
            translation: Vec3::new(0., 0., 1.),
            ..default()
        },
        TextFont {
            font_size: 50.0,
            ..default()
        },
        JoinScreenEntity,
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
        Sprite::from_image(asset_server.load("title_screen/keys/key1.png")),
        Transform {
            translation: Vec3::new(-250., -300., 1.),
            ..default()
        },
        JoinScreenEntity
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