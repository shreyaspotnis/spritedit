#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Pencil,
    Eraser,
    Fill,
    ColorPicker,
}

impl Tool {
    pub fn name(&self) -> &'static str {
        match self {
            Tool::Pencil => "Pencil",
            Tool::Eraser => "Eraser",
            Tool::Fill => "Fill",
            Tool::ColorPicker => "Pick Color",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            Tool::Pencil => "P",
            Tool::Eraser => "E",
            Tool::Fill => "F",
            Tool::ColorPicker => "I",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Tool::Pencil => "\u{270F}",
            Tool::Eraser => "\u{2B1C}",
            Tool::Fill => "\u{2B24}",
            Tool::ColorPicker => "\u{25C9}",
        }
    }
}

/// Bresenham's line algorithm — returns all pixels along a line between two points.
pub fn line_pixels(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<(i32, i32)> {
    let mut pixels = Vec::new();
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = x0;
    let mut y = y0;
    loop {
        pixels.push((x, y));
        if x == x1 && y == y1 {
            break;
        }
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
    pixels
}
