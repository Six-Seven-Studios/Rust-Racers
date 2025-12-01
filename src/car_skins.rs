use bevy::prelude::Resource;
use rand::seq::IteratorRandom;

/// Available car skin asset paths (player/remote cars).
pub const CAR_SKINS: &[&str] = &[
    "blue-car.png",
    "car.png",
    "heart-car.png",
    "jeremy-car.png",
    "kameren-car.png",
    "red-car.png",
    "stevie-the-star.png",
    "67mobile.png",
];

/// Dedicated AI skin.
pub const AI_SKIN: &str = "CPU.png";

#[derive(Resource, Clone)]
pub struct CarSkinSelection {
    pub index: usize,
}

impl Default for CarSkinSelection {
    fn default() -> Self {
        Self { index: 0 }
    }
}

impl CarSkinSelection {
    pub fn current_skin(&self) -> &str {
        CAR_SKINS
            .get(self.index)
            .copied()
            .unwrap_or_else(|| CAR_SKINS[0])
    }

    pub fn current_label(&self) -> String {
        self.current_skin().to_string()
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % CAR_SKINS.len();
    }

    pub fn prev(&mut self) {
        if self.index == 0 {
            self.index = CAR_SKINS.len() - 1;
        } else {
            self.index -= 1;
        }
    }

    /// Choose a random skin for other players (can optionally avoid matching the local selection).
    pub fn random_other(&self) -> &str {
        let mut rng = rand::thread_rng();
        CAR_SKINS
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != self.index)
            .map(|(_, s)| *s)
            .choose(&mut rng)
            .unwrap_or_else(|| self.current_skin())
    }
}
