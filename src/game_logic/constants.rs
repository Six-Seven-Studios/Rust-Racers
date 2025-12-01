// Client runs at 60 Hz for input capture and prediction
pub const CLIENT_TIMESTEP: f32 = 1.0 / 60.0; // 16.67ms - 60 Hz

// Server runs at 20 Hz for authoritative simulation
pub const SERVER_TIMESTEP: f32 = 1.0 / 60.0; // 16.67ms - 60 Hz

pub const FRICTION: f32 = 0.95;
pub const ACCEL_RATE: f32 = 600.0;
pub const TURNING_RATE: f32 = 3.0;
pub const PLAYER_SPEED: f32 = 400.0;
pub const LATERAL_FRICTION: f32 = 8.0;
pub const EASY_DRIFT_TURN_MULTIPLIER: f32 = 1.35;
pub const EASY_DRIFT_SPEED_BONUS: f32 = 1.1;
pub const EASY_DRIFT_LATERAL_FRICTION: f32 = 4.0;

pub const CAR_SIZE: u32 = 64;
pub const TILE_SIZE: u32 = 64;

// Fixed grid of starting positions (world coordinates) for up to 4 racers
pub const START_POSITIONS: [(f32, f32); 4] = [
    (2752.0, 960.0),
    (2852.0, 960.0),
    (2752.0, 860.0),
    (2852.0, 860.0),
];

// Orientation (radians) for spawned cars so they face the track direction
pub const START_ORIENTATION: f32 = std::f32::consts::FRAC_PI_2;
