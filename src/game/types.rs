use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Unique identifier for a player in the game.
pub type PlayerId = usize;

/// Available ammunition types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmmoType {
    /// Heavy projectile: high damage on direct hit, no splash, less affected by wind.
    Cannonball,
    /// Lighter projectile: moderate damage with splash radius, more affected by wind.
    Explosive,
}

impl AmmoType {
    pub fn base_damage(self) -> f32 {
        match self {
            AmmoType::Cannonball => 50.0,
            AmmoType::Explosive => 25.0,
        }
    }

    /// Splash damage radius in world units. Cannonball has a tiny tolerance zone instead of true splash.
    pub fn splash_radius(self) -> f32 {
        match self {
            AmmoType::Cannonball => 12.0,
            AmmoType::Explosive => 30.0,
        }
    }

    /// Multiplier applied when a hit is within the direct-hit tolerance.
    pub fn direct_hit_multiplier(self) -> f32 {
        match self {
            AmmoType::Cannonball => 2.0,
            AmmoType::Explosive => 1.0,
        }
    }

    /// Aerodynamic drag coefficient. Higher = more affected by wind and air resistance.
    pub fn drag_coefficient(self) -> f32 {
        match self {
            AmmoType::Cannonball => 0.00005,
            AmmoType::Explosive => 0.0002,
        }
    }

    /// Crater radius when hitting terrain.
    pub fn crater_radius(self) -> f32 {
        match self {
            AmmoType::Cannonball => 5.0,
            AmmoType::Explosive => 15.0,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            AmmoType::Cannonball => "Cannonball",
            AmmoType::Explosive => "Explosive",
        }
    }
}

/// A tank (player unit) on the battlefield.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tank {
    pub id: PlayerId,
    pub name: String,
    pub position: Vec2,
    pub health: f32,
    pub max_health: f32,
    pub is_ai: bool,
    pub last_shot_params: ShotParams,
}

impl Tank {
    pub fn new(id: PlayerId, name: String, position: Vec2, is_ai: bool) -> Self {
        Self {
            id,
            name,
            position,
            health: 100.0,
            max_health: 100.0,
            is_ai,
            last_shot_params: ShotParams {
                angle: 45.0,
                power: 50.0,
                ammo: AmmoType::Explosive,
            },
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn apply_damage(&mut self, damage: f32) {
        self.health = (self.health - damage).max(0.0);
    }
}

/// Parameters for a shot: angle, power, and ammo type.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ShotParams {
    /// Barrel angle in degrees (0 = horizontal right, 90 = straight up).
    pub angle: f32,
    /// Power as a percentage (0.0 to 100.0).
    pub power: f32,
    /// Selected ammunition.
    pub ammo: AmmoType,
}

impl ShotParams {
    /// Convert angle (degrees) and power (%) to an initial velocity vector.
    /// Power maps to a max velocity of `max_velocity`.
    pub fn to_velocity(self, max_velocity: f32, facing_left: bool) -> Vec2 {
        let radians = self.angle.to_radians();
        let speed = (self.power / 100.0) * max_velocity;
        let direction = if facing_left { -1.0 } else { 1.0 };
        Vec2::new(radians.cos() * speed * direction, radians.sin() * speed)
    }
}

/// Wind state for the current turn.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Wind {
    /// Horizontal wind speed. Positive = blowing right, negative = blowing left.
    pub speed: f32,
}

impl Wind {
    pub fn as_force(self) -> Vec2 {
        Vec2::new(self.speed, 0.0)
    }

    pub fn display_arrow(self) -> &'static str {
        if self.speed > 3.0 {
            ">>>"
        } else if self.speed > 1.0 {
            ">>"
        } else if self.speed > 0.3 {
            ">"
        } else if self.speed < -3.0 {
            "<<<"
        } else if self.speed < -1.0 {
            "<<"
        } else if self.speed < -0.3 {
            "<"
        } else {
            "~"
        }
    }
}

/// Result of a damage calculation for a single target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageResult {
    pub target_id: PlayerId,
    pub damage: f32,
    pub is_critical: bool,
    pub is_direct_hit: bool,
    pub distance: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn shot_velocity_right() {
        let shot = ShotParams {
            angle: 45.0,
            power: 100.0,
            ammo: AmmoType::Cannonball,
        };
        let vel = shot.to_velocity(100.0, false);
        assert_relative_eq!(vel.x, 70.710678, epsilon = 0.01);
        assert_relative_eq!(vel.y, 70.710678, epsilon = 0.01);
    }

    #[test]
    fn shot_velocity_left() {
        let shot = ShotParams {
            angle: 45.0,
            power: 100.0,
            ammo: AmmoType::Cannonball,
        };
        let vel = shot.to_velocity(100.0, true);
        assert_relative_eq!(vel.x, -70.710678, epsilon = 0.01);
        assert_relative_eq!(vel.y, 70.710678, epsilon = 0.01);
    }

    #[test]
    fn shot_velocity_half_power() {
        let shot = ShotParams {
            angle: 90.0,
            power: 50.0,
            ammo: AmmoType::Explosive,
        };
        let vel = shot.to_velocity(100.0, false);
        assert_relative_eq!(vel.x, 0.0, epsilon = 0.01);
        assert_relative_eq!(vel.y, 50.0, epsilon = 0.01);
    }

    #[test]
    fn tank_damage() {
        let mut tank = Tank::new(0, "Test".to_string(), Vec2::ZERO, false);
        assert_eq!(tank.health, 100.0);
        tank.apply_damage(30.0);
        assert_eq!(tank.health, 70.0);
        tank.apply_damage(200.0);
        assert_eq!(tank.health, 0.0);
        assert!(!tank.is_alive());
    }

    #[test]
    fn wind_display() {
        assert_eq!(Wind { speed: 5.0 }.display_arrow(), ">>>");
        assert_eq!(Wind { speed: 2.0 }.display_arrow(), ">>");
        assert_eq!(Wind { speed: 0.5 }.display_arrow(), ">");
        assert_eq!(Wind { speed: 0.0 }.display_arrow(), "~");
        assert_eq!(Wind { speed: -2.0 }.display_arrow(), "<<");
    }
}
