// src/game_logic/difficulty.rs
use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CpuDifficulty {
    Easy,
    Medium,
    Hard,
}

impl Default for CpuDifficulty {
    fn default() -> Self {
        CpuDifficulty::Medium
    }
}

impl CpuDifficulty {
    pub fn as_str(&self) -> &'static str {
        match self {
            CpuDifficulty::Easy => "Easy",
            CpuDifficulty::Medium => "Medium",
            CpuDifficulty::Hard => "Hard",
        }
    }

    pub fn next(self) -> Self {
        match self {
            CpuDifficulty::Easy => CpuDifficulty::Medium,
            CpuDifficulty::Medium => CpuDifficulty::Hard,
            CpuDifficulty::Hard => CpuDifficulty::Easy,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            CpuDifficulty::Easy => CpuDifficulty::Hard,
            CpuDifficulty::Medium => CpuDifficulty::Easy,
            CpuDifficulty::Hard => CpuDifficulty::Medium,
        }
    }
}