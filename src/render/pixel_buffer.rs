use ratatui::prelude::*;

/// An RGB color for the pixel buffer.
#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const BLACK: Self = Self::new(0, 0, 0);

    /// Linearly interpolate between two colors.
    pub fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
        let t = t.clamp(0.0, 1.0);
        Rgb {
            r: (a.r as f32 + (b.r as f32 - a.r as f32) * t) as u8,
            g: (a.g as f32 + (b.g as f32 - a.g as f32) * t) as u8,
            b: (a.b as f32 + (b.b as f32 - a.b as f32) * t) as u8,
        }
    }

    /// Blend `src` over `dst` with given alpha (0.0 = fully transparent, 1.0 = fully opaque).
    pub fn blend(dst: Rgb, src: Rgb, alpha: f32) -> Rgb {
        Self::lerp(dst, src, alpha)
    }

    fn to_ratatui(self) -> Color {
        Color::Rgb(self.r, self.g, self.b)
    }
}

/// A pixel buffer that renders to terminal cells using halfblock characters.
/// Each terminal cell represents 2 vertical pixels.
pub struct PixelBuffer {
    /// Width in pixels (= terminal columns).
    pub width: usize,
    /// Height in pixels (= terminal rows * 2).
    pub height: usize,
    pixels: Vec<Rgb>,
}

impl PixelBuffer {
    pub fn new(term_cols: usize, term_rows: usize) -> Self {
        let width = term_cols;
        let height = term_rows * 2;
        Self {
            width,
            height,
            pixels: vec![Rgb::BLACK; width * height],
        }
    }

    pub fn clear(&mut self, color: Rgb) {
        self.pixels.fill(color);
    }

    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, color: Rgb) {
        if x < self.width && y < self.height {
            let i = self.idx(x, y);
            self.pixels[i] = color;
        }
    }

    #[inline]
    pub fn blend(&mut self, x: usize, y: usize, color: Rgb, alpha: f32) {
        if x < self.width && y < self.height {
            let i = self.idx(x, y);
            self.pixels[i] = Rgb::blend(self.pixels[i], color, alpha);
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Rgb {
        if x < self.width && y < self.height {
            self.pixels[self.idx(x, y)]
        } else {
            Rgb::BLACK
        }
    }

    /// Fill a vertical column from y_start to y_end (inclusive) with a color.
    pub fn fill_column(&mut self, x: usize, y_start: usize, y_end: usize, color: Rgb) {
        if x >= self.width {
            return;
        }
        let ys = y_start.min(self.height.saturating_sub(1));
        let ye = y_end.min(self.height.saturating_sub(1));
        for y in ys..=ye {
            let i = self.idx(x, y);
            self.pixels[i] = color;
        }
    }

    /// Fill a vertical column with a gradient.
    pub fn fill_column_gradient(
        &mut self,
        x: usize,
        y_start: usize,
        y_end: usize,
        top_color: Rgb,
        bottom_color: Rgb,
    ) {
        if x >= self.width || y_start >= self.height {
            return;
        }
        let ye = y_end.min(self.height.saturating_sub(1));
        let span = (ye - y_start).max(1) as f32;
        for y in y_start..=ye {
            let t = (y - y_start) as f32 / span;
            let color = Rgb::lerp(top_color, bottom_color, t);
            let i = self.idx(x, y);
            self.pixels[i] = color;
        }
    }

    /// Draw a filled circle at pixel coordinates.
    pub fn fill_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Rgb) {
        let r_ceil = radius.ceil() as i32;
        for dy in -r_ceil..=r_ceil {
            for dx in -r_ceil..=r_ceil {
                let px = cx + dx as f32;
                let py = cy + dy as f32;
                let dist = ((dx as f32).powi(2) + (dy as f32).powi(2)).sqrt();
                if dist <= radius {
                    // Anti-alias the edge
                    let alpha = if dist > radius - 1.0 {
                        radius - dist
                    } else {
                        1.0
                    };
                    self.blend(px as usize, py as usize, color, alpha);
                }
            }
        }
    }

    /// Draw a single-pixel-wide line using Bresenham-like stepping.
    pub fn draw_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: Rgb) {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let steps = dx.abs().max(dy.abs()).ceil() as usize;
        if steps == 0 {
            self.blend(x0 as usize, y0 as usize, color, 1.0);
            return;
        }
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = x0 + dx * t;
            let y = y0 + dy * t;
            self.blend(x as usize, y as usize, color, 1.0);
        }
    }

    /// Write into a ratatui Buffer using halfblock (▄) encoding.
    /// The top pixel of each pair becomes the background color,
    /// the bottom pixel becomes the foreground color.
    pub fn render_to_buffer(&self, area: Rect, buf: &mut Buffer) {
        for row in 0..area.height {
            let py_top = (row as usize) * 2;
            let py_bot = py_top + 1;
            for col in 0..area.width {
                let px = col as usize;
                let top = self.get(px, py_top);
                let bot = self.get(px, py_bot);

                let pos = Position::new(area.x + col, area.y + row);
                if let Some(cell) = buf.cell_mut(pos) {
                    cell.set_char('\u{2584}') // ▄
                        .set_fg(bot.to_ratatui())
                        .set_bg(top.to_ratatui());
                }
            }
        }
    }
}
