use glam::Vec2;

use crate::terrain::Heightmap;

/// Check if a projectile path segment (from `prev` to `curr`) intersects the terrain.
/// Returns the approximate impact point if a collision occurred.
pub fn check_terrain_collision(prev: Vec2, curr: Vec2, terrain: &Heightmap) -> Option<Vec2> {
    let width = terrain.width() as f32;

    // Determine x range to check
    let x_min = prev.x.min(curr.x).max(0.0);
    let x_max = prev.x.max(curr.x).min(width - 1.0);

    if x_min >= width || x_max < 0.0 {
        return None;
    }

    // Sample along the segment at each integer x position
    let steps = ((x_max - x_min).ceil() as usize).max(1);

    for i in 0..=steps {
        let t = if steps == 0 {
            0.5
        } else {
            i as f32 / steps as f32
        };
        let point = prev.lerp(curr, t);

        if point.x < 0.0 || point.x >= width {
            continue;
        }

        let terrain_height = terrain.height_at(point.x);
        if point.y <= terrain_height {
            return Some(point);
        }
    }

    // Also check the exact endpoints
    if curr.x >= 0.0 && curr.x < width && curr.y <= terrain.height_at(curr.x) {
        return Some(curr);
    }

    None
}

/// Check if a point is below the terrain surface.
#[allow(dead_code)]
pub fn is_below_terrain(point: Vec2, terrain: &Heightmap) -> bool {
    let width = terrain.width() as f32;
    if point.x < 0.0 || point.x >= width {
        return false;
    }
    point.y <= terrain.height_at(point.x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_collision_with_flat_terrain() {
        let terrain = Heightmap::new(vec![50.0; 100], 0.0);
        let prev = Vec2::new(50.0, 100.0);
        let curr = Vec2::new(50.0, 30.0);

        let hit = check_terrain_collision(prev, curr, &terrain);
        assert!(hit.is_some());
    }

    #[test]
    fn no_collision_above_terrain() {
        let terrain = Heightmap::new(vec![50.0; 100], 0.0);
        let prev = Vec2::new(20.0, 100.0);
        let curr = Vec2::new(30.0, 80.0);

        let hit = check_terrain_collision(prev, curr, &terrain);
        assert!(hit.is_none());
    }

    #[test]
    fn detects_collision_on_hill() {
        let mut heights = vec![50.0; 100];
        // Create a hill at x=50
        for i in 40..60 {
            heights[i] = 150.0;
        }
        let terrain = Heightmap::new(heights, 0.0);

        let prev = Vec2::new(45.0, 200.0);
        let curr = Vec2::new(55.0, 100.0);

        let hit = check_terrain_collision(prev, curr, &terrain);
        assert!(hit.is_some());
    }

    #[test]
    fn is_below_terrain_works() {
        let terrain = Heightmap::new(vec![50.0; 100], 0.0);
        assert!(is_below_terrain(Vec2::new(50.0, 30.0), &terrain));
        assert!(!is_below_terrain(Vec2::new(50.0, 80.0), &terrain));
    }
}
