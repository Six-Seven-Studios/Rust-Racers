use bevy::prelude::*;
use crate::car::{Car, PlayerControlled, CAR_LENGTH};
use crate::map::GameMap;
use crate::TILE_SIZE;

/// Check for collisions between a position and other entities using simple distance check
/// Returns true if there would be a collision, false otherwise
pub fn check_car_collisions(
    new_position: Vec3,
    other_cars: &Query<&Transform, (With<Car>, Without<PlayerControlled>)>,
) -> bool {
    // Simple circular collision detection using distance
    // Use CAR_LENGTH as the collision radius
    let collision_radius = CAR_LENGTH as f32;

    for other_car_transform in other_cars.iter() {
        let distance = new_position.truncate().distance(other_car_transform.translation.truncate());
        if distance < collision_radius {
            return true;
        }
    }

    false
}

/// Check for collisions with impassable terrain tiles (walls)
/// Returns true if the tile at the position is not passable
pub fn check_tile_collision(
    position: Vec3,
    game_map: &GameMap,
) -> bool {
    let tile = game_map.get_tile(position.x, position.y, TILE_SIZE as f32);
    !tile.passable
}