use crate::game_logic::map::GameMap;
use crate::game_logic::terrain::TerrainTile;

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

pub struct ThetaGrid {
    pub width: usize,
    pub height: usize,
    pub tile_size: f32,
    nodes: Vec<Vec<GridNode>>,
}

impl ThetaGrid {
    pub fn create_theta_grid(game_map: &GameMap, tile_size: f32) -> Self {
        let width = game_map.width as usize;
        let height = game_map.height as usize;
        ();

        let mut nodes = Vec::with_capacity(height);

        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let terrain = game_map.get_tile(x as f32, y as f32, 64.0);

                // Convert grid coordinates to world coordinates
                let world_x =
                    (x as f32 * tile_size) - (width as f32 * tile_size / 2.0) + (tile_size / 2.0);
                let world_y = -((y as f32 * tile_size) - (height as f32 * tile_size / 2.0)
                    + (tile_size / 2.0));

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

        // tile_id is indexed by how bad it is to drive on (0=best, 5=worst, 6=wall)
        // Add 1 to ensure ROAD=0 has some cost
        (terrain.tile_id as f32) + 1.0
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
}
