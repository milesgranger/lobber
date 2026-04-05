use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::game::state::*;
use crate::game::types::*;

/// Render the HUD (heads-up display) panel showing game controls and status.
pub fn render_hud(frame: &mut Frame, area: Rect, game: &GameState) {
    let tank = game.current_tank();

    let ammo_indicator = |a: AmmoType, selected: AmmoType| {
        if a == selected {
            format!("[{}]", a.display_name())
        } else {
            format!(" {} ", a.display_name())
        }
    };

    let status_line = match &game.phase {
        GamePhase::Aiming => {
            if tank.is_ai {
                "AI is thinking...".to_string()
            } else {
                format!(
                    "Angle: {angle:.0}\u{00b0}  |  Power: {power:.0}%  |  Ammo: {cb}{ex}  |  Wind: {wind} {arrow}  |  Move: {move_left:.0}",
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
            format!("{} wins!  (press Space to exit)", game.tanks[*winner_id].name)
        }
    };

    let p1 = &game.tanks[0];
    let p2 = &game.tanks[1];
    let p1_marker = if game.current_player == 0 { "\u{25b6}" } else { " " };
    let p2_marker = if game.current_player == 1 { "\u{25b6}" } else { " " };

    let lines = vec![
        Line::from(vec![
            Span::styled(format!("{p1_marker}{}: ", p1.name), Style::default().fg(Color::Cyan).bold()),
            health_bar_spans(p1),
            Span::raw("    "),
            Span::styled(format!("{p2_marker}{}: ", p2.name), Style::default().fg(Color::Red).bold()),
            health_bar_spans(p2),
            Span::raw(format!("    Turn {}", game.turn_number)),
        ]),
        Line::from(Span::raw(&status_line)),
        Line::from(Span::styled(
            "h/\u{2190} l/\u{2192}:Angle  k/\u{2191} j/\u{2193}:Power  a/d:Move  Tab:Ammo  Space:Fire  q:Quit",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let hud = Paragraph::new(lines)
        .block(Block::default().borders(Borders::TOP).title(" LOBBER "))
        .style(Style::default().fg(Color::White));

    frame.render_widget(hud, area);
}

fn health_bar_spans(tank: &Tank) -> Span<'static> {
    let pct = tank.health / tank.max_health;
    let bar_w = 10;
    let filled = (pct * bar_w as f32).ceil() as usize;
    let color = if pct > 0.5 {
        Color::Green
    } else if pct > 0.25 {
        Color::Yellow
    } else {
        Color::Red
    };
    let bar = format!(
        "{}{} {:.0}",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(bar_w - filled),
        tank.health,
    );
    Span::styled(bar, Style::default().fg(color))
}
