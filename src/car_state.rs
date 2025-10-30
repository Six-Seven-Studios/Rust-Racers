use bevy::prelude::*;
use crate::car::{Velocity, Orientation};
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
    ToOffense,
    ToDefense,
}

impl CarState {
    pub fn new() -> CarState {
        CarState {
            state: Some(Box::new(Offense {})), // default to offense
        }
    }

    pub fn to_defense(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.to_defense())
        }
    }

    pub fn to_offense(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.to_offense())
        }
    }

    /*
    pub fn execute(&self) -> Transition {
        self.state.as_ref().unwrap().execute(self)
    }
    */
    // this is what will be called every frame to control the behavior of the AI
    pub fn update(
        &mut self,
        transform: &mut Transform,
        velocity: &mut Velocity,
        orientation: &mut Orientation,
    ){
        if let Some(s) = self.state.take() {
            // do the current state's operations
            let transition = s.execute(transform, velocity, orientation);
            
            // transition based off of what each state returns
            self.state = Some(match transition {
                Transition::None => s,
                Transition::ToDefense => s.to_defense(),
                Transition::ToOffense => s.to_offense(),
            });
        }
    }
}

// state defines a behavior shared by different CarState states
trait State: Send + Sync {
    fn to_defense(self: Box<Self>) -> Box<dyn State>;
    fn to_offense(self: Box<Self>) -> Box<dyn State>;

    // execute will return true in the case of a success
    // execute should contain some conditions to change to different states
    fn execute(
        &self,
        transform: &mut Transform,
        velocity: &mut Velocity,
        orientation: &mut Orientation,
    ) -> Transition;
}

// the state objects are offense, defense, etc.
struct Offense {}

impl State for Offense {
    // TRANSITIONS BETWEEN STATES
    // --------------------------
    fn to_defense(self: Box<Self>) -> Box<dyn State> {
        Box::new(Defense {})
    }
    fn to_offense(self: Box<Self>) -> Box<dyn State> {
        self
    }
    // --------------------------
    fn execute(&self,
        transform: &mut Transform,
        velocity: &mut Velocity,
        orientation: &mut Orientation,
    ) -> Transition {
        // MAIN DRIVING LOGIC GOES HERE
        info!("Driving aggressively!");
        let some_driving_condition: bool = true;
        if some_driving_condition == true {
            info!("Switching to defensive driving!");
            Transition::ToDefense
        } else {
            Transition::None
        }
    }
    
}

struct Defense {}

impl State for Defense {
    // TRANSITIONS BETWEEN STATES
    // --------------------------
    fn to_defense(self: Box<Self>) -> Box<dyn State> {
        self // we're already in defense...
    }

    fn to_offense(self: Box<Self>) -> Box<dyn State> {
        Box::new(Offense {})
    }
    // --------------------------

    fn execute(&self,
        transform: &mut Transform,
        velocity: &mut Velocity,
        orientation: &mut Orientation,
    ) -> Transition {
        // MAIN DRIVING LOGIC GOES HERE
        // TODO: use transform, velocity, etc to move the car

        info!("Driving defensively!");
        info!("{:?}", transform.translation);
        let some_driving_condition: bool = true;
        if some_driving_condition == true {
            info!("Switching to offensive driving!");
            Transition::ToOffense
        } else {
            Transition::None
        }
    }
}

// this is doing some weird rust ownership stuff I don't fully understand
// i just sort of copied the structure from the rust book and added extra bevy functions