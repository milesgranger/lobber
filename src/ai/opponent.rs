use rand::Rng;

use crate::game::constants::*;
use crate::game::types::*;
use crate::terrain::Heightmap;

/// AI difficulty level — controls how accurate the computer's aim is.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum AiDifficulty {
    Easy,
    Medium,
    Hard,
}

impl AiDifficulty {
    /// How much random error to add to the AI's aim (in degrees and power %).
    fn aim_error(self) -> (f32, f32) {
        match self {
            AiDifficulty::Easy => (15.0, 20.0),
            AiDifficulty::Medium => (7.0, 10.0),
            AiDifficulty::Hard => (3.0, 5.0),
        }
    }
}

/// Calculate shot parameters for the AI.
pub fn calculate_ai_shot(
    shooter: &Tank,
    target: &Tank,
    wind: &Wind,
    _terrain: &Heightmap,
    difficulty: AiDifficulty,
    rng: &mut impl Rng,
) -> ShotParams {
    let dx = target.position.x - shooter.position.x;
    let dy = target.position.y - shooter.position.y;
    let distance = dx.abs();

    // Estimate a good angle using simplified ballistics (ignore drag/wind for estimate)
    // For a projectile: range = v^2 * sin(2*theta) / g
    // We want to find angle and power that land near the target.

    // Start with 45 degrees (optimal range angle) and adjust
    let base_angle: f32 = if distance < 100.0 {
        60.0 // High arc for close targets
    } else if distance > 500.0 {
        30.0 // Flatter for far targets
    } else {
        45.0
    };

    // Estimate power needed
    // range = v^2 * sin(2*angle) / g, so v = sqrt(range * g / sin(2*angle))
    let angle_rad = base_angle.to_radians();
    let sin2a = (2.0 * angle_rad).sin();
    let needed_v = if sin2a > 0.01 {
        ((distance * GRAVITY) / sin2a).sqrt()
    } else {
        MAX_VELOCITY * 0.5
    };

    // Account for height difference (rough adjustment)
    let height_factor = if dy > 0.0 { 1.1 } else { 0.9 };
    let adjusted_v = needed_v * height_factor;

    // Wind compensation (rough)
    let wind_compensation = -wind.speed * 0.5; // Lean into the wind
    let angle_wind_adjust = wind_compensation;

    let base_power = ((adjusted_v / MAX_VELOCITY) * 100.0).clamp(10.0, 100.0);
    let base_angle_adjusted = (base_angle + angle_wind_adjust).clamp(5.0, 85.0);

    // Apply difficulty-based error
    let (angle_error, power_error) = difficulty.aim_error();
    let angle = (base_angle_adjusted + rng.gen_range(-angle_error..angle_error)).clamp(5.0, 85.0);
    let power = (base_power + rng.gen_range(-power_error..power_error)).clamp(10.0, 100.0);

    // AI ammo selection: prefer explosive at distance, cannonball when close
    let ammo = if distance < 80.0 {
        AmmoType::Cannonball
    } else {
        AmmoType::Explosive
    };

    ShotParams { angle, power, ammo }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn ai_produces_valid_params() {
        let mut rng = StdRng::seed_from_u64(42);
        let terrain = Heightmap::new(vec![50.0; 800], 0.0);
        let shooter = Tank::new(0, "AI".into(), Vec2::new(100.0, 50.0), true);
        let target = Tank::new(1, "Player".into(), Vec2::new(600.0, 50.0), false);
        let wind = Wind { speed: 2.0 };

        for _ in 0..100 {
            let params = calculate_ai_shot(
                &shooter,
                &target,
                &wind,
                &terrain,
                AiDifficulty::Medium,
                &mut rng,
            );
            assert!(params.angle >= 5.0 && params.angle <= 85.0);
            assert!(params.power >= 10.0 && params.power <= 100.0);
        }
    }

    #[test]
    fn hard_ai_is_more_consistent() {
        let terrain = Heightmap::new(vec![50.0; 800], 0.0);
        let shooter = Tank::new(0, "AI".into(), Vec2::new(100.0, 50.0), true);
        let target = Tank::new(1, "Player".into(), Vec2::new(400.0, 50.0), false);
        let wind = Wind { speed: 0.0 };

        let measure_variance = |diff: AiDifficulty| {
            let mut rng = StdRng::seed_from_u64(99);
            let angles: Vec<f32> = (0..50)
                .map(|_| {
                    calculate_ai_shot(&shooter, &target, &wind, &terrain, diff, &mut rng).angle
                })
                .collect();
            let mean = angles.iter().sum::<f32>() / angles.len() as f32;
            angles.iter().map(|a| (a - mean).powi(2)).sum::<f32>() / angles.len() as f32
        };

        let easy_var = measure_variance(AiDifficulty::Easy);
        let hard_var = measure_variance(AiDifficulty::Hard);
        assert!(
            hard_var < easy_var,
            "Hard AI should be more consistent than Easy"
        );
    }
}
