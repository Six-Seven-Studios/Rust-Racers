use crate::game_logic::{CAR_SIZE, GameMap, TILE_SIZE};
use bevy::prelude::*;

// Generic collision handler that uses any iterator of (position, velocity) pairs
// Returns true if position should still be updated, false to indicate wall
pub fn handle_collision<'a, I>(
    new_position: Vec3,
    current_position: Vec2,
    velocity: &mut Vec2,
    game_map: &GameMap,
    other_cars: I,
) -> bool
where
    I: IntoIterator<Item = (Vec2, Vec2)>,
{
    let new_pos_2d = new_position.truncate();

    // Check car-to-car collisions
    for (other_position, other_velocity) in other_cars {
        // Skip self (positions are very close)
        if (other_position.x - current_position.x).abs() < 0.01
            && (other_position.y - current_position.y).abs() < 0.01
        {
            continue;
        }

        let distance = new_pos_2d.distance(other_position);
        if distance < CAR_SIZE as f32 {
            let bounce_direction = (current_position - other_position).normalize_or_zero();
            let relative_speed = (*velocity - other_velocity).dot(bounce_direction);

            if relative_speed < 0.0 {
                *velocity += bounce_direction * relative_speed * -1.5; // Bounce strength
            }
            return true; // Collision occurred, but allow position update
        }
    }

    // Check wall collisions
    let tile = game_map.get_tile(new_pos_2d.x, new_pos_2d.y, TILE_SIZE as f32);
    if !tile.passable {
        *velocity *= -0.3;
        return false; // Wall collision, block position update
    }

    true // No collision, move normally
}
