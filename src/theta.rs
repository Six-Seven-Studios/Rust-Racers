use bevy::prelude::Resource;
use crate::map::GameMap;

#[derive(Default)]
pub enum ThetaCommand {
    #[default]
    Stop,
    Forward,
    TurnLeft,
    TurnRight,
}

pub struct ThetaCheckpoint {
    pub point1: (f32, f32),
    pub point2: (f32, f32),
}

#[derive(Resource)]
pub struct ThetaCheckpointManager {
    pub checkpoints: Vec<ThetaCheckpoint>,
    pub current_checkpoint: i32,
}



//Super basic starter implementation that only finds the shortest path to a goal and goes directly towards it
pub fn theta_star(game_map: &GameMap, start_pos: (f32, f32), end_pos: (f32, f32), current_angle: f32) -> ThetaCommand {
    let dx = end_pos.0 - start_pos.0;
    let dy = end_pos.1 - start_pos.1;
    
    let distance = (dx * dx + dy * dy).sqrt();
    
    let goal_threshold = 10.0; // pixels
    if distance < goal_threshold {
        return ThetaCommand::Stop;
    }
    
    let target_angle = dy.atan2(dx);

    //Normalize, so it knows which way to turn
    let pi = std::f32::consts::PI;
    let mut current_normalized = current_angle % (2.0 * pi);
    if current_normalized > pi {
        current_normalized -= 2.0 * pi;
    } else if current_normalized < -pi {
        current_normalized += 2.0 * pi;
    }
    
    let mut angle_diff = target_angle - current_normalized;
    
    if angle_diff > std::f32::consts::PI {
        angle_diff -= 2.0 * std::f32::consts::PI;
    } else if angle_diff < -std::f32::consts::PI {
        angle_diff += 2.0 * std::f32::consts::PI;
    }

    // Give it some wiggle room so it doesn't oscillate (Greyson's idea)
    let angle_threshold = 0.1; // radians (~5.7 degrees)
    
    if angle_diff.abs() < angle_threshold {
        ThetaCommand::Forward
    } else if angle_diff > 0.0 {
        ThetaCommand::TurnLeft
    } else {
        ThetaCommand::TurnRight
    }
}

// Generates a theta* checkpoint line from point1 -> point2
pub fn generate_theta_checkpoint(point1: (f32, f32), point2: (f32, f32)) {

}

// Sets up all theta* checkpoints
// These should be done in order
pub fn set_up_theta_checkpoints(){
    //WARNING: These are just fake points!!!!
    //TODO: Make these real points
    generate_theta_checkpoint((5.0, 7.0),(9.0,2.0));
    generate_theta_checkpoint((10.0, 15.0),(10.0,19.0));
    //and so on...
}

pub fn get_next_pos() -> (f32, f32) {
    (0.0, 0.0)
}