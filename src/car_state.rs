use bevy::prelude::*;
use crate::game_logic::{Velocity, Orientation};
use rand::prelude::*;
use std::time::Duration;


// defining some states for our car
// https://doc.rust-lang.org/book/ch18-03-oo-design-patterns.html
#[derive(Component)]
pub struct CarState {
    // unfortunately state is not thread safe, so it can't derive Component
    // unless State is enforced to Send + Sync bounds
    state: Option<Box<dyn State>>,
}

#[derive(Component)]
struct FuseTime {
    /// track when the bomb should explode (non-repeating timer)
    timer: Timer,
}

// enums for the different transitions between car driving types
enum Transition {
    None,
    ToAggressive,
    ToNeutral,
    // ToAttack,
}

/**

Random generator

**/

pub fn generate_number() -> i32 {

    // Generates a value between 1 and 10
    let mut rng = rand::rng();
    return rng.random_range(1..=10);

}

impl CarState {
    pub fn new() -> CarState {
        CarState {
            state: Some(Box::new(Neutral::new())), // default to aggressive
        }
    }
    /*
    pub fn to_defense(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.to_defense())
        }
    }

    pub fn to_aggressive(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.to_aggressive())
        }
    }

    
    pub fn execute(&self) -> Transition {
        self.state.as_ref().unwrap().execute(self)
    }
    */
    // this is what will be called every frame to control the behavior of the AI
    pub fn update(
        &mut self,
        delta_time: &mut Res<Time>,
        transform: &mut Transform,
        velocity: &mut Velocity,
        orientation: &mut Orientation,
    ){
        if let Some(mut s) = self.state.take() {
            // do the current state's operations



            let transition: Transition = s.execute(delta_time, transform, velocity, orientation);
            
            // transition based off of what each state returns
            self.state = Some(match transition {
                Transition::None => s,
                Transition::ToNeutral => s.to_neutral(),
                Transition::ToAggressive => s.to_aggressive(),
                // Transition::ToAttack => s.to_attack(),
            });
        }
    }
}

// state defines a behavior shared by different CarState states
trait State: Send + Sync {
    fn to_neutral(self: Box<Self>) -> Box<dyn State>;
    fn to_aggressive(self: Box<Self>) -> Box<dyn State>;
    // fn to_attack(self: Box<Self>) -> Box<dyn State>;

    // execute will return true in the case of a success
    // execute should contain some conditions to change to different states
    fn execute(
        &mut self,
        delta_time: &mut Res<Time>,
        transform: &mut Transform,
        velocity: &mut Velocity,
        orientation: &mut Orientation,
    ) -> Transition;
}

// the state objects are aggressive, Neutral, etc.
struct Aggressive {
    
}

impl State for Aggressive {
    // TRANSITIONS BETWEEN STATES
    // --------------------------
    fn to_neutral(self: Box<Self>) -> Box<dyn State> {
        Box::new(Neutral::new())
    }
    fn to_aggressive(self: Box<Self>) -> Box<dyn State> {
        self
    }
    // --------------------------
    fn execute(
        &mut self,
        delta_time: &mut Res<Time>,
        transform: &mut Transform,
        velocity: &mut Velocity,
        orientation: &mut Orientation,
    ) -> Transition {
        // MAIN DRIVING LOGIC GOES HERE
        info!("Driving aggressively!");

        // using theta* to pursue the player

        
        let some_driving_condition: bool = true;
        if some_driving_condition == true {
            // info!("Switching to defensive driving!");
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
        self // we're already in Neutral...
    }

    fn to_aggressive(self: Box<Self>) -> Box<dyn State> {
        Box::new(Aggressive {})
        // use theta star to target player
        // increase velocity to ram
    }
    // --------------------------
    fn execute(
        &mut self,
        delta_time: &mut Res<Time>,
        transform: &mut Transform,
        velocity: &mut Velocity,
        orientation: &mut Orientation,
    ) -> Transition {
        // MAIN DRIVING LOGIC GOES HERE
        // TODO: use transform, velocity, etc to move the car
        self.decision_timer.tick(delta_time.delta());
        // info!("{:?} ticking...", delta_time.delta_secs());

        // using theta* here to drive normally
        if self.decision_timer.just_finished() {
            let rage_meter = generate_number();
            info!("{:?}", rage_meter);
            if rage_meter >= 9 {
                info!("Switching to offensive driving!");
                return Transition::ToAggressive
            }
            
        }
        Transition::None
    }
}
// struct Attack {}

// impl State for Attack {
//     // TRANSITIONS BETWEEN STATES
//     // --------------------------
//     fn to_attack(self: Box<Self>) -> Box<dyn State> {
//         self // we're already in Neutral...
//     }

//     fn to_aggressive(self: Box<Self>) -> Box<dyn State> {
//         Box::new(Aggressive {})
//     }
//     // --------------------------

//     fn execute(&self,
//         transform: &mut Transform,
//         velocity: &mut Velocity,
//         orientation: &mut Orientation,
//     ) -> Transition {
//         // MAIN DRIVING LOGIC GOES HERE
//         // TODO: use transform, velocity, etc to move the car

//         info!("Attacking!");
//         info!("{:?}", transform.translation);

//         info!("SPAWN MISSILE HERE!");

//         let some_driving_condition: bool = true;
//         if some_driving_condition == true {
//             info!("Switching to offensive driving!");
//             Transition::ToAggressive
//         } else {
//             Transition::None
//         }
//     }
// }


// // this is doing some weird rust ownership stuff I don't fully understand
// // i just sort of copied the structure from the rust book and added extra bevy functions