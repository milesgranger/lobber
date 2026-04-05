use ratatui::prelude::*;
use ratatui::widgets::canvas::{Painter, Shape};

use crate::terrain::Heightmap;

/// Shape that renders the terrain as a filled area on the canvas.
pub struct TerrainShape<'a> {
    pub heightmap: &'a Heightmap,
    pub color: Color,
}

impl Shape for TerrainShape<'_> {
    fn draw(&self, painter: &mut Painter<'_, '_>) {
        let width = self.heightmap.width();
        for x in 0..width {
            let height = self.heightmap.height_at_index(x);
            // Fill from y=0 up to the terrain height
            let fx = x as f64;
            let mut y = 0.0;
            while y <= height as f64 {
                if let Some((px, py)) = painter.get_point(fx, y) {
                    painter.paint(px, py, self.color);
                }
                y += 0.5; // Sub-pixel stepping for solid fill
            }
        }
    }
}

/// Shape that renders a tank with a recognizable profile.
/// Draws treads (base), hull (body), and turret.
pub struct TankShape {
    pub x: f64,
    pub y: f64,
    pub color: Color,
    pub facing_left: bool,
}

impl Shape for TankShape {
    fn draw(&self, painter: &mut Painter<'_, '_>) {
        // Treads: wide flat base
        for dx in -6..=6 {
            for dy in 0..=2 {
                let px = self.x + dx as f64;
                let py = self.y + dy as f64;
                if let Some((sx, sy)) = painter.get_point(px, py) {
                    painter.paint(sx, sy, self.color);
                }
            }
        }

        // Hull: narrower body on top of treads
        for dx in -5..=5 {
            for dy in 3..=6 {
                let px = self.x + dx as f64;
                let py = self.y + dy as f64;
                if let Some((sx, sy)) = painter.get_point(px, py) {
                    painter.paint(sx, sy, self.color);
                }
            }
        }

        // Turret: small raised section
        for dx in -3..=3 {
            for dy in 7..=10 {
                let px = self.x + dx as f64;
                let py = self.y + dy as f64;
                if let Some((sx, sy)) = painter.get_point(px, py) {
                    painter.paint(sx, sy, self.color);
                }
            }
        }

        // Barrel: extends from turret in facing direction
        let barrel_dir: f64 = if self.facing_left { -1.0 } else { 1.0 };
        for i in 0..=10 {
            let px = self.x + 3.0 * barrel_dir + i as f64 * barrel_dir;
            let py = self.y + 9.0;
            if let Some((sx, sy)) = painter.get_point(px, py) {
                painter.paint(sx, sy, self.color);
            }
            // Make barrel 2 pixels thick
            if let Some((sx, sy)) = painter.get_point(px, py + 1.0) {
                painter.paint(sx, sy, self.color);
            }
        }
    }
}

/// Shape that renders a crater flash effect (brief visual feedback on impact).
pub struct CraterFlash {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
    pub color: Color,
}

impl Shape for CraterFlash {
    fn draw(&self, painter: &mut Painter<'_, '_>) {
        let steps = (self.radius * 4.0) as i32;
        for dx in -steps..=steps {
            for dy in -steps..=steps {
                let px = self.x + dx as f64 / 4.0;
                let py = self.y + dy as f64 / 4.0;
                let dist = ((px - self.x).powi(2) + (py - self.y).powi(2)).sqrt();
                if dist <= self.radius {
                    if let Some((sx, sy)) = painter.get_point(px, py) {
                        painter.paint(sx, sy, self.color);
                    }
                }
            }
        }
    }
}
