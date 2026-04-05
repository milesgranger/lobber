/// Game world constants.

/// Gravity acceleration (world units per second squared).
pub const GRAVITY: f32 = 98.0;

/// Maximum projectile velocity at 100% power.
pub const MAX_VELOCITY: f32 = 350.0;

/// Physics simulation timestep in seconds.
pub const PHYSICS_DT: f32 = 1.0 / 60.0;

/// Default world width in units.
pub const WORLD_WIDTH: f32 = 800.0;

/// Minimum terrain height (valley floor).
pub const TERRAIN_MIN_HEIGHT: f32 = 50.0;

/// Maximum terrain height (mountain peak).
pub const TERRAIN_MAX_HEIGHT: f32 = 350.0;

/// Height above which a projectile is considered out of bounds.
pub const WORLD_CEILING: f32 = 600.0;

/// Tolerance distance for a "direct hit" on a tank.
pub const DIRECT_HIT_TOLERANCE: f32 = 5.0;

/// Maximum number of physics steps per shot (safety limit).
pub const MAX_PHYSICS_STEPS: usize = 10_000;

// --- RNG Parameters (WoT-style) ---

/// Damage spread: base_damage * uniform(1 - SPREAD, 1 + SPREAD).
pub const DAMAGE_SPREAD: f32 = 0.25;

/// Accuracy deviation: angle offset in degrees.
pub const ACCURACY_ANGLE_DEVIATION: f32 = 1.5;

/// Accuracy deviation: power offset as a fraction (0.03 = 3%).
pub const ACCURACY_POWER_DEVIATION: f32 = 0.03;

/// Critical hit probability (0.0 to 1.0).
pub const CRITICAL_HIT_CHANCE: f32 = 0.05;

/// Critical hit damage multiplier.
pub const CRITICAL_HIT_MULTIPLIER: f32 = 1.5;

/// Wind speed range: uniform(-MAX_WIND, MAX_WIND).
pub const MAX_WIND: f32 = 5.0;

// --- Movement ---

/// How far a tank can move per turn (world units).
pub const TANK_MOVE_BUDGET: f32 = 30.0;

/// How many units each keypress moves the tank.
pub const TANK_MOVE_STEP: f32 = 3.0;

// --- Rendering ---

/// Target frames per second for TUI rendering.
pub const TARGET_FPS: u32 = 30;

/// Frame duration in milliseconds.
pub const FRAME_DURATION_MS: u64 = 1000 / TARGET_FPS as u64;

/// Projectile trail length (number of past positions to render).
pub const TRAIL_LENGTH: usize = 15;

/// Number of physics steps to show in the aiming trajectory preview.
pub const TRAJECTORY_PREVIEW_STEPS: usize = 80;
