use crate::game_logic::{Checkpoint, GameMap};

#[derive(Default)]
pub enum ThetaCommand {
    #[default]
    Stop,
    Forward,
    TurnLeft,
    TurnRight,
}

#[derive(Clone)]
pub struct ThetaCheckpoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone)]
pub struct ThetaCheckpointList {
    pub checkpoints: Vec<ThetaCheckpoint>,
    pub current_checkpoint_index: usize,
}

impl ThetaCheckpointList {
    pub fn new(checkpoints: Vec<ThetaCheckpoint>) -> Self {
        ThetaCheckpointList {
            checkpoints,
            current_checkpoint_index: 0,
        }
    }

    pub fn current_checkpoint(&self) -> &ThetaCheckpoint {
        &self.checkpoints[self.current_checkpoint_index]
    }

    pub fn advance_checkpoint(&mut self) {
        self.current_checkpoint_index = (self.current_checkpoint_index + 1) % self.checkpoints.len();
    }

    pub fn reset(&mut self) {
        self.current_checkpoint_index = 0;
    }
}

//Super basic starter implementation that only finds the shortest path to a goal and goes directly towards it
pub fn theta_star(start_pos: (f32, f32), current_angle: f32, checkpoints: &mut ThetaCheckpointList) -> ThetaCommand {

    //Grab the current checkpoint from the checkpoint list
    let current_cp = checkpoints.current_checkpoint();
    let end_pos = (current_cp.x, current_cp.y);

    //Calc that distance rq
    let dx = end_pos.0 - start_pos.0;
    let dy = end_pos.1 - start_pos.1;
    
    let distance = (dx * dx + dy * dy).sqrt();
    //end of distance formula (will probably need this later)

    let goal_threshold = 100.0; // pixels
    if distance < goal_threshold {
        checkpoints.advance_checkpoint();
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