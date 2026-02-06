use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>, // RGBA, row-major, 4 bytes per pixel
}

impl Sprite {
    pub fn new(width: u32, height: u32) -> Self {
        let pixels = vec![0u8; (width * height * 4) as usize];
        Self {
            width,
            height,
            pixels,
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let idx = ((y * self.width + x) * 4) as usize;
        [
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ]
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        if x < self.width && y < self.height {
            let idx = ((y * self.width + x) * 4) as usize;
            self.pixels[idx..idx + 4].copy_from_slice(&color);
        }
    }

    pub fn flood_fill(&mut self, x: u32, y: u32, fill_color: [u8; 4]) {
        let target_color = self.get_pixel(x, y);
        if target_color == fill_color {
            return;
        }
        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            if cx >= self.width || cy >= self.height {
                continue;
            }
            if self.get_pixel(cx, cy) != target_color {
                continue;
            }
            self.set_pixel(cx, cy, fill_color);
            if cx > 0 {
                stack.push((cx - 1, cy));
            }
            if cy > 0 {
                stack.push((cx, cy - 1));
            }
            stack.push((cx + 1, cy));
            stack.push((cx, cy + 1));
        }
    }

    pub fn to_color_image(&self) -> egui::ColorImage {
        egui::ColorImage::from_rgba_unmultiplied(
            [self.width as usize, self.height as usize],
            &self.pixels,
        )
    }
}
