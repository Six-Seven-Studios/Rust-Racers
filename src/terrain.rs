//Easy static vars for referencing which road tile we are on. Corresponds directly to TILES array
//indexed based off how bad they are to drive on. Higher == worse
pub const ROAD:u8 = 0;
pub const WET:u8 = 1;
pub const DIRT:u8 = 2;
pub const GRASS:u8 = 3;
pub const SAND:u8 = 4;
pub const OIL:u8 = 5;
//static WALL:u8 = 6;

#[derive(Clone)]
pub struct TerrainTile {
    pub tile_id: u8, // for rendering
    pub friction_modifier: f32,
    pub speed_modifier: f32,
    pub turn_modifier: f32,
    pub decel_modifier: f32,
}

pub const TILES: [TerrainTile; 6] = [
    TerrainTile { tile_id: 0, friction_modifier: 1.0, speed_modifier: 1.5, turn_modifier: 1.0, decel_modifier: 400.0},
    TerrainTile { tile_id: 1, friction_modifier: 0.8, speed_modifier: 1.5, turn_modifier: 1.0, decel_modifier: 350.0},
    TerrainTile { tile_id: 2, friction_modifier: 0.68, speed_modifier: 1.35, turn_modifier: 0.7, decel_modifier: 370.0},
    TerrainTile { tile_id: 3, friction_modifier: 0.5, speed_modifier: 1.05, turn_modifier: 0.4, decel_modifier: 430.0},
    TerrainTile { tile_id: 4, friction_modifier: 0.2, speed_modifier: 0.6, turn_modifier: 0.2, decel_modifier: 500.0},
    TerrainTile { tile_id: 5, friction_modifier: 0.1, speed_modifier: 1.5, turn_modifier: 0.1,decel_modifier: 0.0},
    //TerrainTile {tile_type: WALL, friction_modifier: 1.0, speed_modifier: 1.0, turn_modifier: 1.0, passable: false},
];