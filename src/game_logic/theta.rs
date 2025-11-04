use bevy::prelude::Component;
use rand::prelude::*;
use crate::game_logic::{Checkpoint, GameMap, Car};
use rand::Rng;

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
    pub point1: (f32, f32),
    pub point2: (f32, f32),
}

impl ThetaCheckpoint {
    pub fn new(point1: (f32, f32), point2: (f32, f32)) -> Self {
        Self { point1, point2 }
    }
}

#[derive(Component)]
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

    pub fn advance_checkpoint(&mut self) {
        self.current_checkpoint_index = (self.current_checkpoint_index + 1) % self.checkpoints.len();
    }
    pub fn load_checkpoint_list(&mut self, map_num: u8) -> ThetaCheckpointList {
        let mut checkpoints: Vec<ThetaCheckpoint> = Vec::new();
        if (map_num == 1) {
            checkpoints.push(ThetaCheckpoint::new((91.0, 18.0), (94.0, 18.0)));
            checkpoints.push(ThetaCheckpoint::new((91.0, 10.0), (94.0, 10.0)));
            checkpoints.push(ThetaCheckpoint::new((85.0, 9.0), (85.0, 5.0)));
            checkpoints.push(ThetaCheckpoint::new((59.0, 5.0), (59.0, 8.0)));
            checkpoints.push(ThetaCheckpoint::new((54.0, 11.0), (57.0, 11.0)));
            checkpoints.push(ThetaCheckpoint::new((54.0, 20.0), (52.0, 18.0)));
            checkpoints.push(ThetaCheckpoint::new((49.0, 30.0), (52.0, 30.0)));
            checkpoints.push(ThetaCheckpoint::new((45.0, 38.0), (45.0, 41.0)));
            checkpoints.push(ThetaCheckpoint::new((31.0, 41.0), (31.0, 38.0)));
            checkpoints.push(ThetaCheckpoint::new((20.0, 46.0), (20.0, 43.0)));
            checkpoints.push(ThetaCheckpoint::new((11.0, 47.0), (11.0, 43.0)));
            checkpoints.push(ThetaCheckpoint::new((5.0, 50.0), (8.0, 50.0)));
            checkpoints.push(ThetaCheckpoint::new((8.0, 56.0), (5.0, 56.0)));
            checkpoints.push(ThetaCheckpoint::new((16.0, 68.0), (18.0, 68.0)));
            checkpoints.push(ThetaCheckpoint::new((16.0, 74.0), (19.0, 74.0)));
            checkpoints.push(ThetaCheckpoint::new((7.0, 84.0), (10.0, 84.0)));
            checkpoints.push(ThetaCheckpoint::new((15.0, 94.0), (15.0, 91.0)));
            checkpoints.push(ThetaCheckpoint::new((33.0, 94.0), (33.0, 91.0)));
            checkpoints.push(ThetaCheckpoint::new((35.0, 89.0), (38.0, 89.0)));
            checkpoints.push(ThetaCheckpoint::new((40.0, 86.0), (40.0, 83.0)));
            checkpoints.push(ThetaCheckpoint::new((53.0, 83.0), (53.0, 86.0)));
            checkpoints.push(ThetaCheckpoint::new((59.0, 89.0), (54.0, 89.0)));
            checkpoints.push(ThetaCheckpoint::new((60.0, 91.0), (60.0, 94.0)));
            checkpoints.push(ThetaCheckpoint::new((89.0, 91.0), (89.0, 94.0)));
            checkpoints.push(ThetaCheckpoint::new((91.0, 89.0), (94.0, 89.0)));
            checkpoints.push(ThetaCheckpoint::new((91.0, 34.0), (93.0, 44.0)));

        } else if (map_num == 2) {
            // No checkpoints implemented yet!
            panic!("Checkpoints not implemented for map 2 yet!");
        }
        else { panic!("Invalid map num: {}", map_num); }
        return ThetaCheckpointList::new(checkpoints);
    }
}

pub fn get_next_point(list: &ThetaCheckpointList) -> (f32, f32) {
    let mut rng = rand::thread_rng();

    let curr_checkpoint: ThetaCheckpoint = list.checkpoints[list.current_checkpoint_index].clone();


    let rand_x: f32 =
        if (curr_checkpoint.point1.0 < curr_checkpoint.point2.0){rng.gen_range(curr_checkpoint.point1.0..=curr_checkpoint.point2.0)}
        else{ rng.gen_range(curr_checkpoint.point2.0..=curr_checkpoint.point1.0)};

    let rand_y: f32 =
        if (curr_checkpoint.point1.1 < curr_checkpoint.point2.1){rng.gen_range(curr_checkpoint.point1.1..=curr_checkpoint.point2.1)}
        else{ rng.gen_range(curr_checkpoint.point2.1..=curr_checkpoint.point1.1)};

    return (rand_x, rand_y);
}

//Super basic starter implementation that only finds the shortest path to a goal and goes directly towards it
pub fn theta_star(start_pos: (f32, f32), current_angle: f32, checkpoints: &mut ThetaCheckpointList) -> ThetaCommand {
    if checkpoints.checkpoints.is_empty() {
        return ThetaCommand::Stop;
    }

    //Grab the current checkpoint from the checkpoint list
    let current_cp = get_next_point(&checkpoints);
    let end_pos = (current_cp.0, current_cp.1);
    println!("Current: {}, {}", start_pos.0, start_pos.1);
    println!("Goal: {}, {}", end_pos.0, end_pos.1);
    //Calc that distance rq
    let dx = end_pos.0 - start_pos.0;
    let dy = end_pos.1 - start_pos.1;

    let distance = (dx * dx + dy * dy).sqrt();
    //end of distance formula (will probably need this later)

    let goal_threshold = 5.0; // pixels
    if distance < goal_threshold {
        println!("ADVANCE");
        checkpoints.advance_checkpoint();
    }

    // Negate dy because tile Y-axis is flipped (increasing Y goes down in world space)
    let target_angle = (-dy).atan2(dx);

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