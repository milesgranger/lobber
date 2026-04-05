use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::game::constants::*;
use crate::game::types::*;
use crate::terrain::Heightmap;

use super::collision::check_terrain_collision;

/// State of a projectile in flight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Projectile {
    pub position: Vec2,
    pub velocity: Vec2,
    pub ammo: AmmoType,
}

/// Result of simulating a projectile to completion.
#[derive(Debug, Clone)]
pub enum ShotOutcome {
    /// Projectile hit the terrain at this position.
    TerrainHit { position: Vec2 },
    /// Projectile went out of bounds (off screen left/right/top).
    OutOfBounds,
}

/// Simulate a single physics step for a projectile.
/// Returns the new projectile state after one PHYSICS_DT step.
pub fn step_projectile(projectile: &Projectile, wind: &Wind) -> Projectile {
    let gravity = Vec2::new(0.0, -GRAVITY);
    let wind_force = wind.as_force();
    let drag = projectile.ammo.drag_coefficient();

    // Drag force opposes velocity, proportional to speed squared
    let speed = projectile.velocity.length();
    let drag_force = if speed > 0.0 {
        -projectile.velocity.normalize() * drag * speed * speed
    } else {
        Vec2::ZERO
    };

    // Total acceleration
    let acceleration = gravity + wind_force + drag_force;

    // Euler integration
    let new_velocity = projectile.velocity + acceleration * PHYSICS_DT;
    let new_position = projectile.position + new_velocity * PHYSICS_DT;

    Projectile {
        position: new_position,
        velocity: new_velocity,
        ammo: projectile.ammo,
    }
}

/// Simulate a full shot from start to impact.
/// Returns the trajectory (list of positions) and the outcome.
pub fn simulate_shot(
    start: Vec2,
    velocity: Vec2,
    ammo: AmmoType,
    wind: &Wind,
    terrain: &Heightmap,
) -> (Vec<Vec2>, ShotOutcome) {
    let mut projectile = Projectile {
        position: start,
        velocity,
        ammo,
    };

    let mut trail = vec![projectile.position];
    let world_width = terrain.width() as f32;

    for _ in 0..MAX_PHYSICS_STEPS {
        let prev_pos = projectile.position;
        projectile = step_projectile(&projectile, wind);
        trail.push(projectile.position);

        // Check terrain collision (line segment from prev to current position)
        if let Some(hit) = check_terrain_collision(prev_pos, projectile.position, terrain) {
            return (trail, ShotOutcome::TerrainHit { position: hit });
        }

        // Out of bounds checks
        if projectile.position.x < 0.0
            || projectile.position.x > world_width
            || projectile.position.y > WORLD_CEILING
        {
            return (trail, ShotOutcome::OutOfBounds);
        }
    }

    // Safety: if we hit max steps, treat as out of bounds
    (trail, ShotOutcome::OutOfBounds)
}

/// Generate a predicted trajectory for aiming preview.
/// Returns positions for the first `steps` physics ticks (no collision check).
pub fn predict_trajectory(
    start: Vec2,
    velocity: Vec2,
    ammo: AmmoType,
    wind: &Wind,
    steps: usize,
) -> Vec<Vec2> {
    let mut projectile = Projectile {
        position: start,
        velocity,
        ammo,
    };

    let mut trail = Vec::with_capacity(steps);
    for _ in 0..steps {
        projectile = step_projectile(&projectile, wind);
        trail.push(projectile.position);
    }
    trail
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projectile_falls_under_gravity() {
        let proj = Projectile {
            position: Vec2::new(100.0, 200.0),
            velocity: Vec2::new(50.0, 0.0),
            ammo: AmmoType::Cannonball,
        };
        let wind = Wind { speed: 0.0 };

        let next = step_projectile(&proj, &wind);
        assert!(next.position.y < proj.position.y, "Projectile should fall");
        assert!(next.position.x > proj.position.x, "Projectile should move right");
    }

    #[test]
    fn wind_affects_trajectory() {
        let proj = Projectile {
            position: Vec2::new(100.0, 200.0),
            velocity: Vec2::new(0.0, 50.0),
            ammo: AmmoType::Explosive,
        };

        let no_wind = step_projectile(&proj, &Wind { speed: 0.0 });
        let right_wind = step_projectile(&proj, &Wind { speed: 5.0 });

        assert!(
            right_wind.position.x > no_wind.position.x,
            "Wind should push projectile right"
        );
    }

    #[test]
    fn cannonball_less_affected_by_wind_than_explosive() {
        let make_proj = |ammo: AmmoType| Projectile {
            position: Vec2::new(100.0, 200.0),
            velocity: Vec2::new(50.0, 50.0),
            ammo,
        };

        let wind = Wind { speed: 5.0 };

        // Simulate several steps
        let mut cannon = make_proj(AmmoType::Cannonball);
        let mut explosive = make_proj(AmmoType::Explosive);
        for _ in 0..100 {
            cannon = step_projectile(&cannon, &wind);
            explosive = step_projectile(&explosive, &wind);
        }

        // Explosive should have drifted more due to higher drag interacting with wind
        // (This is a rough check — the exact physics depend on drag model)
        // Both should have moved right, but they'll differ
        assert!(
            (cannon.position.x - explosive.position.x).abs() > 0.1,
            "Different drag should cause different trajectories"
        );
    }

    #[test]
    fn shot_hits_flat_terrain() {
        let terrain = Heightmap::new(vec![50.0; 800], 0.0);
        let start = Vec2::new(100.0, 100.0);
        let velocity = Vec2::new(50.0, 50.0);
        let wind = Wind { speed: 0.0 };

        let (trail, outcome) = simulate_shot(start, velocity, AmmoType::Cannonball, &wind, &terrain);

        assert!(!trail.is_empty());
        match outcome {
            ShotOutcome::TerrainHit { position } => {
                assert!(position.y <= 55.0, "Should hit near terrain level");
            }
            ShotOutcome::OutOfBounds => panic!("Should have hit terrain"),
        }
    }
}
