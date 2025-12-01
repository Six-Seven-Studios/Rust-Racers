use crate::game_logic::map::GameMap;
use crate::game_logic::terrain::TerrainTile;
use bevy::prelude::Resource;

/// Grid node for Theta* pathfinding
#[derive(Debug, Clone)]
pub struct GridNode {
    pub x: usize,
    pub y: usize,
    pub world_x: f32,
    pub world_y: f32,
    pub passable: bool,
    pub cost: f32,
}

#[derive(Resource)]
pub struct ThetaGrid {
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    nodes: Vec<Vec<GridNode>>,
}

impl ThetaGrid {
    pub fn create_theta_grid(game_map: &GameMap, tile_size: f32) -> Self {
        let height = game_map.terrain_layer.len();
        let width = if height > 0 { game_map.terrain_layer[0].len() } else { 0 };

        let mut nodes = Vec::with_capacity(height);

        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let terrain = &game_map.terrain_layer[y][x];

                // Convert grid coordinates to world coordinates
                let world_x = (x as f32 * tile_size) - (game_map.width / 2.0) + (tile_size / 2.0);
                let world_y = -((y as f32 * tile_size) - (game_map.height / 2.0) + (tile_size / 2.0));

                // Calculate movement cost from terrain modifiers
                let cost = Self::calculate_node_cost(&terrain);

                row.push(GridNode {
                    x,
                    y,
                    world_x,
                    world_y,
                    passable: terrain.passable,
                    cost,
                });
            }
            nodes.push(row);
        }

        ThetaGrid {
            width,
            height,
            tile_size,
            nodes,
        }
    }

    fn calculate_node_cost(terrain: &TerrainTile) -> f32 {
        if !terrain.passable {
            return f32::INFINITY;
        }

        // Cost is inversely proportional to speed and turn modifiers
        // Higher friction also increases cost
        let speed_factor = 1.0 / terrain.speed_modifier.max(0.1);
        let turn_factor = 1.0 / terrain.turn_modifier.max(0.1);
        let friction_factor = 1.0 + terrain.friction_modifier;

        // Base cost related to tile type index
        let base_cost = (terrain.tile_id as f32) + 1.0;

        base_cost * speed_factor * turn_factor * friction_factor
    }


    pub fn get_node(&self, x: usize, y: usize) -> Option<&GridNode> {
        if x < self.width && y < self.height {
            Some(&self.nodes[y][x])
        } else {
            None
        }
    }

    pub fn world_to_grid(&self, world_x: f32, world_y: f32) -> (usize, usize) {
        let map_x = world_x + (self.width as f32 * self.tile_size / 2.0);
        let map_y = -world_y + (self.height as f32 * self.tile_size / 2.0);

        let grid_x = (map_x / self.tile_size).floor() as usize;
        let grid_y = (map_y / self.tile_size).floor() as usize;

        (
            grid_x.clamp(0, self.width - 1),
            grid_y.clamp(0, self.height - 1),
        )
    }

    pub fn get_neighbors(&self, x: usize, y: usize) -> Vec<&GridNode> {
        let mut neighbors = Vec::with_capacity(8);

        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue; // Skip the center node
                }

                let nx = x as i32 + dx;
                let ny = y as i32 + dy;

                if nx >= 0 && ny >= 0 {
                    if let Some(node) = self.get_node(nx as usize, ny as usize) {
                        if node.passable {
                            neighbors.push(node);
                        }
                    }
                }
            }
        }

        neighbors
    }

    // Based on Wikipedia pseudocode https://en.wikipedia.org/wiki/Theta* (All Greyson's code. I just ripped it from Map.rs)
    pub fn line_of_sight(&self, point1: (f32, f32), point2: (f32, f32)) -> bool
    {
        let mut x0 = point1.0 as usize;
        let mut y0 = point1.1 as usize;
        let x1 = point2.0 as usize;
        let y1 = point2.1 as usize;

        let dx = (x1 as i32 - x0 as i32).abs();
        let dy = (y1 as i32 - y0 as i32).abs();
        let sx = if x0 < x1 { 1i32 } else { -1i32 };
        let sy = if y0 < y1 { 1i32 } else { -1i32 };
        let mut err = dx - dy;

        loop {
            // Check current tile using nodes instead of terrain_layer
            if let Some(node) = self.get_node(x0, y0) {
                // If we found a wall, no LOS
                if !node.passable {
                    return false;
                }
            } else {
                // Out of bounds
                return false;
            }

            if x0 == x1 && y0 == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 = (x0 as i32 + sx) as usize;
            }
            if e2 < dx {
                err += dx;
                y0 = (y0 as i32 + sy) as usize;
            }
        }

        true
    }
}