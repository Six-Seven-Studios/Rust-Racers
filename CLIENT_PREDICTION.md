# Client-Side Prediction System

## Overview

This document describes the client-side prediction system implemented for Rust Racers multiplayer racing game. The system eliminates input lag by predicting local player movement while maintaining server authority for anti-cheat.

## Problem Statement

In networked multiplayer games, there's an inherent delay (latency) between:
1. Player presses a key
2. Input is sent to server
3. Server processes input and updates physics
4. Server sends new state back to client
5. Client renders the new position

This round-trip delay (typically 50-150ms) creates noticeable input lag, making the game feel unresponsive.

## Solution: Client-Side Prediction with Server Reconciliation

Our implementation predicts movement locally for instant feedback, then reconciles with the authoritative server state when it arrives.

### Key Principles

1. **Client predicts immediately** - Apply physics locally when input is pressed
2. **Server remains authoritative** - Server is the source of truth
3. **Reconcile differences** - When server state arrives, compare and correct if needed
4. **Smooth corrections** - Use visual interpolation to hide prediction errors

## Architecture

### Module Structure

```
src/prediction/
├── mod.rs                      # Module exports and public API
├── input_buffer.rs             # Ring buffer for storing inputs
├── state_snapshot.rs           # Complete car state at a moment in time
├── client_prediction.rs        # Main prediction system and Bevy resource
├── reconciliation.rs           # Compare predicted vs server state
└── smooth_correction.rs        # Visual interpolation for corrections
```

### Component Overview

#### 1. InputBuffer (`input_buffer.rs`)

**Purpose:** Store recent player inputs with sequence numbers for reconciliation.

**Key Features:**
- Ring buffer with 120 frame capacity (~2 seconds at 60fps)
- Each input has a sequence number for matching with server acknowledgments
- Efficient retrieval of inputs from a specific sequence forward

**API:**
```rust
pub fn add(&mut self, input: PhysicsInput, timestamp: f64) -> u64
pub fn get_from_sequence(&self, start_sequence: u64) -> Vec<TimestampedInput>
pub fn clear_before(&mut self, sequence: u64)
```

**Usage Example:**
```rust
let sequence = input_buffer.add(input, game_time);
// Later, when server acknowledges sequence 50:
let pending = input_buffer.get_from_sequence(51);
```

#### 2. StateSnapshot (`state_snapshot.rs`)

**Purpose:** Capture complete physics state at a specific moment for comparison and rollback.

**Contains:**
- Position (Vec2)
- Velocity (Vec2)
- Rotation angle (f32)
- Sequence number
- Timestamp

**Key Methods:**
```rust
pub fn from_components(...) -> Self
pub fn apply_to_components(...)
pub fn distance_to(&self, other: &StateSnapshot) -> f32
pub fn from_server_data(...) -> Self
```

#### 3. ClientPredictionState (`client_prediction.rs`)

**Purpose:** Main Bevy Resource that orchestrates the prediction system.

**Stores:**
- InputBuffer for reconciliation
- HashMap of predicted states by sequence number
- Last received server state
- Current sequence counter
- Game time accumulator

**Key System:**
```rust
pub fn predict_local_movement(
    prediction_state: ResMut<ClientPredictionState>,
    player_query: Query<...>,
    input: Res<ButtonInput<KeyCode>>,
    game_map: Res<GameMap>,
    time: Res<Time>,
    network_client: ResMut<NetworkClient>,
)
```

**What it does:**
1. Captures current input (WASD + Space)
2. Sends input to server
3. Adds input to buffer with sequence number
4. Applies physics locally (prediction)
5. Stores predicted state for later comparison

#### 4. ReconciliationEngine (`reconciliation.rs`)

**Purpose:** Compare predicted state with server state and re-simulate if needed.

**Algorithm:**
1. Receive server state with acknowledged sequence number
2. Get all pending inputs after that sequence
3. Re-simulate physics from server state using pending inputs
4. Calculate error magnitude
5. Return corrected state

**Key Method:**
```rust
pub fn reconcile(
    server_state: &StateSnapshot,
    input_buffer: &InputBuffer,
    game_map: &GameMap,
    delta: f32,
) -> (StateSnapshot, bool, f32)
```

**Thresholds:**
- Position error: 5 pixels
- Velocity error: 50 pixels/second

#### 5. SmoothCorrection (`smooth_correction.rs`)

**Purpose:** Visual-only interpolation layer to hide prediction snaps.

**How it works:**
1. When prediction error is detected, physics state snaps immediately to correct position
2. A visual offset is added to the rendering transform
3. Over 120ms, the offset smoothly reduces to zero using cubic easing
4. Player never sees the "snap" - only smooth correction

**Component:**
```rust
#[derive(Component)]
pub struct SmoothCorrection {
    pub visual_offset: Vec2,
    pub time_remaining: f32,
    pub initial_offset: Vec2,
}
```

**System:**
```rust
pub fn apply_smooth_correction(
    query: Query<(&mut Transform, &mut SmoothCorrection)>,
    time: Res<Time>,
)
```

## Data Flow

### Frame-by-Frame Execution

**Every Client Frame:**
1. `predict_local_movement` runs:
   - Captures input (W/A/S/D/Space)
   - Sends to server via UDP
   - Adds to input buffer with sequence N
   - Applies physics locally (prediction)
   - Stores predicted state at sequence N

2. `get_car_positions` runs when server message arrives:
   - Receives server state with acknowledged sequence M
   - Calls `reconcile_with_server`:
     - Creates snapshot from server data
     - Compares with our predicted state at sequence M
     - If error > threshold:
       - Re-simulates from server state using inputs M+1 to N
       - Calculates error offset
       - Snaps physics to corrected position
       - Adds SmoothCorrection component with error offset
     - Cleans up old inputs before sequence M

3. `apply_smooth_correction` runs:
   - Updates all SmoothCorrection components
   - Reduces visual offset using cubic easing
   - Applies offset to rendering transform

### Sequence Diagram

```
Client Frame 1:
  Input: W pressed
  → predict_local_movement
    - Send to server (seq 100)
    - Buffer input 100
    - Predict: pos += velocity * dt
    - Store predicted state at seq 100

Client Frame 2-5:
  [More predictions happen...]
  Sequences 101, 102, 103, 104

Client Frame 6:
  Server message arrives: "seq 100 acknowledged, pos = (100, 200)"
  → get_car_positions
    - Compare predicted[100] vs server[100]
    - Error: 2 pixels (below threshold)
    - Re-simulate 101-104
    - Update position smoothly

Client Frame 50:
  Server message: "seq 95 acknowledged" (out of order packet)
  → Already have newer data, ignore old sequence
```

## Configuration

### Feature Flag

Located in `src/prediction/client_prediction.rs:19`:

```rust
pub const ENABLE_PREDICTION: bool = true;
```

Set to `false` to disable prediction and use purely server-authoritative movement. Useful for debugging and comparison.

### Tunable Parameters

**Input Buffer Size** (`input_buffer.rs:33`):
```rust
InputBuffer::new(120)  // ~2 seconds at 60fps
```

**Reconciliation Thresholds** (`reconciliation.rs:7-11`):
```rust
pub const RECONCILIATION_THRESHOLD: f32 = 5.0;  // pixels
pub const VELOCITY_THRESHOLD: f32 = 50.0;       // pixels/second
```

**Smooth Correction Duration** (`smooth_correction.rs:4`):
```rust
pub const CORRECTION_DURATION: f32 = 0.12;  // 120ms
```

## Integration Points

### Modified Files

1. **src/main.rs**
   - Added `mod prediction`
   - Added `ClientPredictionState` resource
   - Added prediction systems to game loop

2. **src/multiplayer.rs**
   - Modified `get_car_positions` to call reconciliation
   - Added `reconcile_with_server` function

3. **src/game_logic/physics.rs**
   - Added `Debug` derive to `PhysicsInput`

4. **src/server/main.rs**
   - Added prediction module path for server compilation

### System Ordering

```rust
Update schedule (GameState::Playing):
  1. predict_local_movement       // Immediate input response
  2. get_car_positions           // Reconcile when server data arrives
  3. apply_smooth_correction     // Visual smoothing
  4. move_camera                 // Camera follows (potentially smoothed) position
```

## Testing and Debugging

### Enable Debug Logging

Prediction errors are automatically logged to console:

```
[Prediction] Error detected: 12.50 pixels at sequence 42
[Prediction] Applied correction with offset: (10.23, 5.67)
```

### Visual Debugging (Optional)

You can add visual indicators by modifying `smooth_correction.rs` to draw the error offset:

```rust
// In apply_smooth_correction:
gizmos.line_2d(
    corrected_position,
    corrected_position + visual_offset,
    Color::RED
);
```

### Testing Scenarios

1. **No latency test:**
   - Run server and client locally
   - Prediction should be nearly perfect
   - Few or no corrections

2. **Artificial latency test:**
   - Use network tools to add 100ms latency
   - Movement should still feel responsive
   - Occasional corrections visible in logs

3. **Packet loss test:**
   - Drop 5-10% of packets
   - System should handle gracefully
   - Re-simulation fills gaps

4. **Comparison test:**
   - Set `ENABLE_PREDICTION = false`
   - Compare input responsiveness
   - Prediction version should feel instant

## Performance Considerations

### CPU Cost

- **Input buffering:** O(1) per frame
- **Prediction:** One physics simulation per frame (~0.1ms)
- **Reconciliation:** Re-simulation of N pending inputs (~0.1ms × N)
  - Typical N = 3-5 inputs
  - Worst case: 120 inputs (unlikely)

### Memory Cost

- InputBuffer: ~2KB (120 inputs × ~16 bytes each)
- Predicted states: ~4KB (60 states × ~64 bytes each)
- Total: <10KB per player

### Network Cost

**No change** - Same input messages sent as before. The `input_count` field was already being sent by the server.

## Edge Cases Handled

1. **Out-of-order packets:** Compare sequence numbers, ignore stale data
2. **Packet loss:** Input buffer maintains history for re-simulation
3. **Large prediction errors:** Threshold-based correction with visual smoothing
4. **Server-side collisions:** Server state is always trusted, client re-simulates
5. **Terrain changes:** Both client and server use same GameMap

## Future Enhancements

### Potential Improvements

1. **Velocity prediction:** Currently only position is actively predicted
2. **Rotation smoothing:** Add separate smoothing for angle corrections
3. **Adaptive thresholds:** Adjust based on measured latency
4. **Multi-step prediction:** Predict multiple frames ahead
5. **Lag compensation visualization:** Show predicted vs server positions

### Known Limitations

1. **Collisions with other players:** Cannot be perfectly predicted (requires server simulation)
2. **Map changes:** Dynamic terrain requires server synchronization
3. **High packet loss (>20%):** May cause frequent corrections

## Troubleshooting

### Issue: Car still feels laggy

**Solutions:**
1. Verify `ENABLE_PREDICTION = true`
2. Check console for frequent corrections (may indicate physics mismatch)
3. Ensure `predict_local_movement` runs before `get_car_positions`

### Issue: Car "jumps" or "teleports"

**Cause:** Prediction error above threshold without smooth correction.

**Solutions:**
1. Check if `SmoothCorrection` component is being added
2. Verify `apply_smooth_correction` is running
3. Increase `RECONCILIATION_THRESHOLD` to reduce corrections
4. Check for physics desync between client and server

### Issue: Corrections happen every frame

**Cause:** Physics simulation differs between client and server.

**Solutions:**
1. Ensure both use same `apply_physics` function
2. Check terrain modifiers are identical
3. Verify delta time is consistent
4. Look for floating-point precision issues

## References

### Further Reading

- [Valve's Source Engine Networking](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking)
- [Gabriel Gambetta's Fast-Paced Multiplayer](https://www.gabrielgambetta.com/client-side-prediction-server-reconciliation.html)
- [Overwatch Gameplay Architecture](https://www.youtube.com/watch?v=W3aieHjyNvw)

### Code References

All prediction code is located in:
- **Client-side:** `src/prediction/` module
- **Integration:** `src/multiplayer.rs:95-179`
- **System setup:** `src/main.rs:96-102`

## Summary

This client-side prediction system provides:

✅ **Instant input feedback** - No perceived lag on local player
✅ **Server authority maintained** - Anti-cheat and consistency preserved
✅ **Smooth corrections** - Prediction errors are visually masked
✅ **Modular design** - Easy to test, debug, and extend
✅ **Low overhead** - Minimal CPU and memory cost

The system handles typical multiplayer scenarios (latency, packet loss, out-of-order packets) gracefully while maintaining the responsive feel of a local game.
