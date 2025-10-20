use crate::map::GameMap;

#[derive(Default)]
pub enum ThetaCommand {
    #[default]
    Stop,
    Forward,
    TurnLeft,
    TurnRight,
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