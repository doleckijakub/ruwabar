use std::sync::Arc;
use std::sync::Mutex;

use fontdue::Font;

use crate::modules::*;

#[derive(Debug)]
pub struct Canvas {
    width: u32,
    height: u32,
    offset: u32, // in pixels, not bytes
    stride: u32, // in pixels, not bytes

    pub(crate) pixels: Arc<Mutex<Vec<u32>>>,

    background_color: u32,
}

#[allow(dead_code)]
impl Canvas {
    pub fn new(width: u32, height: u32, background_color: u32) -> Self {
        Self {
            width,
            height,
            offset: 0,
            stride: width,
            pixels: Arc::new(Mutex::new(vec![background_color; (width * height) as usize])),
            background_color,
        }
    }

    fn subcanvas(&self, x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            offset: x + y * self.stride + self.offset,
            stride: self.stride,
            pixels: self.pixels.clone(),
            background_color: self.background_color,
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            let mut pixels = self.pixels.lock().unwrap();
            pixels[(x + y * self.stride + self.offset) as usize] = color;
        }
    }

    pub fn draw_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32) {
        let x_end = x + width - 1;
        let y_end = y + height - 1;

        for px in x..=x_end {
            self.set_pixel(px, y, color);
            self.set_pixel(px, y_end, color);
        }

        for py in y..=y_end {
            self.set_pixel(x, py, color);
            self.set_pixel(x_end, py, color);
        }
    }

    pub fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32) {
        for j in y..y + height {
            for i in x..x + width {
                self.set_pixel(i, j, color);
            }
        }
    }

    pub fn fill(&mut self, color: u32) {
        self.fill_rect(0, 0, self.width, self.height, color);
    }

    pub fn draw_line(&mut self, x0: u32, y0: u32, x1: u32, y1: u32, color: u32) {
        let dx = (x1 as i32 - x0 as i32).abs();
        let dy = -(y1 as i32 - y0 as i32).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let (mut x, mut y) = (x0 as i32, y0 as i32);

        while x != x1 as i32 || y != y1 as i32 {
            self.set_pixel(x as u32, y as u32, color);
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn draw_oval(&mut self, cx: u32, cy: u32, width: u32, height: u32, color: u32) {
        let rx = width / 2;
        let ry = height / 2;
        let cx = cx + rx;
        let cy = cy + ry;

        for y in 0..height {
            for x in 0..width {
                let dx = x as i32 - rx as i32;
                let dy = y as i32 - ry as i32;
                if dx * dx * ry as i32 * ry as i32 + dy * dy * rx as i32 * rx as i32
                    == (rx * ry) as i32 * (rx * ry) as i32
                {
                    self.set_pixel(cx + dx as u32, cy + dy as u32, color);
                }
            }
        }
    }

    pub fn fill_oval(&mut self, cx: u32, cy: u32, width: u32, height: u32, color: u32) {
        let rx = width / 2;
        let ry = height / 2;
        let cx = cx + rx;
        let cy = cy + ry;

        for y in 0..height {
            for x in 0..width {
                let dx = x as i32 - rx as i32;
                let dy = y as i32 - ry as i32;
                if dx * dx * ry as i32 * ry as i32 + dy * dy * rx as i32 * rx as i32
                    <= (rx * ry) as i32 * (rx * ry) as i32
                {
                    self.set_pixel(cx + dx as u32, cy + dy as u32, color);
                }
            }
        }
    }

    pub fn draw_rounded_rect(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        arc_width: u32,
        arc_height: u32,
        color: u32,
    ) {
        self.draw_line(x + arc_width, y, x + width - arc_width, y, color);
        self.draw_line(x + arc_width, y + height - 1, x + width - arc_width, y + height - 1, color);
        self.draw_line(x, y + arc_height, x, y + height - arc_height, color);
        self.draw_line(x + width - 1, y + arc_height, x + width - 1, y + height - arc_height, color);
        self.set_pixel(x + arc_width - 1, y + arc_height - 1, color);
        self.set_pixel(x + width - arc_width, y + arc_height - 1, color);
        self.set_pixel(x + arc_width - 1, y + height - arc_height, color);
        self.set_pixel(x + width - arc_width, y + height - arc_height, color);
    }

    pub fn fill_rounded_rect(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        radius: u32,
        color: u32,
    ) {
        if radius == 0 {
            self.fill_rect(x, y, width, height, color);
            return;
        }

        let radius = radius.min(width / 2).min(height / 2);

        self.fill_oval(x, y, radius * 2, radius * 2, color);
        self.fill_oval(x + width - radius * 2, y, radius * 2, radius * 2, color);
        self.fill_oval(x, y + height - radius * 2, radius * 2, radius * 2, color);
        self.fill_oval(
            x + width - radius * 2,
            y + height - radius * 2,
            radius * 2,
            radius * 2,
            color,
        );

        self.fill_rect(x + radius, y, width - radius * 2, radius, color);
        self.fill_rect(
            x + radius,
            y + height - radius,
            width - radius * 2,
            radius,
            color,
        );

        self.fill_rect(x, y + radius, radius, height - radius * 2, color);
        self.fill_rect(
            x + width - radius,
            y + radius,
            radius,
            height - radius * 2,
            color,
        );

        self.fill_rect(
            x + radius,
            y + radius,
            width - radius * 2,
            height - radius * 2,
            color,
        );
    }

    fn blend_pixel(&self, foreground: u32, background: u32, alpha: u32) -> u32 {
        let fg_a = (foreground >> 24) & 0xFF; // Alpha
        let fg_r = (foreground >> 16) & 0xFF; // Red
        let fg_g = (foreground >> 8) & 0xFF;  // Green
        let fg_b = foreground & 0xFF;         // Blue
    
        let bg_color = background;
        let bg_a = (bg_color >> 24) & 0xFF;
        let bg_r = (bg_color >> 16) & 0xFF;
        let bg_g = (bg_color >> 8) & 0xFF;
        let bg_b = bg_color & 0xFF;
    
        let alpha = alpha as f32 / 255.0;
        let fg_alpha = fg_a as f32 / 255.0 * alpha;
        let bg_alpha = bg_a as f32 / 255.0 * (1.0 - fg_alpha);
    
        let out_a = (fg_alpha + bg_alpha) * 255.0;
    
        let out_r = ((fg_r as f32 * fg_alpha + bg_r as f32 * bg_alpha) / (fg_alpha + bg_alpha)) as u32;
        let out_g = ((fg_g as f32 * fg_alpha + bg_g as f32 * bg_alpha) / (fg_alpha + bg_alpha)) as u32;
        let out_b = ((fg_b as f32 * fg_alpha + bg_b as f32 * bg_alpha) / (fg_alpha + bg_alpha)) as u32;
    
        ((out_a as u32) << 24) | ((out_r as u32) << 16) | ((out_g as u32) << 8) | out_b as u32
    }
    

    pub fn draw_char(
        &mut self,
        x: u32,
        y: u32,
        c: char,
        color: u32,
        font: &Font,
        size: f32,
    ) {
        let (metrics, bitmap) = font.rasterize(c, size);
    
        let baseline_offset = metrics.height as i32 + metrics.ymin;
    
        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let pixel_x = x + col as u32;
                let pixel_y = (y as i32 + row as i32 - baseline_offset) as u32;
    
                if pixel_x >= self.width || pixel_y as u32 >= self.height {
                    continue;
                }
    
                let alpha = bitmap[row * metrics.width + col] as u32;
                if alpha > 0 {
                    let mut pixels = self.pixels.lock().unwrap();
                    let blended_color = self.blend_pixel(color, pixels.clone()[(pixel_x + pixel_y * self.stride + self.offset) as usize], alpha);
                    pixels[(pixel_x + pixel_y * self.stride + self.offset) as usize] = blended_color; // self.set_pixel(pixel_x, pixel_y as u32, blended_color);
                }
            }
        }
    }    

    pub fn draw_string(
        &mut self,
        x: u32,
        y: u32,
        text: &str,
        color: u32,
        font: &Font,
        size: f32,
    ) {
        let mut cursor_x = x;
        for c in text.chars() {
            let (metrics, _) = font.rasterize(c, size);
            self.draw_char(cursor_x, y, c, color, font, size);
            cursor_x += metrics.advance_width as u32;
        }
    }    

    pub fn draw_modules(&mut self, modules: &Modules, position: ModulePosition) {
        match position {
            ModulePosition::Left => {
                let mut cursor_x = 0;
                for module in &modules.modules {
                    let width = module.get_width();
                    let mut canvas = self.subcanvas(cursor_x, 0, width, self.height);
                    
                    module.draw(&mut canvas);

                    cursor_x += width;
                }
            }
            _ => unimplemented!()
        }
    }
}

impl Clone for Canvas {
    fn clone (&self) -> Self {
        Self {
            pixels: self.pixels.clone(),
            ..*self
        }
    }
}