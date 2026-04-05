use glam::Vec2;
use rand::Rng;

use super::constants::*;
use super::types::*;

/// Apply WoT-style RNG to shot parameters before firing.
pub fn apply_accuracy_rng(params: ShotParams, rng: &mut impl Rng) -> ShotParams {
    let angle_offset = rng.gen_range(-ACCURACY_ANGLE_DEVIATION..=ACCURACY_ANGLE_DEVIATION);
    let power_factor = 1.0 + rng.gen_range(-ACCURACY_POWER_DEVIATION..=ACCURACY_POWER_DEVIATION);

    ShotParams {
        angle: (params.angle + angle_offset).clamp(0.0, 90.0),
        power: (params.power * power_factor).clamp(1.0, 100.0),
        ammo: params.ammo,
    }
}

/// Calculate damage dealt to a tank given the impact point.
pub fn calculate_damage(
    impact: Vec2,
    target: &Tank,
    ammo: AmmoType,
    rng: &mut impl Rng,
) -> Option<DamageResult> {
    let distance = impact.distance(target.position);
    let splash_radius = ammo.splash_radius();

    if distance > splash_radius {
        return None;
    }

    let is_direct_hit = distance <= DIRECT_HIT_TOLERANCE;

    // Base damage with distance falloff (linear within splash radius)
    let falloff = if is_direct_hit {
        1.0
    } else {
        1.0 - (distance / splash_radius)
    };

    let mut damage = ammo.base_damage() * falloff;

    // Direct hit multiplier
    if is_direct_hit {
        damage *= ammo.direct_hit_multiplier();
    }

    // WoT-style damage spread: +/- 25%
    let spread = rng.gen_range(1.0 - DAMAGE_SPREAD..=1.0 + DAMAGE_SPREAD);
    damage *= spread;

    // Critical hit check
    let is_critical = rng.r#gen::<f32>() < CRITICAL_HIT_CHANCE;
    if is_critical {
        damage *= CRITICAL_HIT_MULTIPLIER;
    }

    Some(DamageResult {
        target_id: target.id,
        damage,
        is_critical,
        is_direct_hit,
        distance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn test_rng() -> StdRng {
        StdRng::seed_from_u64(42)
    }

    #[test]
    fn direct_hit_cannonball() {
        let mut rng = test_rng();
        let impact = Vec2::new(100.0, 50.0);
        let tank = Tank::new(0, "Target".into(), Vec2::new(100.0, 50.0), false);

        let result = calculate_damage(impact, &tank, AmmoType::Cannonball, &mut rng).unwrap();
        assert!(result.is_direct_hit);
        // Base 50 * 2.0 direct hit * spread — should be roughly 75-125
        assert!(result.damage > 50.0);
        assert!(result.damage < 200.0);
    }

    #[test]
    fn miss_cannonball_outside_radius() {
        let mut rng = test_rng();
        let impact = Vec2::new(100.0, 50.0);
        let tank = Tank::new(0, "Target".into(), Vec2::new(200.0, 50.0), false);

        let result = calculate_damage(impact, &tank, AmmoType::Cannonball, &mut rng);
        assert!(result.is_none());
    }

    #[test]
    fn explosive_splash_damage() {
        let mut rng = test_rng();
        let impact = Vec2::new(100.0, 50.0);
        let tank = Tank::new(0, "Target".into(), Vec2::new(120.0, 50.0), false);

        let result = calculate_damage(impact, &tank, AmmoType::Explosive, &mut rng).unwrap();
        assert!(!result.is_direct_hit);
        // Within splash radius (30), distance ~20, falloff = 1 - 20/30 = 0.33
        assert!(result.damage > 0.0);
        assert!(result.damage < 25.0); // Less than base due to falloff
    }

    #[test]
    fn explosive_outside_splash() {
        let mut rng = test_rng();
        let impact = Vec2::new(100.0, 50.0);
        let tank = Tank::new(0, "Target".into(), Vec2::new(140.0, 50.0), false);

        let result = calculate_damage(impact, &tank, AmmoType::Explosive, &mut rng);
        assert!(result.is_none()); // 40 units away > 30 splash radius
    }

    #[test]
    fn accuracy_rng_stays_in_bounds() {
        let mut rng = test_rng();
        let params = ShotParams {
            angle: 1.0,
            power: 2.0,
            ammo: AmmoType::Cannonball,
        };

        for _ in 0..1000 {
            let adjusted = apply_accuracy_rng(params, &mut rng);
            assert!(adjusted.angle >= 0.0 && adjusted.angle <= 90.0);
            assert!(adjusted.power >= 1.0 && adjusted.power <= 100.0);
        }
    }
}
