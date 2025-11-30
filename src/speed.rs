use bevy::prelude::*;
use rand::{Rng, rngs::ThreadRng};
use crate::game_logic::{GameMap, Car, PlayerControlled, AIControlled, Orientation, Velocity, TILE_SIZE};


// Component for the powerup
#[derive(Component)]
pub struct SpeedPowerup;


#[derive(Component)]
pub struct SpeedBoost {
    pub timer: Timer,
}

// System to spawn powerups on road tiles
pub fn spawn_speed_powerups(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_map: Res<GameMap>,
    powerups: Query<Entity, With<SpeedPowerup>>
) {

    let current_count = powerups.iter().count();
    let target_count = 10;

    if current_count < target_count {
        let to_spawn = target_count - current_count;
        spawn_powerups(&mut commands, &asset_server, &game_map, to_spawn);
    }
}

fn spawn_powerups(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    game_map: &Res<GameMap>,
    count: usize,
) {
    
    let mut rng: ThreadRng = rand::rng();
    
    // Collect all road tile positions
    let mut road_tiles = Vec::new();

    // Iterate through gameMap to find road tiles
    for row in game_map.terrain_layer.iter() {
        for tile in row.iter() {
            if tile.tile_id <= 15 {
                road_tiles.push((tile.x_coordinate, tile.y_coordinate));
            }
        }
    }
    
    // Spawn powerups at random road positions
    for _ in 0..count {
        if let Some(&(x, y)) = road_tiles.get(rng.random_range(0..road_tiles.len())) {
            // Convert tile coordinates to world coordinates
            let world_pos = game_map.tile_to_world(x as f32, y as f32, TILE_SIZE as f32);
            commands.spawn((
                Sprite::from_image(asset_server.load("GasCanPowerUp.png")),
                Transform::from_xyz(world_pos.x, world_pos.y, 15.0),
                SpeedPowerup,
            ));
        }
    }
}

// System to handle powerup collection
pub fn collect_powerups(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform), With<PlayerControlled>>,
    powerup_query: Query<(Entity, &Transform), With<SpeedPowerup>>,
) {
    const PICKUP_DISTANCE: f32 = 64.0;
    
    if let Ok((player_entity, player_transform)) = player_query.single() {
        let player_pos = player_transform.translation.truncate();
        
        for (powerup_entity, powerup_transform) in powerup_query.iter() {
            let powerup_pos = powerup_transform.translation.truncate();
            let distance = player_pos.distance(powerup_pos);
            
            if distance < PICKUP_DISTANCE {
                // Despawn the powerup
                commands.entity(powerup_entity).despawn();
                
                // Add speed boost component to player
                commands.entity(player_entity).insert(SpeedBoost {
                    timer: Timer::from_seconds(5.0, TimerMode::Once), // 5 second boost
                });
                
                println!("Powerup collected! Speed boost activated!");
            }
        }
    }
}

// System to handle boost expiration
pub fn update_speed_boost(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SpeedBoost, &mut Sprite), With<PlayerControlled>>,
) {
    for (entity, mut boost, mut sprite) in query.iter_mut() {
        boost.timer.tick(time.delta());
        
        if boost.timer.finished() {
            // Remove boost and reset color
            sprite.color = Color::WHITE;
            commands.entity(entity).remove::<SpeedBoost>();
            println!("Speed boost expired!");
        }
    }
}