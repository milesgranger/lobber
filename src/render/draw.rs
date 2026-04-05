use std::time::Duration;

use ::glam::Vec2;
use macroquad::prelude::*;

use crate::game::constants::*;
use crate::game::state::*;
use crate::terrain::Heightmap;

use super::app::App;

// ── Colors ──────────────────────────────────────────────────────────────

const SKY_TOP: Color = color_u8!(8, 8, 30, 255);
const SKY_BOTTOM: Color = color_u8!(25, 35, 80, 255);
const TERRAIN_GRASS: Color = color_u8!(50, 150, 50, 255);
const TERRAIN_EARTH: Color = color_u8!(80, 60, 30, 255);
const TERRAIN_ROCK: Color = color_u8!(100, 90, 70, 255);
const PLAYER_BODY: Color = color_u8!(0, 190, 220, 255);
const PLAYER_DARK: Color = color_u8!(0, 120, 150, 255);
const CPU_BODY: Color = color_u8!(220, 55, 55, 255);
const CPU_DARK: Color = color_u8!(155, 30, 30, 255);
const HUD_BG: Color = color_u8!(0, 0, 0, 180);

// ── Coordinate mapping ──────────────────────────────────────────────────

struct ScreenMap {
    sx: f32, // screen pixels per world unit (x)
    sy: f32, // screen pixels per world unit (y)
    sh: f32, // screen height (for y flip)
    hud_h: f32,
}

impl ScreenMap {
    fn new() -> Self {
        let sw = screen_width();
        let sh = screen_height();
        let hud_h = 60.0;
        let play_h = sh - hud_h;
        Self {
            sx: sw / WORLD_WIDTH,
            sy: play_h / WORLD_CEILING,
            sh: play_h,
            hud_h,
        }
    }

    fn x(&self, wx: f32) -> f32 {
        wx * self.sx
    }

    fn y(&self, wy: f32) -> f32 {
        self.sh - wy * self.sy
    }

    fn scale_x(&self, w: f32) -> f32 {
        w * self.sx
    }

    fn scale_y(&self, h: f32) -> f32 {
        h * self.sy
    }
}

// ── Main draw entry ─────────────────────────────────────────────────────

pub fn draw_frame(app: &App) {
    let sm = ScreenMap::new();

    draw_sky(&sm);
    draw_wind_particles(&app.wind_particles, &sm);
    draw_terrain(&app.terrain, &sm);
    draw_landing_zone(&app.game, &app.terrain, &sm);
    draw_projectile(&app.animation, &sm);
    draw_impact_flash(&app.impact_flash, &app.game, &sm);
    draw_tanks(&app.game, &sm);
    draw_hud(&app.game, &sm);
}

// ── Sky ─────────────────────────────────────────────────────────────────

fn draw_sky(sm: &ScreenMap) {
    clear_background(SKY_TOP);
    // Gradient: draw horizontal strips
    let steps = 32;
    let strip_h = sm.sh / steps as f32;
    for i in 0..steps {
        let t = i as f32 / steps as f32;
        let color = lerp_color(SKY_TOP, SKY_BOTTOM, t);
        draw_rectangle(0.0, i as f32 * strip_h, screen_width(), strip_h + 1.0, color);
    }
}

// ── Wind particles ──────────────────────────────────────────────────────

fn draw_wind_particles(particles: &[super::app::WindParticle], sm: &ScreenMap) {
    for p in particles {
        let sx = sm.x(p.x);
        let sy = sm.y(p.y);
        let r = p.size * sm.sx;
        // Cloud-like: multiple overlapping soft circles
        let cloud_color = Color::new(0.7, 0.7, 0.8, p.alpha);
        draw_circle(sx, sy, r, cloud_color);
        draw_circle(sx + r * 0.6, sy - r * 0.2, r * 0.7, cloud_color);
        draw_circle(sx - r * 0.5, sy + r * 0.15, r * 0.6, cloud_color);
        draw_circle(sx + r * 0.2, sy + r * 0.3, r * 0.5, cloud_color);
    }
}

// ── Terrain ─────────────────────────────────────────────────────────────

fn draw_terrain(terrain: &Heightmap, sm: &ScreenMap) {
    let sw = screen_width();
    let cols = sw as usize;

    // Draw terrain columns
    for px in 0..cols {
        let world_x = px as f32 / sm.sx;
        let terrain_h = terrain.height_at(world_x);
        let surface_y = sm.y(terrain_h);
        let bottom_y = sm.sh;

        if surface_y >= bottom_y {
            continue;
        }

        let total_h = bottom_y - surface_y;

        // Grass layer (top ~15%)
        let grass_h = (total_h * 0.15).max(2.0);
        draw_rectangle(px as f32, surface_y, 1.0, grass_h, TERRAIN_GRASS);

        // Earth layer (next ~50%)
        let earth_start = surface_y + grass_h;
        let earth_h = total_h * 0.5;
        draw_rectangle(px as f32, earth_start, 1.0, earth_h, TERRAIN_EARTH);

        // Rock layer (bottom)
        let rock_start = earth_start + earth_h;
        let rock_h = bottom_y - rock_start;
        if rock_h > 0.0 {
            draw_rectangle(px as f32, rock_start, 1.0, rock_h, TERRAIN_ROCK);
        }
    }

    // Smooth surface line on top
    let step = 2.0;
    let mut wx = 0.0_f32;
    while wx < WORLD_WIDTH - step {
        let h1 = terrain.height_at(wx);
        let h2 = terrain.height_at(wx + step);
        draw_line(sm.x(wx), sm.y(h1), sm.x(wx + step), sm.y(h2), 2.0, TERRAIN_GRASS);
        wx += step;
    }
}

// ── Tanks ───────────────────────────────────────────────────────────────

fn draw_tanks(game: &GameState, sm: &ScreenMap) {
    for tank in &game.tanks {
        if !tank.is_alive() {
            continue;
        }

        let (body, dark) = if tank.id == 0 {
            (PLAYER_BODY, PLAYER_DARK)
        } else {
            (CPU_BODY, CPU_DARK)
        };

        let cx = sm.x(tank.position.x);
        let cy = sm.y(tank.position.y);

        let tw = sm.scale_x(14.0); // tank width in screen pixels
        let th = sm.scale_y(6.0);  // body height
        let turret_r = sm.scale_x(4.0);

        // Treads
        let tread_h = sm.scale_y(3.0);
        draw_rectangle(cx - tw / 2.0, cy - tread_h, tw, tread_h, dark);
        // Tread detail lines
        let tread_count = 5;
        for i in 0..tread_count {
            let tx = cx - tw / 2.0 + (i as f32 + 0.5) * tw / tread_count as f32;
            draw_line(tx, cy, tx, cy - tread_h, 1.0, Color::new(0.0, 0.0, 0.0, 0.3));
        }

        // Hull
        let hull_y = cy - tread_h;
        draw_rectangle(cx - tw * 0.4, hull_y - th, tw * 0.8, th, body);

        // Turret dome
        let turret_cy = hull_y - th - turret_r * 0.5;
        draw_circle(cx, turret_cy, turret_r, body);

        // Barrel
        let faces_left = if game.tanks.len() == 2 {
            tank.position.x > game.tanks[1 - tank.id].position.x
        } else {
            tank.position.x > WORLD_WIDTH / 2.0
        };

        // During aiming for current player, show barrel at aim angle
        let barrel_angle = if tank.id == game.current_player && matches!(game.phase, GamePhase::Aiming) {
            game.shot_params.angle
        } else {
            tank.last_shot_params.angle
        };

        let angle_rad = barrel_angle.to_radians();
        let dir: f32 = if faces_left { -1.0 } else { 1.0 };
        let barrel_len = sm.scale_x(18.0);
        let bx_end = cx + angle_rad.cos() * barrel_len * dir;
        let by_end = turret_cy - angle_rad.sin() * barrel_len;
        draw_line(cx, turret_cy, bx_end, by_end, 3.0, body);

        // Name label
        let is_current = tank.id == game.current_player;
        let label = if is_current {
            format!("\u{25bc} {}", tank.name)
        } else {
            tank.name.clone()
        };
        let font_size = 18.0;
        let dims = measure_text(&label, None, font_size as u16, 1.0);
        let label_x = cx - dims.width / 2.0;
        let label_y = turret_cy - turret_r - 24.0;

        // Label background
        draw_rectangle(
            label_x - 4.0,
            label_y - dims.height - 2.0,
            dims.width + 8.0,
            dims.height + 6.0,
            HUD_BG,
        );
        draw_text(&label, label_x, label_y, font_size, body);

        // Health bar
        let bar_w = 50.0;
        let bar_h = 5.0;
        let bar_x = cx - bar_w / 2.0;
        let bar_y = label_y + 4.0;
        let pct = tank.health / tank.max_health;
        let health_color = if pct > 0.5 {
            GREEN
        } else if pct > 0.25 {
            YELLOW
        } else {
            RED
        };

        draw_rectangle(bar_x, bar_y, bar_w, bar_h, color_u8!(40, 40, 40, 200));
        draw_rectangle(bar_x, bar_y, bar_w * pct, bar_h, health_color);
        draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 1.0, color_u8!(120, 120, 120, 200));
    }
}

// ── Trajectory spread cone ──────────────────────────────────────────────

fn draw_landing_zone(game: &GameState, _terrain: &Heightmap, sm: &ScreenMap) {
    use crate::physics::projectile::predict_trajectory;

    if !matches!(game.phase, GamePhase::Aiming) || game.current_tank().is_ai {
        return;
    }

    let tank = game.current_tank();
    let facing_left = game.current_faces_left();
    let start = Vec2::new(tank.position.x, tank.position.y + 5.0);

    let params = game.shot_params;
    let ammo = params.ammo;

    // Build 3 trajectories: nominal center + two edges of the RNG spread
    let min_angle = (params.angle - ACCURACY_ANGLE_DEVIATION).max(0.0);
    let max_angle = (params.angle + ACCURACY_ANGLE_DEVIATION).min(90.0);
    let min_power = params.power * (1.0 - ACCURACY_POWER_DEVIATION);
    let max_power = (params.power * (1.0 + ACCURACY_POWER_DEVIATION)).min(100.0);

    let v_center = params.to_velocity(MAX_VELOCITY, facing_left);
    let v_lo = crate::game::types::ShotParams { angle: min_angle, power: min_power, ammo }
        .to_velocity(MAX_VELOCITY, facing_left);
    let v_hi = crate::game::types::ShotParams { angle: max_angle, power: max_power, ammo }
        .to_velocity(MAX_VELOCITY, facing_left);

    // Only show ~30% of the full trajectory
    let preview_steps = (TRAJECTORY_PREVIEW_STEPS as f32 * 1.1) as usize;

    let path_center = predict_trajectory(start, v_center, ammo, &game.wind, preview_steps);
    let path_lo = predict_trajectory(start, v_lo, ammo, &game.wind, preview_steps);
    let path_hi = predict_trajectory(start, v_hi, ammo, &game.wind, preview_steps);

    if path_center.is_empty() {
        return;
    }

    // Color by ammo type
    let cone_color = match ammo {
        crate::game::types::AmmoType::Cannonball => Color::new(0.3, 0.5, 1.0, 1.0),
        crate::game::types::AmmoType::Explosive => Color::new(1.0, 0.5, 0.2, 1.0),
    };

    let len = path_center.len().min(path_lo.len()).min(path_hi.len());

    // Draw filled cone as triangle strips between lo and hi edges
    for i in 0..len.saturating_sub(1) {
        let t = i as f32 / len as f32;
        let alpha = (1.0 - t) * 0.18; // fades out along the path

        let lo1 = path_lo[i];
        let lo2 = path_lo[i + 1];
        let hi1 = path_hi[i];
        let hi2 = path_hi[i + 1];

        let fill = Color::new(cone_color.r, cone_color.g, cone_color.b, alpha);

        // Two triangles to fill the quad between lo and hi edges
        draw_triangle(
            macroquad::math::Vec2::new(sm.x(lo1.x), sm.y(lo1.y)),
            macroquad::math::Vec2::new(sm.x(hi1.x), sm.y(hi1.y)),
            macroquad::math::Vec2::new(sm.x(lo2.x), sm.y(lo2.y)),
            fill,
        );
        draw_triangle(
            macroquad::math::Vec2::new(sm.x(hi1.x), sm.y(hi1.y)),
            macroquad::math::Vec2::new(sm.x(hi2.x), sm.y(hi2.y)),
            macroquad::math::Vec2::new(sm.x(lo2.x), sm.y(lo2.y)),
            fill,
        );
    }

    // Draw outer edge lines (subtle)
    for i in 0..len.saturating_sub(1) {
        let t = i as f32 / len as f32;
        let alpha = (1.0 - t) * 0.2;
        let edge = Color::new(cone_color.r, cone_color.g, cone_color.b, alpha);

        let lo1 = path_lo[i];
        let lo2 = path_lo[i + 1];
        let hi1 = path_hi[i];
        let hi2 = path_hi[i + 1];

        draw_line(sm.x(lo1.x), sm.y(lo1.y), sm.x(lo2.x), sm.y(lo2.y), 1.0, edge);
        draw_line(sm.x(hi1.x), sm.y(hi1.y), sm.x(hi2.x), sm.y(hi2.y), 1.0, edge);
    }

    // Center line — very faint dashed
    for i in 0..len.saturating_sub(1) {
        if (i / 3) % 2 != 0 {
            continue;
        }
        let t = i as f32 / len as f32;
        let alpha = (1.0 - t) * 0.15;
        let c = Color::new(1.0, 1.0, 1.0, alpha);
        let p1 = path_center[i];
        let p2 = path_center[i + 1];
        draw_line(sm.x(p1.x), sm.y(p1.y), sm.x(p2.x), sm.y(p2.y), 1.0, c);
    }
}

// ── Projectile ──────────────────────────────────────────────────────────

fn draw_projectile(animation: &Option<super::app::TrajectoryAnimation>, sm: &ScreenMap) {
    let Some(anim) = animation else { return };

    // Trail
    let trail = anim.trail();
    for (i, pos) in trail.iter().enumerate() {
        let t = i as f32 / trail.len().max(1) as f32;
        let r = 1.5 + t * 2.0;
        let alpha = t * 0.8;
        let color = Color::new(1.0, 0.7 + t * 0.3, t * 0.3, alpha);
        draw_circle(sm.x(pos.x), sm.y(pos.y), r, color);
    }

    // Projectile head with glow
    let pos = anim.current_position();
    let sx = sm.x(pos.x);
    let sy = sm.y(pos.y);
    draw_circle(sx, sy, 6.0, Color::new(1.0, 0.9, 0.5, 0.4)); // outer glow
    draw_circle(sx, sy, 3.5, Color::new(1.0, 0.95, 0.7, 0.7)); // mid glow
    draw_circle(sx, sy, 2.0, WHITE); // bright core
}

// ── Impact flash ────────────────────────────────────────────────────────

fn draw_impact_flash(
    flash: &Option<(Vec2, std::time::Instant)>,
    game: &GameState,
    sm: &ScreenMap,
) {
    let Some((pos, time)) = flash else { return };
    let elapsed = time.elapsed();
    if elapsed > Duration::from_millis(600) {
        return;
    }

    let progress = elapsed.as_millis() as f32 / 600.0;
    let max_r = sm.scale_x(game.shot_params.ammo.crater_radius());
    let radius = max_r * (0.3 + progress * 0.7);
    let alpha = (1.0 - progress).max(0.0);

    let cx = sm.x(pos.x);
    let cy = sm.y(pos.y);

    // Outer blast
    draw_circle(cx, cy, radius, Color::new(1.0, 0.7, 0.2, alpha * 0.4));
    // Inner flash
    draw_circle(cx, cy, radius * 0.5, Color::new(1.0, 0.9, 0.6, alpha * 0.7));
    // Core
    draw_circle(cx, cy, radius * 0.15, Color::new(1.0, 1.0, 0.9, alpha));
}

// ── HUD ─────────────────────────────────────────────────────────────────

fn draw_hud(game: &GameState, sm: &ScreenMap) {
    let sw = screen_width();
    let hud_y = sm.sh;

    // HUD background
    draw_rectangle(0.0, hud_y, sw, sm.hud_h, color_u8!(15, 15, 25, 240));
    draw_line(0.0, hud_y, sw, hud_y, 2.0, color_u8!(60, 60, 80, 255));

    let fs = 18.0;
    let small = 14.0;
    let y1 = hud_y + 20.0;
    let y2 = hud_y + 42.0;

    // Player health bars
    for tank in &game.tanks {
        let (color, x_pos) = if tank.id == 0 {
            (PLAYER_BODY, 20.0)
        } else {
            (CPU_BODY, sw - 220.0)
        };

        let is_current = tank.id == game.current_player;
        let marker = if is_current { "\u{25b6} " } else { "  " };
        let label = format!("{}{}", marker, tank.name);
        draw_text(&label, x_pos, y1, fs, color);

        let bar_x = x_pos;
        let bar_y = y1 + 6.0;
        let bar_w = 120.0;
        let bar_h = 8.0;
        let pct = tank.health / tank.max_health;
        let hc = if pct > 0.5 { GREEN } else if pct > 0.25 { YELLOW } else { RED };
        draw_rectangle(bar_x, bar_y, bar_w, bar_h, color_u8!(40, 40, 40, 255));
        draw_rectangle(bar_x, bar_y, bar_w * pct, bar_h, hc);
        draw_text(
            &format!("{:.0} HP", tank.health),
            bar_x + bar_w + 8.0,
            bar_y + bar_h,
            small,
            LIGHTGRAY,
        );
    }

    // Center: shot info
    let center_x = sw / 2.0;

    let status = match &game.phase {
        GamePhase::Aiming if game.current_tank().is_ai => "AI thinking...".to_string(),
        GamePhase::Aiming => {
            let ammo_name = game.shot_params.ammo.display_name();
            format!(
                "Angle: {:.0}\u{00b0}   Power: {:.0}%   Ammo: {}   Wind: {:.1} {}   Move: {:.0}",
                game.shot_params.angle,
                game.shot_params.power,
                ammo_name,
                game.wind.speed,
                game.wind.display_arrow(),
                game.move_budget,
            )
        }
        GamePhase::Firing { .. } => "Firing...".to_string(),
        GamePhase::Resolving { damages, .. } => {
            if damages.is_empty() {
                "Miss!".to_string()
            } else {
                damages
                    .iter()
                    .map(|d| {
                        let crit = if d.is_critical { " CRIT!" } else { "" };
                        let direct = if d.is_direct_hit { " Direct!" } else { "" };
                        format!("{:.0} dmg{}{}", d.damage, direct, crit)
                    })
                    .collect::<Vec<_>>()
                    .join("  |  ")
            }
        }
        GamePhase::TurnTransition => "Next turn...".to_string(),
        GamePhase::GameOver { winner_id } => {
            format!("{} wins!  [Space to exit]", game.tanks[*winner_id].name)
        }
    };

    let dims = measure_text(&status, None, fs as u16, 1.0);
    draw_text(&status, center_x - dims.width / 2.0, y1, fs, WHITE);

    // Controls hint
    let hint = "h/\u{2190} l/\u{2192}:Angle   k/\u{2191} j/\u{2193}:Power   a/d:Move   Tab:Ammo   Space:Fire   Esc:Quit";
    let hint_dims = measure_text(hint, None, small as u16, 1.0);
    draw_text(hint, center_x - hint_dims.width / 2.0, y2, small, DARKGRAY);

    // Turn counter
    let turn_text = format!("Turn {}", game.turn_number);
    let turn_dims = measure_text(&turn_text, None, small as u16, 1.0);
    draw_text(&turn_text, center_x - turn_dims.width / 2.0, y2 + 14.0, small, GRAY);
}

// ── Title screen ────────────────────────────────────────────────────────

pub fn draw_title() {
    clear_background(color_u8!(10, 10, 30, 255));

    let sw = screen_width();
    let sh = screen_height();
    let cx = sw / 2.0;

    // Title
    let title = "LOBBER";
    let title_dims = measure_text(title, None, 72, 1.0);
    draw_text(title, cx - title_dims.width / 2.0, sh * 0.25, 72.0, GOLD);

    // Subtitle
    let sub = "An Artillery Game";
    let sub_dims = measure_text(sub, None, 24, 1.0);
    draw_text(sub, cx - sub_dims.width / 2.0, sh * 0.32, 24.0, LIGHTGRAY);

    let author = "by Miles Granger";
    let auth_dims = measure_text(author, None, 18, 1.0);
    draw_text(author, cx - auth_dims.width / 2.0, sh * 0.37, 18.0, LIGHTGRAY);

    let inspired = "Inspired by Scorched Earth (1991)";
    let ins_dims = measure_text(inspired, None, 16, 1.0);
    draw_text(inspired, cx - ins_dims.width / 2.0, sh * 0.42, 16.0, GRAY);

    // Controls
    let controls = [
        ("h/\u{2190}  l/\u{2192}", "Adjust angle"),
        ("k/\u{2191}  j/\u{2193}", "Adjust power"),
        ("a / d", "Move tank"),
        ("Tab", "Switch ammo"),
        ("Space", "Fire!"),
        ("Esc", "Quit"),
    ];

    let start_y = sh * 0.48;
    let key_x = cx - 120.0;
    let desc_x = cx + 10.0;

    for (i, (key, desc)) in controls.iter().enumerate() {
        let y = start_y + i as f32 * 28.0;
        draw_text(key, key_x, y, 20.0, GOLD);
        draw_text(desc, desc_x, y, 20.0, LIGHTGRAY);
    }

    // Prompt
    let prompt = "Press Space to start...";
    let prompt_dims = measure_text(prompt, None, 22, 1.0);
    let blink = (get_time() * 2.0).sin() * 0.5 + 0.5;
    let prompt_color = Color::new(1.0, 1.0, 1.0, blink as f32);
    draw_text(prompt, cx - prompt_dims.width / 2.0, sh * 0.82, 22.0, prompt_color);
}

// ── Helpers ─────────────────────────────────────────────────────────────

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}
