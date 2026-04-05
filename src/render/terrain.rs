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
