use crate::game_logic::{
    DIRT, GRASS, OIL, ROAD, SAND, TILES, TerrainTile, ThetaCheckpoint, ThetaCheckpointList, WALL,
    WET,
};
use bevy::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Resource, Clone)]
pub struct GameMap {
    pub height: f32,
    pub width: f32,

    // logical terrain for physics
    pub terrain_layer: Vec<Vec<TerrainTile>>,

    // visual only layers
    pub visual_layers: Vec<Vec<Vec<u8>>>, // Vec<Layer<Rows<Tiles>>>
}

//I gave it an argument called "filename" in order to make it WAY easier for us to do multiple maps if we want
pub fn load_map_from_file(filename: &str) -> GameMap {
    let fd = File::open(filename).expect("failed to open map file");
    let reader = BufReader::new(fd);
    let mut lines = reader.lines().map(|l| l.expect("failed to read line"));

    // read map dimensions
    let dims = lines.next().expect("missing map dimensions");
    let parts: Vec<_> = dims.split_whitespace().collect();
    let (width, height) = (
        parts[0].parse::<f32>().expect("failed to parse map width"),
        parts[1].parse::<f32>().expect("failed to parse map height"),
    );

    let mut terrain_layer: Vec<Vec<TerrainTile>> = Vec::new();
    let mut visual_layers: Vec<Vec<Vec<u8>>> = Vec::new();
    let mut current_layer: Vec<Vec<u8>> = Vec::new();
    let mut is_terrain = true;

    // helper to map raw tile index to logical terrain
    fn create_terrain_tile(tile_index: u8, x: usize, y: usize) -> TerrainTile {
        // get a copy of the correct template (ROAD, GRASS, etc.)
        let mut template = match tile_index {
            0..=15 => TILES[ROAD as usize].clone(),
            16..=31 => TILES[WET as usize].clone(),
            32..=47 => TILES[DIRT as usize].clone(),
            48..=63 => TILES[GRASS as usize].clone(),
            64..=79 => TILES[SAND as usize].clone(),
            80..=95 => TILES[OIL as usize].clone(),
            112..=127 => TILES[WALL as usize].clone(), // 0x70-0x7F in hex
            _ => TILES[GRASS as usize].clone(),
        };

        // overwrite the template's visual ID with the specific ID from the map file
        template.tile_id = tile_index;

        // set the tile coordinates for pathfinding
        template.x_coordinate = x as f32;
        template.y_coordinate = y as f32;

        // return the finished tile
        template
    }

    for line in lines.chain(std::iter::once("---".to_string())) {
        // check if the line is a delimiter
        if line.starts_with("---") {
            // if we have data in the current layer, process it now
            if !current_layer.is_empty() {
                if is_terrain {
                    // convert u8 rows to TerrainTile rows
                    let terrain_rows: Vec<Vec<TerrainTile>> = current_layer
                        .drain(..)
                        .enumerate()
                        .map(|(y, row)| {
                            row.into_iter()
                                .enumerate()
                                .map(|(x, tile_index)| create_terrain_tile(tile_index, x, y))
                                .collect()
                        })
                        .collect();
                    terrain_layer = terrain_rows;
                    is_terrain = false;
                } else {
                    visual_layers.push(current_layer.drain(..).collect());
                }
            }
            // skip the delimiter line itself from being parsed as data
            continue;
        }

        // if the line is not a delimiter, parse it as a row of tiles
        let row: Vec<u8> = line
            .trim()
            .split_whitespace()
            .filter(|s| !s.is_empty()) // filter to handle potential extra spaces
            .map(|s| u8::from_str_radix(s, 16).unwrap()) // changing this to interpet HEX
            .collect();

        // avoid adding empty rows if the line was blank
        if !row.is_empty() {
            current_layer.push(row);
        }
    }

    GameMap {
        width,
        height,
        terrain_layer,
        visual_layers,
    }
}

/*
    rendering the map from the GameMap and tile atlas
*/
pub fn spawn_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
    game_map: Res<GameMap>,
) {
    // using nearest neighbor filtering so that there isn't weird gaps between tiles
    let texture_handle = asset_server.load("aseprite-tiles/tiles.png");

    // create the atlas based on the dimension of tiles.png
    let layout = TextureAtlasLayout::from_grid(UVec2::new(64, 64), 16, 16, None, None);
    let layout_handle = atlases.add(layout);

    let tile_size = 64.0;

    // rendering the terrain (logical) layer first
    for (y, row) in game_map.terrain_layer.iter().enumerate() {
        for (x, tile_id) in row.iter().enumerate() {
            if tile_id.tile_id == 255 {
                continue;
            }
            commands.spawn((
                Sprite::from_atlas_image(
                    texture_handle.clone(),
                    TextureAtlas {
                        layout: layout_handle.clone(),
                        index: tile_id.tile_id as usize,
                    },
                ),
                Transform::from_xyz(
                    x as f32 * tile_size - game_map.width / 2.0 + tile_size / 2.0,
                    -(y as f32 * tile_size) + game_map.height / 2.0 - tile_size / 2.0,
                    1.0, // terrain on top
                ),
            ));
        }
    }

    // now, rendering the visual layers afterward
    for (layer_index, layer) in game_map.visual_layers.iter().enumerate() {
        for (y, row) in layer.iter().enumerate() {
            for (x, tile_id) in row.iter().enumerate() {
                if *tile_id == 255 {
                    continue;
                } // assume 255 is empty.
                commands.spawn((
                    Sprite::from_atlas_image(
                        texture_handle.clone(),
                        TextureAtlas {
                            layout: layout_handle.clone(),
                            index: *tile_id as usize,
                        },
                    ),
                    // need to place the tiles starting from the topleft of the map.
                    Transform::from_xyz(
                        x as f32 * tile_size - game_map.width / 2.0 + tile_size / 2.0,
                        -(y as f32 * tile_size) + game_map.height / 2.0 - tile_size / 2.0,
                        0.1 + layer_index as f32 * 0.1, // important so no z-fighting
                    ),
                ));
            }
        }
    }
}

impl GameMap {
    // get tile from a world position
    pub fn get_tile(&self, world_x: f32, world_y: f32, tile_size: f32) -> &TerrainTile {
        // Or whatever your Tile struct is
        // translate from world origin (center) to map origin (top-left)
        // this shifts the coordinates so that (0,0) is the top-left of the map.
        let map_x = world_x + self.width / 2.0;

        // need to invert y-axis it because +y is up in the world,
        // but down in the array.
        let map_y = -world_y + self.height / 2.0;

        // convert from pixels to tile indices
        // divide by the tile size and floor the result to get the array index.
        let mut tile_x = (map_x / tile_size).floor() as usize;
        let mut tile_y = (map_y / tile_size).floor() as usize;

        // clamp so we're sure that even if the car is slightly off the map, we don't crash.
        let max_y = self.terrain_layer.len() - 1;
        let max_x = self.terrain_layer[0].len() - 1;

        tile_x = tile_x.clamp(0, max_x);
        tile_y = tile_y.clamp(0, max_y);

        // return t ile
        &self.terrain_layer[tile_y][tile_x]
    }

    // Based on Wikipedia pseudocode https://en.wikipedia.org/wiki/Theta*
    pub fn line_of_sight(&self, point1: (f32, f32), point2: (f32, f32)) -> bool {
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
            // Check current tile
            let tile = &self.terrain_layer[y0][x0];

            // If we found a wall, no LOS
            if !tile.passable {
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

    // For debug
    pub fn tile_to_world(&self, tile_x: f32, tile_y: f32, tile_size: f32) -> Vec2 {
        let world_x = tile_x * tile_size - self.width / 2.0 + tile_size / 2.0;
        let world_y = -(tile_y * tile_size) + self.height / 2.0 - tile_size / 2.0;
        Vec2::new(world_x, world_y)
    }
}

impl Default for GameMap {
    fn default() -> Self {
        load_map_from_file("assets/big-map.txt")
    }
}
