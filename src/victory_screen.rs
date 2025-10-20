use bevy::prelude::*;

pub fn setup_victory_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    if let Ok(mut camera) = camera_query.get_single_mut() {
        camera.translation = Vec3::ZERO;
    }

    commands.spawn((
        Sprite::from_image(asset_server.load("victory-screen/victory_screen.png")),
        Transform {
            translation: Vec3::new(0., 0., 100.),
            ..default()
        },
    ));

    commands.spawn(AudioPlayer::new(asset_server.load("victory-screen/67.mp3")));
}
