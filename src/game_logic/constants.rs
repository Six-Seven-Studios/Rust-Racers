// Network and physics timing
pub const FIXED_TIMESTEP: f32 = 1.0 / 30.0; // 33.33ms - 30 Hz (reduces UDP packet loss)

// Physics constants
pub const FRICTION: f32 = 0.95;
pub const ACCEL_RATE: f32 = 600.0;
pub const TURNING_RATE: f32 = 3.0;
pub const PLAYER_SPEED: f32 = 400.0;
pub const LATERAL_FRICTION: f32 = 8.0;

// Rendering constants
pub const CAR_SIZE: u32 = 64;
pub const TILE_SIZE: u32 = 64;
