use bevy::prelude::*;
use crate::car::{Car, PlayerControlled, CAR_SIZE};
use crate::map::GameMap;
use crate::TILE_SIZE;

// Check for car collisions
pub fn check_car_collisions(
    new_position: Vec3,
    other_cars: &Query<&Transform, (With<Car>, Without<PlayerControlled>)>,
) -> bool {
    let collision_radius = CAR_SIZE as f32;

    for other_car_transform in other_cars.iter() {
        let distance = new_position.truncate().distance(other_car_transform.translation.truncate());
        if distance < collision_radius {
            return true;
        }
    }

    false
}

// Check for wall collisions
pub fn check_tile_collision(
    position: Vec3,
    game_map: &GameMap,
) -> bool {
    let tile = game_map.get_tile(position.x, position.y, TILE_SIZE as f32);
    !tile.passable
}
