use glam::Vec2;

use crate::game::constants::*;

/// Manages the animated playback of a projectile trajectory.
pub struct TrajectoryAnimation {
    /// All positions in the trajectory.
    positions: Vec<Vec2>,
    /// Current playback index.
    current_index: usize,
    /// Playback speed: how many positions to advance per frame.
    speed: usize,
}

impl TrajectoryAnimation {
    pub fn new(positions: Vec<Vec2>) -> Self {
        Self {
            positions,
            current_index: 0,
            speed: 3,
        }
    }

    /// Advance the animation by one frame. Returns true if still playing.
    pub fn advance(&mut self) -> bool {
        if self.is_complete() {
            return false;
        }
        self.current_index = (self.current_index + self.speed).min(self.positions.len() - 1);
        true
    }

    pub fn is_complete(&self) -> bool {
        self.current_index >= self.positions.len() - 1
    }

    /// Get the current projectile position.
    pub fn current_position(&self) -> Vec2 {
        self.positions[self.current_index]
    }

    /// Get the trail (recent positions) for rendering.
    pub fn trail(&self) -> &[Vec2] {
        let start = self.current_index.saturating_sub(TRAIL_LENGTH);
        &self.positions[start..=self.current_index]
    }

    /// Get all positions up to current for the full trail line.
    pub fn path_so_far(&self) -> &[Vec2] {
        &self.positions[..=self.current_index]
    }
}
