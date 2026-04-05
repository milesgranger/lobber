use rand::Rng;

use super::heightmap::Heightmap;
use crate::game::constants::*;

/// Generate a terrain heightmap using midpoint displacement algorithm.
/// This creates natural-looking mountain profiles similar to classic Scorched Earth.
pub fn generate_terrain(width: usize, rng: &mut impl Rng) -> Heightmap {
    let mut heights = vec![0.0f32; width];

    // Start with random endpoints
    heights[0] = rng.gen_range(TERRAIN_MIN_HEIGHT..TERRAIN_MAX_HEIGHT);
    heights[width - 1] = rng.gen_range(TERRAIN_MIN_HEIGHT..TERRAIN_MAX_HEIGHT);

    // Midpoint displacement
    midpoint_displace(&mut heights, 0, width - 1, TERRAIN_MAX_HEIGHT * 0.4, rng);

    // Smooth the result with a simple moving average
    smooth(&mut heights, 3);

    // Clamp to valid range
    for h in &mut heights {
        *h = h.clamp(TERRAIN_MIN_HEIGHT, TERRAIN_MAX_HEIGHT);
    }

    Heightmap::new(heights, TERRAIN_MIN_HEIGHT * 0.5)
}

/// Recursive midpoint displacement for 1D terrain.
fn midpoint_displace(
    heights: &mut [f32],
    left: usize,
    right: usize,
    roughness: f32,
    rng: &mut impl Rng,
) {
    if right - left <= 1 {
        return;
    }

    let mid = (left + right) / 2;
    let avg = (heights[left] + heights[right]) / 2.0;
    let displacement = rng.gen_range(-roughness..roughness);
    heights[mid] = avg + displacement;

    let new_roughness = roughness * 0.6; // Decay factor controls smoothness
    midpoint_displace(heights, left, mid, new_roughness, rng);
    midpoint_displace(heights, mid, right, new_roughness, rng);
}

/// Simple moving average smoothing pass.
fn smooth(heights: &mut [f32], passes: usize) {
    for _ in 0..passes {
        let copy = heights.to_vec();
        for i in 1..heights.len() - 1 {
            heights[i] = (copy[i - 1] + copy[i] + copy[i + 1]) / 3.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn terrain_has_correct_width() {
        let mut rng = StdRng::seed_from_u64(42);
        let terrain = generate_terrain(800, &mut rng);
        assert_eq!(terrain.width(), 800);
    }

    #[test]
    fn terrain_heights_in_range() {
        let mut rng = StdRng::seed_from_u64(42);
        let terrain = generate_terrain(800, &mut rng);
        for h in terrain.heights() {
            assert!(*h >= TERRAIN_MIN_HEIGHT, "Height {} below min", h);
            assert!(*h <= TERRAIN_MAX_HEIGHT, "Height {} above max", h);
        }
    }

    #[test]
    fn terrain_is_deterministic() {
        let mut rng1 = StdRng::seed_from_u64(123);
        let mut rng2 = StdRng::seed_from_u64(123);
        let t1 = generate_terrain(200, &mut rng1);
        let t2 = generate_terrain(200, &mut rng2);
        assert_eq!(t1.heights(), t2.heights());
    }

    #[test]
    fn terrain_has_variation() {
        let mut rng = StdRng::seed_from_u64(42);
        let terrain = generate_terrain(800, &mut rng);
        let min = terrain.heights().iter().cloned().reduce(f32::min).unwrap();
        let max = terrain.heights().iter().cloned().reduce(f32::max).unwrap();
        assert!(max - min > 50.0, "Terrain should have meaningful height variation");
    }
}
