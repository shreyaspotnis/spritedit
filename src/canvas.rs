use egui::{Color32, Pos2, Rect, Stroke, Vec2, pos2, vec2};

use crate::sprite::Sprite;

pub struct CanvasState {
    pub zoom: f32,
    pub offset: Vec2,
    pub show_grid: bool,
    pub isometric: bool,
    pub pixels_per_grid: u32,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            zoom: 20.0,
            offset: Vec2::ZERO,
            show_grid: true,
            isometric: false,
            pixels_per_grid: 1,
        }
    }
}

pub struct CanvasResponse {
    pub hovered_pixel: Option<(u32, u32)>,
    pub painted_pixels: Vec<(u32, u32)>,
    pub picked_color: Option<[u8; 4]>,
}

pub fn show_canvas(
    ui: &mut egui::Ui,
    sprite: &Sprite,
    state: &mut CanvasState,
) -> CanvasResponse {
    let available = ui.available_size();
    let (response, painter) =
        ui.allocate_painter(available, egui::Sense::click_and_drag());
    let rect = response.rect;

    // Zoom with scroll wheel
    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > 0.1 {
            let old_zoom = state.zoom;
            state.zoom = (state.zoom * (1.0 + scroll * 0.005)).clamp(2.0, 128.0);
            // Zoom toward mouse position
            if let Some(mouse) = response.hover_pos() {
                let mouse_rel = mouse - rect.center() - state.offset;
                state.offset += mouse_rel * (1.0 - state.zoom / old_zoom);
            }
        }
    }

    // Pan with middle mouse button
    if response.dragged_by(egui::PointerButton::Middle) {
        state.offset += response.drag_delta();
    }

    // Draw canvas background
    painter.rect_filled(rect, 0.0, Color32::from_rgb(40, 40, 40));

    // Draw sprite
    if state.isometric {
        draw_isometric(&painter, sprite, rect, state);
    } else {
        draw_flat(&painter, sprite, rect, state);
    }

    // Build response
    let mut canvas_response = CanvasResponse {
        hovered_pixel: None,
        painted_pixels: Vec::new(),
        picked_color: None,
    };

    if let Some(mouse_pos) = response.hover_pos() {
        let pixel = if state.isometric {
            screen_to_pixel_iso(mouse_pos, rect, state, sprite)
        } else {
            screen_to_pixel_flat(mouse_pos, rect, state, sprite)
        };

        if let Some((px, py)) = pixel {
            canvas_response.hovered_pixel = Some((px, py));

            // Draw hover highlight
            if !state.isometric {
                let origin = sprite_origin(rect, state, sprite);
                let highlight_rect = Rect::from_min_size(
                    pos2(
                        origin.x + px as f32 * state.zoom,
                        origin.y + py as f32 * state.zoom,
                    ),
                    vec2(state.zoom, state.zoom),
                );
                painter.rect_stroke(
                    highlight_rect,
                    0.0,
                    Stroke::new(2.0, Color32::WHITE),
                );
            }

            // Paint on primary click/drag
            if response.dragged_by(egui::PointerButton::Primary)
                || response.clicked_by(egui::PointerButton::Primary)
            {
                canvas_response.painted_pixels.push((px, py));
            }

            // Color pick on right click
            if response.clicked_by(egui::PointerButton::Secondary) {
                canvas_response.picked_color = Some(sprite.get_pixel(px, py));
            }
        }
    }

    canvas_response
}

fn sprite_origin(rect: Rect, state: &CanvasState, sprite: &Sprite) -> Pos2 {
    let sprite_screen_w = sprite.width as f32 * state.zoom;
    let sprite_screen_h = sprite.height as f32 * state.zoom;
    pos2(
        rect.center().x - sprite_screen_w / 2.0 + state.offset.x,
        rect.center().y - sprite_screen_h / 2.0 + state.offset.y,
    )
}

fn draw_flat(painter: &egui::Painter, sprite: &Sprite, rect: Rect, state: &CanvasState) {
    let pixel_size = state.zoom;
    let origin = sprite_origin(rect, state, sprite);

    let light = Color32::from_rgb(200, 200, 200);
    let dark = Color32::from_rgb(160, 160, 160);
    let check_size = (pixel_size / 2.0).max(1.0);

    for y in 0..sprite.height {
        for x in 0..sprite.width {
            let px = origin.x + x as f32 * pixel_size;
            let py = origin.y + y as f32 * pixel_size;
            let pixel_rect = Rect::from_min_size(pos2(px, py), vec2(pixel_size, pixel_size));

            if !rect.intersects(pixel_rect) {
                continue;
            }

            // Checkerboard background (transparency indicator)
            for cy in 0..2u32 {
                for cx in 0..2u32 {
                    let cr = Rect::from_min_size(
                        pos2(px + cx as f32 * check_size, py + cy as f32 * check_size),
                        vec2(check_size, check_size),
                    );
                    let color = if (cx + cy) % 2 == 0 { light } else { dark };
                    painter.rect_filled(cr, 0.0, color);
                }
            }

            // Draw pixel
            let [r, g, b, a] = sprite.get_pixel(x, y);
            if a > 0 {
                painter.rect_filled(
                    pixel_rect,
                    0.0,
                    Color32::from_rgba_unmultiplied(r, g, b, a),
                );
            }
        }
    }

    // Grid lines
    if state.show_grid && state.zoom >= 4.0 {
        let thin_color = Color32::from_rgba_unmultiplied(100, 100, 100, 60);
        let thick_color = Color32::from_rgba_unmultiplied(140, 140, 140, 100);
        let ppg = state.pixels_per_grid.max(1);

        for x in 0..=sprite.width {
            let sx = origin.x + x as f32 * pixel_size;
            let is_major = ppg > 1 && x % ppg == 0;
            let stroke = if is_major {
                Stroke::new(2.0, thick_color)
            } else {
                Stroke::new(1.0, thin_color)
            };
            painter.line_segment(
                [
                    pos2(sx, origin.y),
                    pos2(sx, origin.y + sprite.height as f32 * pixel_size),
                ],
                stroke,
            );
        }

        for y in 0..=sprite.height {
            let sy = origin.y + y as f32 * pixel_size;
            let is_major = ppg > 1 && y % ppg == 0;
            let stroke = if is_major {
                Stroke::new(2.0, thick_color)
            } else {
                Stroke::new(1.0, thin_color)
            };
            painter.line_segment(
                [
                    pos2(origin.x, sy),
                    pos2(origin.x + sprite.width as f32 * pixel_size, sy),
                ],
                stroke,
            );
        }

        // Border
        let border = Rect::from_min_size(
            origin,
            vec2(
                sprite.width as f32 * pixel_size,
                sprite.height as f32 * pixel_size,
            ),
        );
        painter.rect_stroke(border, 0.0, Stroke::new(2.0, Color32::from_rgb(80, 80, 80)));
    }
}

fn draw_isometric(painter: &egui::Painter, sprite: &Sprite, rect: Rect, state: &CanvasState) {
    let tile_w = state.zoom;
    let tile_h = state.zoom / 2.0;
    let center_x = rect.center().x + state.offset.x;
    let center_y =
        rect.center().y + state.offset.y - (sprite.height as f32 * tile_h / 2.0);

    let light = Color32::from_rgb(200, 200, 200);
    let dark = Color32::from_rgb(160, 160, 160);

    for y in 0..sprite.height {
        for x in 0..sprite.width {
            let iso_x = center_x + (x as f32 - y as f32) * tile_w / 2.0;
            let iso_y = center_y + (x as f32 + y as f32) * tile_h / 2.0;

            let top = pos2(iso_x, iso_y);
            let right = pos2(iso_x + tile_w / 2.0, iso_y + tile_h / 2.0);
            let bottom = pos2(iso_x, iso_y + tile_h);
            let left = pos2(iso_x - tile_w / 2.0, iso_y + tile_h / 2.0);

            let diamond = vec![top, right, bottom, left];

            // Checkerboard
            let check_color = if (x + y) % 2 == 0 { light } else { dark };
            painter.add(egui::Shape::convex_polygon(
                diamond.clone(),
                check_color,
                Stroke::NONE,
            ));

            // Pixel color
            let [r, g, b, a] = sprite.get_pixel(x, y);
            if a > 0 {
                painter.add(egui::Shape::convex_polygon(
                    diamond.clone(),
                    Color32::from_rgba_unmultiplied(r, g, b, a),
                    Stroke::NONE,
                ));
            }

            // Grid
            if state.show_grid {
                let grid_stroke =
                    Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 100, 100, 60));
                painter.line_segment([top, right], grid_stroke);
                painter.line_segment([right, bottom], grid_stroke);
                painter.line_segment([bottom, left], grid_stroke);
                painter.line_segment([left, top], grid_stroke);
            }
        }
    }
}

fn screen_to_pixel_flat(
    mouse: Pos2,
    rect: Rect,
    state: &CanvasState,
    sprite: &Sprite,
) -> Option<(u32, u32)> {
    let origin = sprite_origin(rect, state, sprite);
    let rel_x = mouse.x - origin.x;
    let rel_y = mouse.y - origin.y;
    if rel_x >= 0.0 && rel_y >= 0.0 {
        let px = (rel_x / state.zoom) as u32;
        let py = (rel_y / state.zoom) as u32;
        if px < sprite.width && py < sprite.height {
            return Some((px, py));
        }
    }
    None
}

fn screen_to_pixel_iso(
    mouse: Pos2,
    rect: Rect,
    state: &CanvasState,
    sprite: &Sprite,
) -> Option<(u32, u32)> {
    let tile_w = state.zoom;
    let tile_h = state.zoom / 2.0;
    let center_x = rect.center().x + state.offset.x;
    let center_y =
        rect.center().y + state.offset.y - (sprite.height as f32 * tile_h / 2.0);

    let rel_x = mouse.x - center_x;
    let rel_y = mouse.y - center_y;

    let col = (rel_x / (tile_w / 2.0) + rel_y / (tile_h / 2.0)) / 2.0;
    let row = (rel_y / (tile_h / 2.0) - rel_x / (tile_w / 2.0)) / 2.0;

    if col >= 0.0 && row >= 0.0 {
        let px = col as u32;
        let py = row as u32;
        if px < sprite.width && py < sprite.height {
            return Some((px, py));
        }
    }
    None
}
