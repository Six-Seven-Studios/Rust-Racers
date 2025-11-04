use bevy::prelude::*;

#[derive(Component)]
pub struct Car;

#[derive(Component)]
pub struct PlayerControlled;

#[derive(Component)]
pub struct AIControlled;

#[derive(Component, Clone)]
pub struct Orientation {
    pub angle: f32,
}

impl Orientation {
    pub fn new(angle: f32) -> Self {
        Self { angle }
    }

    pub fn forward_vector(&self) -> Vec2 {
        Vec2::new(self.angle.cos(), self.angle.sin())
    }
}

#[derive(Component, Clone, Deref, DerefMut)]
pub struct Velocity {
    pub velocity: Vec2,
}

impl Velocity {
    pub fn new() -> Self {
        Self {
            velocity: Vec2::ZERO,
        }
    }
}

impl From<Vec2> for Velocity {
    fn from(velocity: Vec2) -> Self {
        Self { velocity }
    }
}
