use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::constants::*;
use super::types::*;

/// The current phase of the game — drives the entire game flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GamePhase {
    /// Player is adjusting angle/power/ammo.
    Aiming,
    /// Projectile is in flight (animated).
    Firing {
        trail: Vec<Vec2>,
    },
    /// Damage is being applied, terrain deformed.
    Resolving {
        impact: Vec2,
        damages: Vec<DamageResult>,
    },
    /// Brief pause before next player's turn.
    TurnTransition,
    /// Game is over.
    GameOver {
        winner_id: PlayerId,
    },
}

/// Full game state — everything needed to represent a game in progress.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub tanks: Vec<Tank>,
    pub current_player: PlayerId,
    pub wind: Wind,
    pub phase: GamePhase,
    pub turn_number: u32,
    pub shot_params: ShotParams,
    /// Remaining movement budget for the current turn.
    pub move_budget: f32,
}

impl GameState {
    pub fn current_tank(&self) -> &Tank {
        &self.tanks[self.current_player]
    }

    pub fn current_tank_mut(&mut self) -> &mut Tank {
        &mut self.tanks[self.current_player]
    }

    pub fn opponent_tank(&self) -> &Tank {
        let opponent_id = 1 - self.current_player;
        &self.tanks[opponent_id]
    }

    /// Check if the current player's tank faces left (opponent is to their left).
    pub fn current_faces_left(&self) -> bool {
        self.opponent_tank().position.x < self.current_tank().position.x
    }

    /// Save current shot params to the current tank, then advance turn.
    pub fn advance_turn(&mut self) {
        // Save current player's shot params
        self.tanks[self.current_player].last_shot_params = self.shot_params;
        // Switch player
        self.current_player = 1 - self.current_player;
        self.turn_number += 1;
        // Restore next player's shot params
        self.shot_params = self.tanks[self.current_player].last_shot_params;
        self.move_budget = TANK_MOVE_BUDGET;
        self.phase = GamePhase::Aiming;
    }

    /// Check if the game is over (only one tank alive).
    pub fn check_game_over(&self) -> Option<PlayerId> {
        let alive: Vec<_> = self.tanks.iter().filter(|t| t.is_alive()).collect();
        if alive.len() == 1 {
            Some(alive[0].id)
        } else if alive.is_empty() {
            // Draw — last shooter "wins" (same as Scorched Earth)
            Some(self.current_player)
        } else {
            None
        }
    }
}
