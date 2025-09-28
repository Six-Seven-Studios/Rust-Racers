use std::fs::File;
use std::io::{BufReader, BufRead};
use crate::Resource;

#[derive(Resource)]
pub struct GameMap {
    pub height: f32,
    pub width: f32,
    pub tile_grid: Vec<Vec<u8>>
}

//I gave it an argument called "filename" in order to make it WAY easier for us to do multiple maps if we want
pub fn load_map_from_file(filename: &str) -> GameMap {
    let fd = File::open(filename).unwrap();
    let mut reader = BufReader::new(fd);
    let mut line = String::new();

    // Read first line to get dimensions
    reader.read_line(&mut line).unwrap();
    let dimensions: Vec<&str> = line.trim().split_whitespace().collect();
    let (width, height) = (dimensions[0].parse::<f32>()
                                    .unwrap(), dimensions[1].parse::<f32>().unwrap());

    // Read map data
    let mut tiles = Vec::new();
    for _ in 0.. (height/64.0) as u32 {
        line.clear();
        reader.read_line(&mut line).unwrap();
        let row: Vec<u8> = line.trim().split_whitespace()
                            .map(|c| c.parse().unwrap()).collect();
        tiles.push(row);
    }

    return GameMap {width: width as f32,
                    height: height as f32,
                    tile_grid: tiles}
}

impl GameMap {
    // Tiler Getter
    pub fn get_tile(&self, world_x: f32, world_y: f32, tile_size: f32) -> u8 {
        let half_w = self.width * 0.5;
        let half_h = self.height * 0.5;

        // Shift so (0,0) is the top-left of the map
        let local_x = world_x + half_w;
        let local_y = world_y + half_h;

        // Convert to tile indices
        let tile_x = (local_x / tile_size).floor() as usize;
        let tile_y = (local_y / tile_size).floor() as usize;

        // Return tile number
        self.tile_grid[tile_y][tile_x]
    }
}