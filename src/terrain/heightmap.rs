use serde::{Deserialize, Serialize};

/// A 1D heightmap representing terrain as a series of height values.
/// Each index corresponds to an x-coordinate in world space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heightmap {
    /// Height values indexed by x position.
    heights: Vec<f32>,
    /// Minimum allowed height.
    min_height: f32,
}

impl Heightmap {
    pub fn new(heights: Vec<f32>, min_height: f32) -> Self {
        Self {
            heights,
            min_height,
        }
    }

    pub fn width(&self) -> usize {
        self.heights.len()
    }

    /// Get the terrain height at an exact integer x position.
    pub fn height_at_index(&self, x: usize) -> f32 {
        self.heights.get(x).copied().unwrap_or(0.0)
    }

    /// Get the interpolated terrain height at a floating-point x position.
    pub fn height_at(&self, x: f32) -> f32 {
        if x < 0.0 || x >= self.heights.len() as f32 {
            return 0.0;
        }

        let x0 = x.floor() as usize;
        let x1 = (x0 + 1).min(self.heights.len() - 1);
        let frac = x - x.floor();

        self.heights[x0] * (1.0 - frac) + self.heights[x1] * frac
    }

    /// Apply a crater at the given x position with the specified radius.
    /// Uses a parabolic crater profile.
    pub fn apply_crater(&mut self, center_x: f32, radius: f32) {
        let start = ((center_x - radius).floor() as isize).max(0) as usize;
        let end = ((center_x + radius).ceil() as usize).min(self.heights.len());

        for x in start..end {
            let dx = x as f32 - center_x;
            let normalized = dx / radius;
            // Parabolic crater profile: deepest at center
            let depth = radius * 0.5 * (1.0 - normalized * normalized);
            self.heights[x] = (self.heights[x] - depth).max(self.min_height);
        }
    }

    /// Get all heights as a slice (for rendering).
    pub fn heights(&self) -> &[f32] {
        &self.heights
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_interpolation() {
        let hm = Heightmap::new(vec![10.0, 20.0, 30.0], 0.0);
        assert_eq!(hm.height_at(0.0), 10.0);
        assert_eq!(hm.height_at(1.0), 20.0);
        assert_eq!(hm.height_at(0.5), 15.0);
    }

    #[test]
    fn height_out_of_bounds() {
        let hm = Heightmap::new(vec![10.0, 20.0], 0.0);
        assert_eq!(hm.height_at(-1.0), 0.0);
        assert_eq!(hm.height_at(100.0), 0.0);
    }

    #[test]
    fn crater_reduces_height() {
        let mut hm = Heightmap::new(vec![100.0; 100], 0.0);
        let original_center = hm.height_at(50.0);
        hm.apply_crater(50.0, 10.0);
        assert!(hm.height_at(50.0) < original_center);
    }

    #[test]
    fn crater_respects_min_height() {
        let mut hm = Heightmap::new(vec![5.0; 100], 3.0);
        hm.apply_crater(50.0, 20.0);
        for h in hm.heights() {
            assert!(*h >= 3.0);
        }
    }

    #[test]
    fn crater_is_deepest_at_center() {
        let mut hm = Heightmap::new(vec![100.0; 100], 0.0);
        hm.apply_crater(50.0, 10.0);
        let center = hm.height_at(50.0);
        let edge = hm.height_at(45.0);
        assert!(center < edge, "Center should be deeper than edge");
    }
}
