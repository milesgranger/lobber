use std::time::Instant;

use ::glam::Vec2;
use macroquad::prelude::*;
use ::rand::rngs::StdRng;
use ::rand::{Rng, SeedableRng};

use crate::ai::{calculate_ai_shot, AiDifficulty};
use crate::game::constants::*;
use crate::game::damage::*;
use crate::game::state::*;
use crate::game::types::*;
use crate::physics::projectile::*;
use crate::terrain::{generate_terrain, Heightmap};

use super::draw;

/// Trajectory animation state for projectile playback.
pub struct TrajectoryAnimation {
    positions: Vec<Vec2>,
    current_index: usize,
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

    pub fn current_position(&self) -> Vec2 {
        self.positions[self.current_index]
    }

    pub fn trail(&self) -> &[Vec2] {
        let start = self.current_index.saturating_sub(TRAIL_LENGTH);
        &self.positions[start..=self.current_index]
    }
}

/// A single wind particle (cloud wisp) in the sky.
pub struct WindParticle {
    pub x: f32,     // world x
    pub y: f32,     // world y (upper portion of sky)
    pub size: f32,  // radius
    pub alpha: f32, // opacity
}

/// Top-level application state.
pub struct App {
    pub game: GameState,
    pub terrain: Heightmap,
    pub rng: StdRng,
    pub animation: Option<TrajectoryAnimation>,
    pub should_quit: bool,
    pub show_title: bool,
    pub resolve_timer: Option<Instant>,
    pub ai_difficulty: AiDifficulty,
    pub impact_flash: Option<(Vec2, Instant)>,
    pub wind_particles: Vec<WindParticle>,
}

impl App {
    pub fn new() -> Self {
        let mut rng = StdRng::from_entropy();
        let terrain = generate_terrain(WORLD_WIDTH as usize, &mut rng);

        let p1_x = WORLD_WIDTH * 0.15 + rng.gen_range(-30.0..30.0);
        let p2_x = WORLD_WIDTH * 0.85 + rng.gen_range(-30.0..30.0);
        let p1_y = terrain.height_at(p1_x) + 2.0;
        let p2_y = terrain.height_at(p2_x) + 2.0;

        let tanks = vec![
            Tank::new(0, "Player".to_string(), Vec2::new(p1_x, p1_y), false),
            Tank::new(1, "CPU".to_string(), Vec2::new(p2_x, p2_y), true),
        ];

        let wind = Wind {
            speed: rng.gen_range(-MAX_WIND..MAX_WIND),
        };

        let game = GameState {
            tanks,
            current_player: 0,
            wind,
            phase: GamePhase::Aiming,
            turn_number: 1,
            shot_params: ShotParams {
                angle: 45.0,
                power: 50.0,
                ammo: AmmoType::Explosive,
            },
            move_budget: TANK_MOVE_BUDGET,
        };

        let wind_particles = Self::spawn_initial_particles(&mut rng);

        Self {
            game,
            terrain,
            rng,
            animation: None,
            should_quit: false,
            show_title: true,
            resolve_timer: None,
            ai_difficulty: AiDifficulty::Medium,
            impact_flash: None,
            wind_particles,
        }
    }

    fn spawn_initial_particles(rng: &mut StdRng) -> Vec<WindParticle> {
        (0..30)
            .map(|_| WindParticle {
                x: rng.gen_range(0.0..WORLD_WIDTH),
                y: rng.gen_range(WORLD_CEILING * 0.55..WORLD_CEILING * 0.98),
                size: rng.gen_range(6.0..25.0),
                alpha: rng.gen_range(0.05..0.2),
            })
            .collect()
    }

    pub fn handle_input(&mut self) {
        if (is_key_pressed(KeyCode::Q) || is_key_pressed(KeyCode::Escape))
            && (matches!(self.game.phase, GamePhase::GameOver { .. })
                || is_key_pressed(KeyCode::Escape))
        {
            self.should_quit = true;
            return;
        }

        if matches!(self.game.phase, GamePhase::GameOver { .. }) {
            if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                self.should_quit = true;
            }
            return;
        }

        if !matches!(self.game.phase, GamePhase::Aiming) || self.game.current_tank().is_ai {
            return;
        }

        // Held keys for smooth adjustment
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::H) {
            self.game.shot_params.angle = (self.game.shot_params.angle + 0.5).min(90.0);
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::L) {
            self.game.shot_params.angle = (self.game.shot_params.angle - 0.5).max(0.0);
        }
        if is_key_down(KeyCode::Up) || is_key_down(KeyCode::K) {
            self.game.shot_params.power = (self.game.shot_params.power + 0.5).min(100.0);
        }
        if is_key_down(KeyCode::Down) || is_key_down(KeyCode::J) {
            self.game.shot_params.power = (self.game.shot_params.power - 0.5).max(1.0);
        }
        if is_key_down(KeyCode::A) {
            self.move_tank(-TANK_MOVE_STEP * 0.3);
        }
        if is_key_down(KeyCode::D) {
            self.move_tank(TANK_MOVE_STEP * 0.3);
        }

        // Single-press actions
        if is_key_pressed(KeyCode::Tab) {
            self.game.shot_params.ammo = match self.game.shot_params.ammo {
                AmmoType::Cannonball => AmmoType::Explosive,
                AmmoType::Explosive => AmmoType::Cannonball,
            };
            self.game.current_tank_mut().last_shot_params.ammo = self.game.shot_params.ammo;
        }
        if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
            self.fire_shot();
        }
    }

    pub fn update(&mut self) {
        self.update_wind_particles();

        match &self.game.phase {
            GamePhase::Aiming => {
                if self.game.current_tank().is_ai {
                    self.do_ai_turn();
                }
            }
            GamePhase::Firing { .. } => {
                if let Some(ref mut anim) = self.animation
                    && !anim.advance()
                {
                    self.resolve_impact();
                }
            }
            GamePhase::Resolving { .. } => {
                if let Some(timer) = self.resolve_timer
                    && timer.elapsed() > std::time::Duration::from_millis(1500)
                {
                    self.finish_turn();
                }
            }
            GamePhase::TurnTransition => {}
            GamePhase::GameOver { .. } => {}
        }
    }

    fn update_wind_particles(&mut self) {
        let dt = get_frame_time();
        let wind_speed = self.game.wind.speed;
        // Particles drift at wind speed, scaled up for visual effect
        let drift = wind_speed * 8.0 * dt;

        for p in &mut self.wind_particles {
            p.x += drift;
            // Add slight vertical wobble
            p.y += (get_time() as f32 * 0.3 + p.x * 0.01).sin() * 2.0 * dt;

            // Wrap horizontally
            if p.x > WORLD_WIDTH + 50.0 {
                p.x = -50.0;
                p.y = self.rng.gen_range(WORLD_CEILING * 0.55..WORLD_CEILING * 0.98);
                p.alpha = self.rng.gen_range(0.05..0.2);
            } else if p.x < -50.0 {
                p.x = WORLD_WIDTH + 50.0;
                p.y = self.rng.gen_range(WORLD_CEILING * 0.55..WORLD_CEILING * 0.98);
                p.alpha = self.rng.gen_range(0.05..0.2);
            }
        }
    }

    fn move_tank(&mut self, dx: f32) {
        if self.game.move_budget <= 0.0 {
            return;
        }
        let step = dx.abs().min(self.game.move_budget);
        let actual_dx = step * dx.signum();

        let tank = self.game.current_tank_mut();
        let new_x = (tank.position.x + actual_dx).clamp(10.0, WORLD_WIDTH - 10.0);
        tank.position.x = new_x;
        tank.position.y = self.terrain.height_at(new_x) + 2.0;
        self.game.move_budget -= step;
    }

    fn fire_shot(&mut self) {
        let params = apply_accuracy_rng(self.game.shot_params, &mut self.rng);
        let tank = self.game.current_tank();
        let start = tank.position + Vec2::new(0.0, 5.0);
        let facing_left = self.game.current_faces_left();
        let velocity = params.to_velocity(MAX_VELOCITY, facing_left);

        let (trail, _outcome) =
            simulate_shot(start, velocity, params.ammo, &self.game.wind, &self.terrain);

        self.animation = Some(TrajectoryAnimation::new(trail.clone()));
        self.game.phase = GamePhase::Firing { trail };
    }

    fn do_ai_turn(&mut self) {
        let shooter = self.game.current_tank().clone();
        let target = self.game.opponent_tank().clone();
        let params = calculate_ai_shot(
            &shooter,
            &target,
            &self.game.wind,
            &self.terrain,
            self.ai_difficulty,
            &mut self.rng,
        );
        self.game.shot_params = params;
        self.fire_shot();
    }

    fn resolve_impact(&mut self) {
        let impact_pos = self
            .animation
            .as_ref()
            .map(|a| a.current_position())
            .unwrap_or(Vec2::ZERO);

        let ammo = self.game.shot_params.ammo;
        self.terrain.apply_crater(impact_pos.x, ammo.crater_radius());

        let damages: Vec<DamageResult> = self
            .game
            .tanks
            .iter()
            .filter_map(|tank| calculate_damage(impact_pos, tank, ammo, &mut self.rng))
            .collect();

        for d in &damages {
            self.game.tanks[d.target_id].apply_damage(d.damage);
        }

        for tank in &mut self.game.tanks {
            let terrain_y = self.terrain.height_at(tank.position.x);
            tank.position.y = terrain_y + 2.0;
        }

        self.impact_flash = Some((impact_pos, Instant::now()));
        self.game.phase = GamePhase::Resolving {
            impact: impact_pos,
            damages,
        };
        self.resolve_timer = Some(Instant::now());
        self.animation = None;
    }

    fn finish_turn(&mut self) {
        self.resolve_timer = None;
        self.impact_flash = None;

        if let Some(winner) = self.game.check_game_over() {
            self.game.phase = GamePhase::GameOver { winner_id: winner };
        } else {
            self.game.advance_turn();
            self.game.wind.speed = self.rng.gen_range(-MAX_WIND..MAX_WIND);
        }
    }

    pub fn render(&self) {
        draw::draw_frame(self);
    }

    pub fn draw_title_screen(&self) {
        draw::draw_title();
    }
}
