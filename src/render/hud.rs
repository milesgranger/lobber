use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::game::state::*;
use crate::game::types::*;

/// Render the HUD (heads-up display) panel showing game controls and status.
pub fn render_hud(frame: &mut Frame, area: Rect, game: &GameState) {
    let tank = game.current_tank();
    let _opponent = game.opponent_tank();

    let ammo_indicator = |a: AmmoType, selected: AmmoType| {
        if a == selected {
            format!("[{}]", a.display_name())
        } else {
            format!(" {} ", a.display_name())
        }
    };

    let controls_text = match &game.phase {
        GamePhase::Aiming => {
            if tank.is_ai {
                "AI is thinking...".to_string()
            } else {
                format!(
                    "Turn {turn}  |  {name}  |  Angle: {angle:.0}\u{00b0}  |  Power: {power:.0}%  |  Ammo: {cb} {ex}  |  Wind: {wind} {arrow}  |  Move: {move_left:.0}\n\
                     h/\u{2190} l/\u{2192}: Angle  k/\u{2191} j/\u{2193}: Power  a/d: Move  Tab: Ammo  Space: Fire  Q: Quit",
                    turn = game.turn_number,
                    name = tank.name,
                    angle = game.shot_params.angle,
                    power = game.shot_params.power,
                    cb = ammo_indicator(AmmoType::Cannonball, game.shot_params.ammo),
                    ex = ammo_indicator(AmmoType::Explosive, game.shot_params.ammo),
                    wind = format!("{:.1}", game.wind.speed),
                    arrow = game.wind.display_arrow(),
                    move_left = game.move_budget,
                )
            }
        }
        GamePhase::Firing { .. } => "Firing...".to_string(),
        GamePhase::Resolving { damages, .. } => {
            if damages.is_empty() {
                "Miss!".to_string()
            } else {
                damages
                    .iter()
                    .map(|d| {
                        let crit = if d.is_critical { " CRITICAL!" } else { "" };
                        let direct = if d.is_direct_hit { " Direct hit!" } else { "" };
                        format!("Hit! {:.0} damage{}{}", d.damage, direct, crit)
                    })
                    .collect::<Vec<_>>()
                    .join("  ")
            }
        }
        GamePhase::TurnTransition => "Next turn...".to_string(),
        GamePhase::GameOver { winner_id } => {
            format!("{} wins!", game.tanks[*winner_id].name)
        }
    };

    // Health bars
    let p1_health = format!(
        "{}: {:.0}/{:.0} HP",
        game.tanks[0].name, game.tanks[0].health, game.tanks[0].max_health
    );
    let p2_health = format!(
        "{}: {:.0}/{:.0} HP",
        game.tanks[1].name, game.tanks[1].health, game.tanks[1].max_health
    );

    let health_line = format!("{}    {}", p1_health, p2_health);

    let text = format!("{}\n{}", health_line, controls_text);

    let hud = Paragraph::new(text)
        .block(Block::default().borders(Borders::TOP).title(" LOBBER "))
        .style(Style::default().fg(Color::White));

    frame.render_widget(hud, area);
}
