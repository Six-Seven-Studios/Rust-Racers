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
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
pub struct ThetaCheckpoint {
    pub point1: Point,
    pub point2: Point,
}

impl ThetaCheckpoint {
    pub fn new(point1: Point, point2: Point) -> Self {
        Self { point1, point2 }
    }
}

#[derive(Resource)]
pub struct ThetaCheckpointManager {
    pub checkpoints: Vec<ThetaCheckpoint>,
    pub current_checkpoint: usize,
    pub num_checkpoints: usize,
}

impl ThetaCheckpointManager {
    fn new() -> ThetaCheckpointManager {
        ThetaCheckpointManager {
            checkpoints: Vec::new(),
            current_checkpoint: usize::MAX,
            num_checkpoints: 0,
        }
    }

    // Creates a theta* checkpoint line from point1 -> point2
    pub fn add_theta_checkpoint(&mut self, point1: (f32, f32), point2: (f32, f32)) {
        let p1 = Point::new(point1.0 as f32, point1.1 as f32);
        let p2 = Point::new(point2.0 as f32, point2.1 as f32);
        self.checkpoints.push(ThetaCheckpoint::new(p1, p2));
        self.num_checkpoints += 1;
    }

    // If you need the next pos, make sure to reference the ThetaCheckpointManager resource
    pub fn get_next_pos(&mut self) -> Point {
        // -1 , 3
        self.advance_counter();

        if(self.current_checkpoint >= self.num_checkpoints) {
            self.reset_counter();
        }

        // TODO: Select random point along the line of the two point
        return (self.checkpoints[self.current_checkpoint]).point1;
    }

    pub fn advance_counter(&mut self){
        self.current_checkpoint += 1;
    }

    pub fn reset_counter(&mut self) {
        self.current_checkpoint = 0;
    }
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



// Sets up all theta* checkpoints
// These should be done in order
pub fn set_up_theta_checkpoints() -> ThetaCheckpointManager {
    // Create ThetaCheckpointManager
    let mut cpm = ThetaCheckpointManager::new();

    //WARNING: These are just fake points!!!!
    //TODO: Make these real points
    // The first checkpoint should be the start line
    cpm.add_theta_checkpoint((5.0, 7.0),(9.0,2.0));
    cpm.add_theta_checkpoint((10.0, 15.0),(10.0,19.0));
    //and so on...

    return cpm;
}
