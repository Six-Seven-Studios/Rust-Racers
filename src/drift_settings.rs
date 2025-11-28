use bevy::prelude::*;

/// Global settings that control how drifting behaves.
#[derive(Resource, Clone)]
pub struct DriftSettings {
    pub easy_mode: bool,
}

impl Default for DriftSettings {
    fn default() -> Self {
        Self { easy_mode: false }
    }
}

impl DriftSettings {
    /// Toggle the easy drifting mode and return the new state.
    pub fn toggle(&mut self) -> bool {
        self.easy_mode = !self.easy_mode;
        self.easy_mode
    }

    /// Helper for displaying the current mode in UI.
    pub fn mode_label(&self) -> &str {
        if self.easy_mode { "ON" } else { "OFF" }
    }
}
