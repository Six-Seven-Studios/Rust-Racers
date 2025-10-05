use bevy::prelude::*;
use crate::map::GameMap;
use crate::car::PlayerControlled;

// Camera-related constants
pub const WIN_W: f32 = 1280.;
pub const WIN_H: f32 = 720.;

// Camera movement system that follows the player
pub fn move_camera(
    game_map: Res<GameMap>,
    player_car: Single<&Transform, With<PlayerControlled>>,
    mut camera: Single<&mut Transform, (With<Camera>, Without<PlayerControlled>)>,
) {
    let max = Vec3::new(game_map.width / 2. - WIN_W / 2., game_map.height / 2. - WIN_H / 2., 0.);
    let min = -max.clone();

    // clamp to map bounds
    let mut target = player_car.translation.clamp(min, max);

    // round to integers to prevent subpixel gaps
    target.x = target.x.round();
    target.y = target.y.round();

    camera.translation = target;
}

// Reset camera position for credits screen
pub fn reset_camera_for_credits(mut camera: Single<&mut Transform, With<Camera>>) {
    camera.translation = Vec3::ZERO;
}
