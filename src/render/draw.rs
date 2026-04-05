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

// ── Coordinate mapping ──────────────────────────────────────────────────

struct ScreenMap {
    sx: f32, // screen pixels per world unit (x)
    sy: f32, // screen pixels per world unit (y)
    sh: f32, // full screen height
}

impl ScreenMap {
    fn new() -> Self {
        let sw = screen_width();
        let sh = screen_height();
        Self {
            sx: sw / WORLD_WIDTH,
            sy: sh / WORLD_CEILING,
            sh,
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
        draw_rectangle(
            0.0,
            i as f32 * strip_h,
            screen_width(),
            strip_h + 1.0,
            color,
        );
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
        draw_line(
            sm.x(wx),
            sm.y(h1),
            sm.x(wx + step),
            sm.y(h2),
            2.0,
            TERRAIN_GRASS,
        );
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
        let th = sm.scale_y(6.0); // body height
        let turret_r = sm.scale_x(4.0);

        // Treads
        let tread_h = sm.scale_y(3.0);
        draw_rectangle(cx - tw / 2.0, cy - tread_h, tw, tread_h, dark);
        // Tread detail lines
        let tread_count = 5;
        for i in 0..tread_count {
            let tx = cx - tw / 2.0 + (i as f32 + 0.5) * tw / tread_count as f32;
            draw_line(
                tx,
                cy,
                tx,
                cy - tread_h,
                1.0,
                Color::new(0.0, 0.0, 0.0, 0.3),
            );
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
        let barrel_angle =
            if tank.id == game.current_player && matches!(game.phase, GamePhase::Aiming) {
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

        // Small name tag above tank
        let is_current = tank.id == game.current_player;
        let label = if is_current {
            format!("\u{25bc} {}", tank.name)
        } else {
            tank.name.clone()
        };
        let font_size = 14.0;
        let dims = measure_text(&label, None, font_size as u16, 1.0);
        let label_x = cx - dims.width / 2.0;
        let label_y = turret_cy - turret_r - 10.0;
        draw_text(
            &label,
            label_x,
            label_y,
            font_size,
            Color::new(body.r, body.g, body.b, 0.7),
        );
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
    let v_lo = crate::game::types::ShotParams {
        angle: min_angle,
        power: min_power,
        ammo,
    }
    .to_velocity(MAX_VELOCITY, facing_left);
    let v_hi = crate::game::types::ShotParams {
        angle: max_angle,
        power: max_power,
        ammo,
    }
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

        draw_line(
            sm.x(lo1.x),
            sm.y(lo1.y),
            sm.x(lo2.x),
            sm.y(lo2.y),
            1.0,
            edge,
        );
        draw_line(
            sm.x(hi1.x),
            sm.y(hi1.y),
            sm.x(hi2.x),
            sm.y(hi2.y),
            1.0,
            edge,
        );
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

fn draw_impact_flash(flash: &Option<(Vec2, std::time::Instant)>, game: &GameState, sm: &ScreenMap) {
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

// ── HUD (overlay-based) ─────────────────────────────────────────────────

fn draw_hud(game: &GameState, _sm: &ScreenMap) {
    let sw = screen_width();
    let sh = screen_height();
    let cx = sw / 2.0;

    draw_health_bars(game, sw);
    draw_top_center_info(game, cx);
    draw_status_message(game, cx, sh);

    // Bottom overlay: shot params + controls hint (aiming only)
    if matches!(game.phase, GamePhase::Aiming) && !game.current_tank().is_ai {
        draw_aiming_overlay(game, cx, sh);
    }
}

/// Health bars in top corners.
fn draw_health_bars(game: &GameState, sw: f32) {
    let pad = 16.0;
    let bar_w = 140.0;
    let bar_h = 10.0;
    let fs = 16.0;
    let small = 13.0;

    for tank in &game.tanks {
        let is_left = tank.id == 0;
        let color = if tank.id == 0 { PLAYER_BODY } else { CPU_BODY };
        let is_current = tank.id == game.current_player;

        let x = if is_left { pad } else { sw - pad - bar_w };

        // Background panel
        let panel_w = bar_w + 16.0;
        let panel_x = x - 8.0;
        draw_rectangle(panel_x, pad - 4.0, panel_w, 42.0, color_u8!(0, 0, 0, 140));
        draw_rectangle_lines(
            panel_x,
            pad - 4.0,
            panel_w,
            42.0,
            1.0,
            Color::new(color.r, color.g, color.b, 0.3),
        );

        // Name
        let marker = if is_current { "\u{25b6} " } else { "" };
        let label = format!("{}{}", marker, tank.name);
        draw_text(&label, x, pad + 12.0, fs, color);

        // Health bar
        let bar_y = pad + 20.0;
        let pct = tank.health / tank.max_health;
        let hc = if pct > 0.5 {
            GREEN
        } else if pct > 0.25 {
            YELLOW
        } else {
            RED
        };
        draw_rectangle(x, bar_y, bar_w, bar_h, color_u8!(30, 30, 30, 200));
        draw_rectangle(x, bar_y, bar_w * pct, bar_h, hc);
        draw_rectangle_lines(x, bar_y, bar_w, bar_h, 1.0, color_u8!(80, 80, 80, 150));

        // HP text
        let hp_text = format!("{:.0}/{:.0}", tank.health, tank.max_health);
        let hp_dims = measure_text(&hp_text, None, small as u16, 1.0);
        draw_text(
            &hp_text,
            x + bar_w / 2.0 - hp_dims.width / 2.0,
            bar_y + bar_h - 1.0,
            small,
            WHITE,
        );
    }
}

/// Wind + turn indicator at top center.
fn draw_top_center_info(game: &GameState, cx: f32) {
    let pad = 16.0;
    let small = 14.0;

    // Background pill
    let info_w = 200.0;
    draw_rectangle(
        cx - info_w / 2.0,
        pad - 4.0,
        info_w,
        32.0,
        color_u8!(0, 0, 0, 120),
    );
    draw_rectangle_lines(
        cx - info_w / 2.0,
        pad - 4.0,
        info_w,
        32.0,
        1.0,
        color_u8!(60, 60, 80, 100),
    );

    // Wind
    let wind_text = format!("Wind: {:.1} {}", game.wind.speed, game.wind.display_arrow());
    let wind_dims = measure_text(&wind_text, None, small as u16, 1.0);
    let wind_color = if game.wind.speed.abs() > 3.0 {
        color_u8!(255, 180, 80, 255) // strong wind = orange
    } else {
        LIGHTGRAY
    };
    draw_text(
        &wind_text,
        cx - wind_dims.width / 2.0,
        pad + 12.0,
        small,
        wind_color,
    );

    // Turn
    let turn_text = format!("Turn {}", game.turn_number);
    let turn_dims = measure_text(&turn_text, None, 12, 1.0);
    draw_text(
        &turn_text,
        cx - turn_dims.width / 2.0,
        pad + 24.0,
        12.0,
        GRAY,
    );
}

/// Large centered status messages (Hit!/Miss!/Game Over).
fn draw_status_message(game: &GameState, cx: f32, sh: f32) {
    let center_y = sh * 0.4;

    match &game.phase {
        GamePhase::Firing { .. } => {
            // Subtle "firing" text — not too distracting
            let text = "Firing...";
            let dims = measure_text(text, None, 20, 1.0);
            draw_text(
                text,
                cx - dims.width / 2.0,
                60.0,
                20.0,
                Color::new(1.0, 1.0, 1.0, 0.4),
            );
        }
        GamePhase::Resolving { damages, .. } => {
            if damages.is_empty() {
                let text = "MISS";
                let dims = measure_text(text, None, 48, 1.0);
                // Background
                draw_rectangle(
                    cx - dims.width / 2.0 - 16.0,
                    center_y - 40.0,
                    dims.width + 32.0,
                    56.0,
                    color_u8!(0, 0, 0, 150),
                );
                draw_text(
                    text,
                    cx - dims.width / 2.0,
                    center_y,
                    48.0,
                    color_u8!(180, 180, 180, 255),
                );
            } else {
                for (i, d) in damages.iter().enumerate() {
                    let crit = if d.is_critical { "  CRITICAL!" } else { "" };
                    let direct = if d.is_direct_hit {
                        "DIRECT HIT!"
                    } else {
                        "HIT!"
                    };
                    let text = format!("{} {:.0} dmg{}", direct, d.damage, crit);

                    let size = if d.is_critical {
                        44.0
                    } else if d.is_direct_hit {
                        40.0
                    } else {
                        36.0
                    };
                    let color = if d.is_critical {
                        GOLD
                    } else if d.is_direct_hit {
                        color_u8!(255, 100, 100, 255)
                    } else {
                        ORANGE
                    };

                    let y = center_y + i as f32 * 50.0;
                    let dims = measure_text(&text, None, size as u16, 1.0);
                    draw_rectangle(
                        cx - dims.width / 2.0 - 12.0,
                        y - dims.height - 8.0,
                        dims.width + 24.0,
                        dims.height + 16.0,
                        color_u8!(0, 0, 0, 160),
                    );
                    draw_text(&text, cx - dims.width / 2.0, y, size, color);
                }
            }
        }
        GamePhase::GameOver { winner_id } => {
            let winner = &game.tanks[*winner_id];
            let color = if winner.id == 0 {
                PLAYER_BODY
            } else {
                CPU_BODY
            };

            let text = format!("{} WINS!", winner.name.to_uppercase());
            let dims = measure_text(&text, None, 56, 1.0);
            draw_rectangle(
                cx - dims.width / 2.0 - 20.0,
                center_y - 50.0,
                dims.width + 40.0,
                80.0,
                color_u8!(0, 0, 0, 180),
            );
            draw_text(&text, cx - dims.width / 2.0, center_y, 56.0, color);

            let sub = "Press Space to exit";
            let sub_dims = measure_text(sub, None, 20, 1.0);
            let blink = (get_time() * 2.0).sin() * 0.5 + 0.5;
            draw_text(
                sub,
                cx - sub_dims.width / 2.0,
                center_y + 30.0,
                20.0,
                Color::new(1.0, 1.0, 1.0, blink as f32),
            );
        }
        _ => {}
    }
}

/// Translucent bottom overlay with shot params and controls (aiming phase only).
fn draw_aiming_overlay(game: &GameState, cx: f32, sh: f32) {
    let fs = 17.0;
    let small = 13.0;

    // Semi-transparent panel at bottom
    let panel_h = 52.0;
    let panel_y = sh - panel_h;
    draw_rectangle(
        0.0,
        panel_y,
        screen_width(),
        panel_h,
        color_u8!(0, 0, 0, 120),
    );

    // Shot parameters — centered
    let ammo_name = game.shot_params.ammo.display_name();
    let ammo_color = match game.shot_params.ammo {
        crate::game::types::AmmoType::Cannonball => color_u8!(100, 160, 255, 255),
        crate::game::types::AmmoType::Explosive => color_u8!(255, 160, 80, 255),
    };

    // Lay out params with spacing
    let params = [
        (
            format!("Angle: {:.0}\u{00b0}", game.shot_params.angle),
            WHITE,
        ),
        (format!("Power: {:.0}%", game.shot_params.power), WHITE),
        (format!("Ammo: {}", ammo_name), ammo_color),
        (
            format!("Move: {:.0}", game.move_budget),
            if game.move_budget > 0.0 {
                WHITE
            } else {
                DARKGRAY
            },
        ),
    ];

    let spacing = 24.0;
    let total_w: f32 = params
        .iter()
        .map(|(t, _)| measure_text(t, None, fs as u16, 1.0).width + spacing)
        .sum::<f32>()
        - spacing;
    let mut px = cx - total_w / 2.0;
    let py = panel_y + 18.0;

    for (text, color) in &params {
        draw_text(text, px, py, fs, *color);
        px += measure_text(text, None, fs as u16, 1.0).width + spacing;
    }

    // Controls hint — dim, below params
    let hint = "h/\u{2190} l/\u{2192}:Angle   k/\u{2191} j/\u{2193}:Power   a/d:Move   Tab:Ammo   Space:Fire   Esc:Quit";
    let hint_dims = measure_text(hint, None, small as u16, 1.0);
    draw_text(
        hint,
        cx - hint_dims.width / 2.0,
        panel_y + 38.0,
        small,
        color_u8!(100, 100, 100, 180),
    );
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
    draw_text(
        author,
        cx - auth_dims.width / 2.0,
        sh * 0.37,
        18.0,
        LIGHTGRAY,
    );

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
    draw_text(
        prompt,
        cx - prompt_dims.width / 2.0,
        sh * 0.82,
        22.0,
        prompt_color,
    );
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
