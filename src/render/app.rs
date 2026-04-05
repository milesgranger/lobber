use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use glam::Vec2;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use ratatui::prelude::*;

use crate::ai::{calculate_ai_shot, AiDifficulty};
use crate::game::constants::*;
use crate::game::damage::*;
use crate::game::state::*;
use crate::game::types::*;
use crate::physics::projectile::*;
use crate::render::animation::TrajectoryAnimation;
use crate::render::battlefield;
use crate::render::hud::render_hud;
use crate::terrain::{generate_terrain, Heightmap};

/// Top-level application state.
pub struct App {
    pub game: GameState,
    pub terrain: Heightmap,
    pub rng: StdRng,
    pub animation: Option<TrajectoryAnimation>,
    pub should_quit: bool,
    pub resolve_timer: Option<Instant>,
    pub ai_difficulty: AiDifficulty,
    pub impact_flash: Option<(Vec2, Instant)>,
}

impl App {
    pub fn new() -> Self {
        let mut rng = StdRng::from_entropy();
        let terrain = generate_terrain(WORLD_WIDTH as usize, &mut rng);

        // Place tanks on the terrain, roughly 1/5 and 4/5 across
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

        Self {
            game,
            terrain,
            rng,
            animation: None,
            should_quit: false,
            resolve_timer: None,
            ai_difficulty: AiDifficulty::Medium,
            impact_flash: None,
        }
    }

    /// Handle a single frame: input, update, render.
    pub fn handle_input(&mut self) {
        if !event::poll(Duration::from_millis(FRAME_DURATION_MS)).unwrap_or(false) {
            return;
        }

        let Event::Key(key) = event::read().unwrap() else {
            return;
        };

        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
                return;
            }
            _ => {}
        }

        if !matches!(self.game.phase, GamePhase::Aiming) || self.game.current_tank().is_ai {
            // During non-aiming phases or AI turns, only quit works
            if matches!(self.game.phase, GamePhase::GameOver { .. }) {
                if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                    self.should_quit = true;
                }
            }
            return;
        }

        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                self.game.shot_params.angle = (self.game.shot_params.angle + 1.0).min(90.0);
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.game.shot_params.angle = (self.game.shot_params.angle - 1.0).max(0.0);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.game.shot_params.power = (self.game.shot_params.power + 2.0).min(100.0);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.game.shot_params.power = (self.game.shot_params.power - 2.0).max(1.0);
            }
            KeyCode::Tab => {
                self.game.shot_params.ammo = match self.game.shot_params.ammo {
                    AmmoType::Cannonball => AmmoType::Explosive,
                    AmmoType::Explosive => AmmoType::Cannonball,
                };
                self.game.current_tank_mut().last_shot_params.ammo = self.game.shot_params.ammo;
            }
            KeyCode::Char('a') => {
                self.move_tank(-TANK_MOVE_STEP);
            }
            KeyCode::Char('d') => {
                self.move_tank(TANK_MOVE_STEP);
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                self.fire_shot();
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        match &self.game.phase {
            GamePhase::Aiming => {
                if self.game.current_tank().is_ai {
                    self.do_ai_turn();
                }
            }
            GamePhase::Firing { .. } => {
                if let Some(ref mut anim) = self.animation {
                    if !anim.advance() {
                        self.resolve_impact();
                    }
                }
            }
            GamePhase::Resolving { .. } => {
                if let Some(timer) = self.resolve_timer {
                    if timer.elapsed() > Duration::from_millis(1500) {
                        self.finish_turn();
                    }
                }
            }
            GamePhase::TurnTransition => {
                // Brief pause already handled by resolve timer
            }
            GamePhase::GameOver { .. } => {}
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
        let start = tank.position + Vec2::new(0.0, 5.0); // Barrel offset
        let facing_left = self.game.current_faces_left();
        let velocity = params.to_velocity(MAX_VELOCITY, facing_left);

        let (trail, outcome) =
            simulate_shot(start, velocity, params.ammo, &self.game.wind, &self.terrain);

        self.animation = Some(TrajectoryAnimation::new(trail.clone()));
        self.game.phase = GamePhase::Firing { trail };

        // Pre-compute what will happen on impact (used when animation completes)
        let _ = outcome; // outcome is handled in resolve_impact via animation end position
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

        // Apply terrain crater
        self.terrain.apply_crater(impact_pos.x, ammo.crater_radius());

        // Calculate damage to all tanks
        let damages: Vec<DamageResult> = self
            .game
            .tanks
            .iter()
            .filter_map(|tank| calculate_damage(impact_pos, tank, ammo, &mut self.rng))
            .collect();

        // Apply damage
        for d in &damages {
            self.game.tanks[d.target_id].apply_damage(d.damage);
        }

        // Update tank positions after terrain deformation (tanks settle to terrain)
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
            // New wind each turn
            self.game.wind.speed = self.rng.gen_range(-MAX_WIND..MAX_WIND);
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(4)])
            .split(frame.area());

        battlefield::render_battlefield(
            chunks[0],
            frame.buffer_mut(),
            &self.terrain,
            &self.game,
            &self.animation,
            &self.impact_flash,
        );
        render_hud(frame, chunks[1], &self.game);
    }
}
