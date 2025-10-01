//Easy static vars for referencing which road tile we are on. Corresponds directly to TILES array
//indexed based off how bad they are to drive on. Higher == worse
const ROAD:u8 = 0;
const WET:u8 = 1;
const DIRT:u8 = 2;
const GRASS:u8 = 3;
const SAND:u8 = 4;
const OIL:u8 = 5;
//static WALL:u8 = 6;

pub struct TerrainTile {
    pub friction_modifier: f32,
    pub speed_modifier: f32,
    pub turn_modifier: f32,
}

pub const TILES: [TerrainTile; 6] = [
    TerrainTile {friction_modifier: 1.0, speed_modifier: 1.0, turn_modifier: 1.0},
    TerrainTile {friction_modifier: 0.8, speed_modifier: 1.0, turn_modifier: 1.0},
    TerrainTile {friction_modifier: 0.68, speed_modifier: 0.9, turn_modifier: 0.7},
    TerrainTile {friction_modifier: 0.36, speed_modifier: 0.7, turn_modifier: 0.4},
    TerrainTile {friction_modifier: 0.2, speed_modifier: 0.4, turn_modifier: 0.2},
    TerrainTile {friction_modifier: 0.0, speed_modifier: 1.0, turn_modifier: 0.0},
    //TerrainTile {tile_type: WALL, friction_modifier: 1.0, speed_modifier: 1.0, turn_modifier: 1.0, passable: false},
];