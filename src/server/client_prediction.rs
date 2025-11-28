// Stub module for server compilation
// Server doesn't need client prediction, but shared modules (car.rs, multiplayer.rs) import it

use crate::game_logic::physics::PhysicsInput;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct InputSequence {
    pub current: u64,
}

#[derive(Clone)]
pub struct PredictedState {
    pub sequence: u64,
    pub input: PhysicsInput,
    pub position: Vec2,
    pub velocity: Vec2,
    pub angle: f32,
}

#[derive(Component)]
pub struct PredictionBuffer {
    pub states: Vec<PredictedState>,
}

impl PredictionBuffer {
    pub fn new() -> Self {
        Self { states: Vec::new() }
    }
}
