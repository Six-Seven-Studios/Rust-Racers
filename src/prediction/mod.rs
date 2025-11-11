// Client-side prediction module for reducing input lag in multiplayer
//
// This module implements client-side prediction with server reconciliation:
// 1. Client predicts movement locally for instant feedback
// 2. Server remains authoritative and processes inputs
// 3. Client reconciles predictions with server state
// 4. Smooth visual correction handles any mispredictions

pub mod input_buffer;
pub mod state_snapshot;
pub mod client_prediction;
pub mod reconciliation;
pub mod smooth_correction;

pub use input_buffer::InputBuffer;
pub use state_snapshot::StateSnapshot;
pub use client_prediction::{ClientPredictionState, predict_local_movement, ENABLE_PREDICTION};
pub use reconciliation::ReconciliationEngine;
pub use smooth_correction::{SmoothCorrection, apply_smooth_correction};
