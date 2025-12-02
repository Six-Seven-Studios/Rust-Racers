use crate::game_logic::{
    apply_physics, calculate_steering_command, theta_star, CpuDifficulty, Orientation,
    PhysicsInput, ThetaCheckpointList, ThetaCommand, ThetaGrid, Velocity,
};
use bevy::prelude::*;
use std::time::Duration;

// defining some states for our car
// https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html
#[derive(Component)]
pub struct CarState {
    // unfortunately state is not thread safe, so it can't derive Component
    // unless State is enforced to Send + Sync bounds
    state: Option<Box<dyn State>>,
}

// enums for the different transitions between car driving types
enum Transition {
    None,
    ToAggressive,
    ToNeutral,
}

impl CarState {
    pub fn new() -> CarState {
        CarState {
            state: Some(Box::new(Neutral::new())),
        }
    }

    // this is what will be called every frame to control the behavior of the AI
    pub fn update(
        &mut self,
        delta_time: &mut Res<Time>,
        transform: &mut Transform,
        velocity: &mut Velocity,
        orientation: &mut Orientation,
        car_nearby: bool,
        closest_car_position: Option<Vec2>,
        closest_car_distance: f32,
        difficulty: &CpuDifficulty,
        checkpoints: &mut ThetaCheckpointList,
        grid: &Res<ThetaGrid>,
    ) {
        if let Some(mut s) = self.state.take() {
            // do the current state's operations

            let transition: Transition = s.execute(
                delta_time,
                transform,
                velocity,
                orientation,
                car_nearby,
                closest_car_position,
                closest_car_distance,
                difficulty,
                checkpoints,
                grid,
            );

            // transition based off of what each state returns
            self.state = Some(match transition {
                Transition::None => s,
                Transition::ToNeutral => s.to_neutral(),
                Transition::ToAggressive => s.to_aggressive(),
            });
        }
    }
}

// state defines a behavior shared by different CarState states
trait State: Send + Sync {
    fn to_neutral(self: Box<Self>) -> Box<dyn State>;
    fn to_aggressive(self: Box<Self>) -> Box<dyn State>;

    // execute will return true in the case of a success
    // execute should contain some conditions to change to different states
    fn execute(
        &mut self,
        delta_time: &mut Res<Time>,
        transform: &mut Transform,
        velocity: &mut Velocity,
        orientation: &mut Orientation,
        car_nearby: bool,
        closest_car_position: Option<Vec2>,
        closest_car_distance: f32,
        difficulty: &CpuDifficulty,
        checkpoints: &mut ThetaCheckpointList,
        grid: &Res<ThetaGrid>,
    ) -> Transition;
}

// the state objects are aggressive, Neutral, etc.
struct Aggressive {
    ram_timer: Timer,
}

impl Aggressive {
    pub fn new() -> Self {
        Self {
            ram_timer: Timer::new(Duration::from_millis(2000), TimerMode::Once),
        }
    }
}

impl State for Aggressive {
    // TRANSITIONS BETWEEN STATES
    // --------------------------
    fn to_neutral(self: Box<Self>) -> Box<dyn State> {
        Box::new(Neutral::new())
    }
    fn to_aggressive(self: Box<Self>) -> Box<dyn State> {
        Box::new(Aggressive::new())
    }
    // --------------------------
    fn execute(
        &mut self,
        delta_time: &mut Res<Time>,
        transform: &mut Transform,
        velocity: &mut Velocity,
        orientation: &mut Orientation,
        car_nearby: bool,
        closest_car_position: Option<Vec2>,
        closest_car_distance: f32,
        difficulty: &CpuDifficulty,
        _checkpoints: &mut ThetaCheckpointList,
        _grid: &Res<ThetaGrid>,
    ) -> Transition {
        self.ram_timer.tick(delta_time.delta());

        if self.ram_timer.finished() {
            return Transition::ToNeutral;
        }

        // MAIN DRIVING LOGIC GOES HERE
        if car_nearby {
            if let Some(target_pos) = closest_car_position {
                let start_pos = (transform.translation.x, transform.translation.y);
                let command = calculate_steering_command(
                    start_pos,
                    (target_pos.x, target_pos.y),
                    orientation.angle,
                );

                let mut input = PhysicsInput::default();
                match command {
                    ThetaCommand::Forward => {
                        input.forward = true;
                    }
                    ThetaCommand::Reverse => {
                        input.backward = true;
                    }
                    ThetaCommand::TurnLeft => {
                        input.left = true;
                        input.forward = true;
                    }
                    ThetaCommand::TurnRight => {
                        input.right = true;
                        input.forward = true;
                    }
                    ThetaCommand::Stop => {
                        // do nothing
                    }
                }

                // For ramming, let's add boost.
                input.boost = true;

                let mut pos = transform.translation.truncate();
                apply_physics(
                    &mut pos,
                    velocity,
                    orientation,
                    &input,
                    delta_time.delta_secs(),
                    1.0, // speed_modifier
                    1.0, // friction_modifier
                    1.0, // turn_modifier
                    1.0, // decel_modifier
                );
                transform.translation.x = pos.x;
                transform.translation.y = pos.y;
            }
        }

        // transition back to neutral if no car is nearby
        if !car_nearby {
            // info!("[+] No cars nearby, switching back to neutral driving");
            Transition::ToNeutral
        } else {
            Transition::None
        }
    }
}

struct Neutral {
    decision_timer: Timer,
}

impl Neutral {
    pub fn new() -> Self {
        Self {
            // slower decision interval
            decision_timer: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
        }
    }
}

impl State for Neutral {
    // TRANSITIONS BETWEEN STATES
    // --------------------------
    fn to_neutral(self: Box<Self>) -> Box<dyn State> {
        Box::new(Neutral::new())
    }

    fn to_aggressive(self: Box<Self>) -> Box<dyn State> {
        Box::new(Aggressive::new())
    }
    // --------------------------
    fn execute(
        &mut self,
        delta_time: &mut Res<Time>,
        transform: &mut Transform,
        velocity: &mut Velocity,
        orientation: &mut Orientation,
        car_nearby: bool,
        closest_car_position: Option<Vec2>,
        closest_car_distance: f32,
        difficulty: &CpuDifficulty,
        checkpoints: &mut ThetaCheckpointList,
        grid: &Res<ThetaGrid>,
    ) -> Transition {
        // MAIN DRIVING LOGIC GOES HERE
        self.decision_timer.tick(delta_time.delta());

        // check if a car is nearby - if so, immediately switch to aggressive
        if car_nearby {
            info!(
                "[+] car detected at distance {:.1}! Switching to aggressive mode!",
                closest_car_distance
            );
            return Transition::ToAggressive;
        }

        // using theta* here to drive normally
        if self.decision_timer.just_finished() {
            let start_pos = (transform.translation.x, transform.translation.y);
            let command = theta_star(start_pos, orientation.angle, checkpoints, grid);

            let mut input = PhysicsInput::default();
            match command {
                ThetaCommand::Forward => {
                    input.forward = true;
                }
                ThetaCommand::Reverse => {
                    input.backward = true;
                }
                ThetaCommand::TurnLeft => {
                    input.left = true;
                    input.forward = true;
                }
                ThetaCommand::TurnRight => {
                    input.right = true;
                    input.forward = true;
                }
                ThetaCommand::Stop => {
                    // do nothing
                }
            }
            let mut pos = transform.translation.truncate();
            apply_physics(
                &mut pos,
                velocity,
                orientation,
                &input,
                delta_time.delta_secs(),
                1.0,
                1.0,
                1.0,
                1.0,
            );
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
        Transition::None
    }
}
