use bevy::prelude::*;

/// Duration of smooth correction in seconds
pub const CORRECTION_DURATION: f32 = 0.12; // 120ms

/// Component that stores visual correction state for smooth interpolation
///
/// This is a visual-only layer that sits between physics state and rendering.
/// When prediction error is detected:
/// - Physics immediately snaps to correct position
/// - Rendering smoothly interpolates from error to correct over ~120ms
#[derive(Component)]
pub struct SmoothCorrection {
    /// The visual offset from the true physics position
    pub visual_offset: Vec2,

    /// How much time is left in the correction (counts down to 0)
    pub time_remaining: f32,

    /// Initial offset magnitude (for easing calculation)
    pub initial_offset: Vec2,
}

impl SmoothCorrection {
    /// Start a new smooth correction from an error position to the correct position
    pub fn start(error_offset: Vec2) -> Self {
        Self {
            visual_offset: error_offset,
            time_remaining: CORRECTION_DURATION,
            initial_offset: error_offset,
        }
    }

    /// Update the correction, returning the current visual offset
    pub fn update(&mut self, delta: f32) -> Vec2 {
        if self.time_remaining <= 0.0 {
            return Vec2::ZERO;
        }

        self.time_remaining -= delta;

        if self.time_remaining <= 0.0 {
            self.visual_offset = Vec2::ZERO;
            return Vec2::ZERO;
        }

        // Cubic ease-out for smooth, natural-looking correction
        let t = 1.0 - (self.time_remaining / CORRECTION_DURATION);
        let ease = Self::ease_out_cubic(t);

        // Interpolate from initial offset to zero
        self.visual_offset = self.initial_offset * (1.0 - ease);
        self.visual_offset
    }

    /// Cubic ease-out function for smooth deceleration
    /// https://easings.net/#easeOutCubic
    fn ease_out_cubic(t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        1.0 - (1.0 - t).powi(3)
    }

    /// Check if correction is complete
    pub fn is_complete(&self) -> bool {
        self.time_remaining <= 0.0
    }
}

/// System that applies smooth visual correction to rendering transforms
///
/// This runs after physics updates to add a visual offset to the transform.
/// The offset gradually reduces to zero, creating smooth correction animation.
pub fn apply_smooth_correction(
    mut query: Query<(&mut Transform, &mut SmoothCorrection)>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();

    for (mut transform, mut correction) in query.iter_mut() {
        let visual_offset = correction.update(delta);

        // Apply visual offset to rendering position
        // Note: This modifies the rendering transform, not the physics position
        transform.translation.x += visual_offset.x;
        transform.translation.y += visual_offset.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correction_completes() {
        let mut correction = SmoothCorrection::start(Vec2::new(10.0, 0.0));

        // Simulate frames until correction completes
        let mut frames = 0;
        while !correction.is_complete() && frames < 100 {
            correction.update(0.016); // 60fps
            frames += 1;
        }

        assert!(correction.is_complete());
        assert_eq!(correction.visual_offset, Vec2::ZERO);
    }

    #[test]
    fn test_correction_reduces_offset() {
        let mut correction = SmoothCorrection::start(Vec2::new(10.0, 0.0));

        let initial_magnitude = correction.visual_offset.length();
        correction.update(0.016);
        let after_frame_magnitude = correction.visual_offset.length();

        assert!(after_frame_magnitude < initial_magnitude);
    }

    #[test]
    fn test_ease_out_cubic() {
        // At t=0, ease should be 0
        assert_eq!(SmoothCorrection::ease_out_cubic(0.0), 0.0);

        // At t=1, ease should be 1
        assert!((SmoothCorrection::ease_out_cubic(1.0) - 1.0).abs() < 0.001);

        // At t=0.5, ease should be > 0.5 (fast start, slow end)
        assert!(SmoothCorrection::ease_out_cubic(0.5) > 0.5);
    }
}
