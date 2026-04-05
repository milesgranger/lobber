use std::time::Instant;

use glam::Vec2;
use ratatui::prelude::*;

use crate::game::constants::*;
use crate::game::state::*;
use crate::physics::projectile::predict_trajectory;
use crate::render::animation::TrajectoryAnimation;
use crate::render::pixel_buffer::{PixelBuffer, Rgb};
use crate::terrain::Heightmap;

// --- Color palette ---
const SKY_TOP: Rgb = Rgb::new(10, 10, 35);
const SKY_BOTTOM: Rgb = Rgb::new(30, 30, 70);
const TERRAIN_SURFACE: Rgb = Rgb::new(60, 160, 60);
const TERRAIN_DEEP: Rgb = Rgb::new(30, 80, 20);
const TERRAIN_ROCK: Rgb = Rgb::new(90, 80, 60);
const PLAYER_COLOR: Rgb = Rgb::new(0, 200, 220);
const PLAYER_DARK: Rgb = Rgb::new(0, 130, 150);
const CPU_COLOR: Rgb = Rgb::new(220, 60, 60);
const CPU_DARK: Rgb = Rgb::new(150, 30, 30);
const PROJECTILE: Rgb = Rgb::new(255, 255, 255);
const TRAIL: Rgb = Rgb::new(255, 200, 50);
const PREVIEW: Rgb = Rgb::new(100, 100, 100);
const FLASH_BRIGHT: Rgb = Rgb::new(255, 255, 200);
const FLASH_DIM: Rgb = Rgb::new(255, 180, 50);
const LABEL_BG: Rgb = Rgb::new(0, 0, 0);

/// World-to-pixel coordinate mapping.
struct WorldMap {
    x_scale: f32, // world units per pixel column
    y_scale: f32, // world units per pixel row
    world_h: f32,
}

impl WorldMap {
    fn new(px_w: usize, px_h: usize) -> Self {
        Self {
            x_scale: WORLD_WIDTH / px_w as f32,
            y_scale: WORLD_CEILING / px_h as f32,
            world_h: WORLD_CEILING,
        }
    }

    /// World x -> pixel column.
    fn to_px(&self, wx: f32) -> i32 {
        (wx / self.x_scale) as i32
    }

    /// World y -> pixel row (0 = top).
    fn to_py(&self, wy: f32) -> i32 {
        ((self.world_h - wy) / self.y_scale) as i32
    }

    /// Pixel row -> world y at row center.
    fn row_to_world_y(&self, py: usize) -> f32 {
        self.world_h - (py as f32 + 0.5) * self.y_scale
    }

    /// Pixel col -> world x at column center.
    fn col_to_world_x(&self, px: usize) -> f32 {
        (px as f32 + 0.5) * self.x_scale
    }
}

/// Render the full battlefield into the frame buffer.
pub fn render_battlefield(
    area: Rect,
    buf: &mut Buffer,
    terrain: &Heightmap,
    game: &GameState,
    animation: &Option<TrajectoryAnimation>,
    impact_flash: &Option<(Vec2, Instant)>,
) {
    let px_w = area.width as usize;
    let px_h = area.height as usize * 2; // halfblock = 2 vertical pixels per cell
    if px_w == 0 || px_h == 0 {
        return;
    }

    let wm = WorldMap::new(px_w, px_h);
    let mut pb = PixelBuffer::new(px_w, area.height as usize);

    draw_sky(&mut pb, &wm);
    draw_terrain(&mut pb, terrain, &wm);
    draw_trajectory_preview(&mut pb, game, &wm);
    draw_projectile(&mut pb, animation, &wm);
    draw_impact_flash(&mut pb, impact_flash, game, &wm);
    draw_tanks(&mut pb, game, &wm);
    draw_labels(&mut pb, game, &wm);

    pb.render_to_buffer(area, buf);
}

fn draw_sky(pb: &mut PixelBuffer, _wm: &WorldMap) {
    for y in 0..pb.height {
        let t = y as f32 / pb.height as f32;
        let color = Rgb::lerp(SKY_TOP, SKY_BOTTOM, t);
        for x in 0..pb.width {
            pb.set(x, y, color);
        }
    }
}

fn draw_terrain(pb: &mut PixelBuffer, terrain: &Heightmap, wm: &WorldMap) {
    for px in 0..pb.width {
        let world_x = wm.col_to_world_x(px);
        let terrain_h = terrain.height_at(world_x);
        let surface_py = wm.to_py(terrain_h);

        if surface_py < 0 {
            // Terrain is above viewport
            let depth = pb.height;
            pb.fill_column_gradient(px, 0, depth.saturating_sub(1), TERRAIN_SURFACE, TERRAIN_DEEP);
            continue;
        }

        let surface_py = surface_py as usize;

        // Fill from surface to bottom with terrain gradient
        if surface_py < pb.height {
            let depth = pb.height - surface_py;
            let gradient_end = pb.height.saturating_sub(1);

            // Surface zone: green grass (first few pixels)
            let grass_depth = (depth / 4).max(2).min(8);
            let grass_end = (surface_py + grass_depth).min(gradient_end);
            pb.fill_column_gradient(px, surface_py, grass_end, TERRAIN_SURFACE, TERRAIN_DEEP);

            // Sub-surface: darker with rock tones
            if grass_end < gradient_end {
                pb.fill_column_gradient(px, grass_end, gradient_end, TERRAIN_DEEP, TERRAIN_ROCK);
            }

            // Anti-alias surface edge: blend the pixel right above terrain
            if surface_py > 0 {
                let frac = (surface_py as f32 - wm.to_py(terrain_h) as f32).fract();
                pb.blend(px, surface_py.saturating_sub(1), TERRAIN_SURFACE, frac * 0.5);
            }
        }
    }
}

fn draw_trajectory_preview(pb: &mut PixelBuffer, game: &GameState, wm: &WorldMap) {
    if !matches!(game.phase, GamePhase::Aiming) || game.current_tank().is_ai {
        return;
    }

    let tank = game.current_tank();
    let facing_left = game.current_faces_left();
    let start = Vec2::new(tank.position.x, tank.position.y + 5.0);
    let velocity = game.shot_params.to_velocity(MAX_VELOCITY, facing_left);

    let preview = predict_trajectory(
        start,
        velocity,
        game.shot_params.ammo,
        &game.wind,
        TRAJECTORY_PREVIEW_STEPS,
    );

    // Draw dotted preview (every 3rd point to create dashed look)
    for (i, pos) in preview.iter().enumerate() {
        if (i / 2) % 2 != 0 {
            continue; // dash pattern
        }
        let px = wm.to_px(pos.x);
        let py = wm.to_py(pos.y);
        if px >= 0 && px < pb.width as i32 && py >= 0 && py < pb.height as i32 {
            // Fade out toward the end
            let alpha = 1.0 - (i as f32 / TRAJECTORY_PREVIEW_STEPS as f32) * 0.7;
            pb.blend(px as usize, py as usize, PREVIEW, alpha);
        }
    }
}

fn draw_projectile(pb: &mut PixelBuffer, animation: &Option<TrajectoryAnimation>, wm: &WorldMap) {
    let Some(anim) = animation else { return };

    // Draw trail with fade
    let trail = anim.trail();
    for (i, pos) in trail.iter().enumerate() {
        let px = wm.to_px(pos.x);
        let py = wm.to_py(pos.y);
        if px >= 0 && px < pb.width as i32 && py >= 0 && py < pb.height as i32 {
            let t = i as f32 / trail.len().max(1) as f32;
            let alpha = t * 0.8;
            let color = Rgb::lerp(Rgb::new(150, 100, 0), TRAIL, t);
            pb.fill_circle(px as f32, py as f32, 1.0, color);
            // Glow
            pb.blend(px as usize, py as usize, color, alpha);
        }
    }

    // Draw projectile head with glow
    let pos = anim.current_position();
    let px = wm.to_px(pos.x) as f32;
    let py = wm.to_py(pos.y) as f32;
    // Outer glow
    pb.fill_circle(px, py, 3.0, Rgb::new(255, 200, 100));
    // Inner bright
    pb.fill_circle(px, py, 1.5, PROJECTILE);
}

fn draw_impact_flash(
    pb: &mut PixelBuffer,
    impact_flash: &Option<(Vec2, Instant)>,
    game: &GameState,
    wm: &WorldMap,
) {
    let Some((pos, time)) = impact_flash else {
        return;
    };
    let elapsed_ms = time.elapsed().as_millis() as f32;
    if elapsed_ms > 600.0 {
        return;
    }

    let progress = elapsed_ms / 600.0;
    let max_radius = game.shot_params.ammo.crater_radius();
    let radius_world = max_radius * (0.3 + progress * 0.7);
    let radius_px = (radius_world / wm.x_scale).max(2.0);

    let cx = wm.to_px(pos.x) as f32;
    let cy = wm.to_py(pos.y) as f32;
    let alpha = 1.0 - progress;
    let color = Rgb::lerp(FLASH_BRIGHT, FLASH_DIM, progress);

    pb.fill_circle(cx, cy, radius_px, color);
    // Bright center
    pb.fill_circle(cx, cy, radius_px * 0.3, Rgb::blend(FLASH_BRIGHT, color, alpha));
}

fn draw_tanks(pb: &mut PixelBuffer, game: &GameState, wm: &WorldMap) {
    for tank in &game.tanks {
        if !tank.is_alive() {
            continue;
        }

        let (body, dark) = if tank.id == 0 {
            (PLAYER_COLOR, PLAYER_DARK)
        } else {
            (CPU_COLOR, CPU_DARK)
        };

        let cx = wm.to_px(tank.position.x) as f32;
        let cy = wm.to_py(tank.position.y) as f32;

        // Treads (wide, low)
        let tread_w = 7.0;
        let tread_h = 2.0;
        for dy in 0..tread_h as i32 {
            for dx in -(tread_w as i32)..=tread_w as i32 {
                let px = (cx + dx as f32) as usize;
                let py = (cy + dy as f32) as usize;
                pb.blend(px, py, dark, 0.95);
            }
        }

        // Hull (slightly narrower)
        let hull_w = 5.0;
        let hull_h = 3.0;
        for dy in 0..hull_h as i32 {
            for dx in -(hull_w as i32)..=hull_w as i32 {
                let px = (cx + dx as f32) as usize;
                let py = (cy - 1.0 - dy as f32) as usize;
                pb.blend(px, py, body, 0.95);
            }
        }

        // Turret (small dome)
        pb.fill_circle(cx, cy - hull_h - 1.5, 3.0, body);

        // Barrel
        let faces_left = if game.tanks.len() == 2 {
            tank.position.x > game.tanks[1 - tank.id].position.x
        } else {
            tank.position.x > WORLD_WIDTH / 2.0
        };
        let dir: f32 = if faces_left { -1.0 } else { 1.0 };
        let barrel_len = 8.0;
        let bx = cx + dir * 3.0;
        let by = cy - hull_h - 1.5;
        pb.draw_line(bx, by, bx + dir * barrel_len, by, body);
        pb.draw_line(bx, by - 1.0, bx + dir * barrel_len, by - 1.0, body);
    }
}

fn draw_labels(pb: &mut PixelBuffer, game: &GameState, wm: &WorldMap) {
    for tank in &game.tanks {
        if !tank.is_alive() {
            continue;
        }

        let color = if tank.id == 0 { PLAYER_COLOR } else { CPU_COLOR };
        let is_current = tank.id == game.current_player;

        let cx = wm.to_px(tank.position.x);
        let cy = wm.to_py(tank.position.y);

        // Name tag above tank: render as small colored bar
        let label_y = (cy - 12).max(2) as usize;
        let label_half_w = 8;
        let label_x_start = (cx - label_half_w).max(0) as usize;
        let label_x_end = (cx + label_half_w).min(pb.width as i32 - 1) as usize;

        // Background strip for label
        for x in label_x_start..=label_x_end {
            pb.blend(x, label_y, LABEL_BG, 0.6);
            pb.blend(x, label_y + 1, LABEL_BG, 0.6);
        }

        // Colored indicator line
        let indicator_color = if is_current { color } else { Rgb::lerp(color, Rgb::new(80, 80, 80), 0.5) };
        for x in label_x_start..=label_x_end {
            pb.set(x, label_y, indicator_color);
        }

        // Health bar below indicator
        let health_pct = tank.health / tank.max_health;
        let bar_w = (label_x_end - label_x_start) as f32;
        let filled_w = (health_pct * bar_w) as usize;
        let bar_y = label_y + 1;
        let health_color = if health_pct > 0.5 {
            Rgb::new(50, 200, 50)
        } else if health_pct > 0.25 {
            Rgb::new(220, 200, 30)
        } else {
            Rgb::new(220, 40, 40)
        };
        for dx in 0..filled_w {
            pb.set(label_x_start + dx, bar_y, health_color);
        }
    }
}
