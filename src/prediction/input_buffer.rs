use crate::game_logic::physics::PhysicsInput;
use std::collections::VecDeque;

/// Stores a timestamped input with a sequence number for reconciliation
#[derive(Clone, Debug)]
pub struct TimestampedInput {
    pub sequence: u64,
    pub input: PhysicsInput,
    pub timestamp: f64, // Game time when input was captured
}

/// Ring buffer that stores recent player inputs for prediction and reconciliation
///
/// This buffer allows us to:
/// - Re-simulate physics from a past server state
/// - Handle out-of-order or dropped packets
/// - Maintain a history for debugging prediction errors
pub struct InputBuffer {
    buffer: VecDeque<TimestampedInput>,
    max_size: usize,
    next_sequence: u64,
}

impl InputBuffer {
    /// Create a new input buffer with a maximum capacity
    ///
    /// Suggested size: 120 frames (~2 seconds at 60fps)
    /// This handles RTT up to ~1 second comfortably
    pub fn new(max_size: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_size),
            max_size,
            next_sequence: 0,
        }
    }

    /// Add a new input to the buffer and return its sequence number
    ///
    /// The sequence number is used to match client predictions with server acknowledgments
    pub fn add(&mut self, input: PhysicsInput, timestamp: f64) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence += 1;

        self.buffer.push_back(TimestampedInput {
            sequence,
            input,
            timestamp,
        });

        // Maintain maximum buffer size
        if self.buffer.len() > self.max_size {
            self.buffer.pop_front();
        }

        sequence
    }

    /// Get all inputs from a specific sequence number forward
    ///
    /// Used for re-simulation during reconciliation:
    /// Server says "I processed inputs up to sequence N"
    /// Client re-simulates from that point using all inputs after N
    pub fn get_from_sequence(&self, start_sequence: u64) -> Vec<TimestampedInput> {
        self.buffer
            .iter()
            .filter(|input| input.sequence >= start_sequence)
            .cloned()
            .collect()
    }

    /// Get the input at a specific sequence number
    pub fn get_at_sequence(&self, sequence: u64) -> Option<&TimestampedInput> {
        self.buffer
            .iter()
            .find(|input| input.sequence == sequence)
    }

    /// Remove all inputs before a given sequence number
    ///
    /// Called after successful reconciliation to free memory
    /// Keep a small buffer for late-arriving packets
    pub fn clear_before(&mut self, sequence: u64) {
        self.buffer.retain(|input| input.sequence >= sequence);
    }

    /// Get the most recent input sequence number
    pub fn latest_sequence(&self) -> u64 {
        self.next_sequence.saturating_sub(1)
    }

    /// Get the number of inputs currently buffered
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_add_and_retrieve() {
        let mut buffer = InputBuffer::new(10);

        let input1 = PhysicsInput {
            forward: true,
            ..Default::default()
        };

        let seq1 = buffer.add(input1.clone(), 0.0);
        assert_eq!(seq1, 0);

        let seq2 = buffer.add(input1.clone(), 0.016);
        assert_eq!(seq2, 1);

        assert_eq!(buffer.len(), 2);
    }

    #[test]
    fn test_buffer_max_size() {
        let mut buffer = InputBuffer::new(5);
        let input = PhysicsInput::default();

        for i in 0..10 {
            buffer.add(input.clone(), i as f64);
        }

        // Should only keep the last 5
        assert_eq!(buffer.len(), 5);
    }

    #[test]
    fn test_get_from_sequence() {
        let mut buffer = InputBuffer::new(10);
        let input = PhysicsInput::default();

        buffer.add(input.clone(), 0.0);
        buffer.add(input.clone(), 0.016);
        buffer.add(input.clone(), 0.032);

        let from_seq_1 = buffer.get_from_sequence(1);
        assert_eq!(from_seq_1.len(), 2);
        assert_eq!(from_seq_1[0].sequence, 1);
        assert_eq!(from_seq_1[1].sequence, 2);
    }
}
