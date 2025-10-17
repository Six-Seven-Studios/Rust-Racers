use bevy::prelude::*;
use bevy::ecs::query::QueryFilter;
use crate::car::{CAR_SIZE, Velocity};
use crate::map::GameMap;
use crate::TILE_SIZE;

// Collision handler - does everything automatically
pub fn handle_collision<F: QueryFilter>(
    new_position: Vec3,
    current_position: Vec2,
    velocity: &mut Vec2,
    game_map: &GameMap,
    other_cars: &Query<(&Transform, &Velocity), F>,
) -> bool {
    // Check car-to-car collisions
    for (other_transform, other_velocity) in other_cars.iter() {
        let distance = new_position.truncate().distance(other_transform.translation.truncate());
        if distance < CAR_SIZE as f32 {
            let bounce_direction = (current_position - other_transform.translation.truncate()).normalize_or_zero();
            let relative_speed = (*velocity - other_velocity.velocity).dot(bounce_direction);
            
            if relative_speed < 0.0 {
                *velocity += bounce_direction * relative_speed * -1.5; // Bounce strength
            }
            return true;
        }
    }
    // Check wall collisions
    let tile = game_map.get_tile(new_position.x, new_position.y, TILE_SIZE as f32);
    if !tile.passable {
        *velocity *= -0.3;
        return false;
    }
    true // No collision, move normally
}
