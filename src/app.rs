use egui::Color32;

use crate::canvas::{self, CanvasState};
use crate::command_palette::{Command, CommandPalette};
use crate::io;
use crate::sprite::Sprite;
use crate::tools::{self, Tool};

pub struct SpriteditApp {
    sprite: Sprite,
    canvas_state: CanvasState,
    current_tool: Tool,
    primary_color: Color32,
    command_palette: CommandPalette,

    // For smooth painting — track last painted pixel
    last_paint_pos: Option<(u32, u32)>,

    // New sprite dialog
    show_new_dialog: bool,
    new_width: String,
    new_height: String,

    // URL load dialog
    show_url_dialog: bool,
    url_input: String,

    // GenAI dialog
    show_ai_dialog: bool,
    ai_prompt: String,

    // Status
    status_message: String,
}

impl SpriteditApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Self {
            sprite: Sprite::new(16, 16),
            canvas_state: CanvasState::default(),
            current_tool: Tool::Pencil,
            primary_color: Color32::from_rgb(255, 255, 255),
            command_palette: CommandPalette::default(),
            last_paint_pos: None,
            show_new_dialog: false,
            new_width: "16".into(),
            new_height: "16".into(),
            show_url_dialog: false,
            url_input: String::new(),
            show_ai_dialog: false,
            ai_prompt: String::new(),
            status_message: "Ready".into(),
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        // Don't handle tool shortcuts while command palette or dialogs are open
        if self.command_palette.is_open || self.show_new_dialog || self.show_url_dialog || self.show_ai_dialog {
            return;
        }

        ctx.input(|i| {
            // Cmd/Ctrl + Shift + P -> command palette
            let cmd = i.modifiers.command;
            let shift = i.modifiers.shift;

            if cmd && shift && i.key_pressed(egui::Key::P) {
                self.command_palette.open();
            }

            // File shortcuts
            if cmd && !shift && i.key_pressed(egui::Key::N) {
                self.show_new_dialog = true;
            }
            if cmd && !shift && i.key_pressed(egui::Key::O) {
                self.open_file();
            }
            if cmd && !shift && i.key_pressed(egui::Key::S) {
                self.save_file();
            }

            // Tool shortcuts (only when no modifier)
            if !cmd && !shift && !i.modifiers.alt {
                if i.key_pressed(egui::Key::P) {
                    self.current_tool = Tool::Pencil;
                }
                if i.key_pressed(egui::Key::E) {
                    self.current_tool = Tool::Eraser;
                }
                if i.key_pressed(egui::Key::F) {
                    self.current_tool = Tool::Fill;
                }
                if i.key_pressed(egui::Key::I) {
                    self.current_tool = Tool::ColorPicker;
                }
                if i.key_pressed(egui::Key::G) {
                    self.canvas_state.show_grid = !self.canvas_state.show_grid;
                }
                if i.key_pressed(egui::Key::V) {
                    self.canvas_state.isometric = !self.canvas_state.isometric;
                }
            }
        });
    }

    fn execute_command(&mut self, command: Command) {
        match command {
            Command::NewSprite => self.show_new_dialog = true,
            Command::OpenFile => self.open_file(),
            Command::LoadFromURL => self.show_url_dialog = true,
            Command::SaveFile => self.save_file(),
            Command::ToggleGrid => {
                self.canvas_state.show_grid = !self.canvas_state.show_grid;
            }
            Command::ToggleIsometric => {
                self.canvas_state.isometric = !self.canvas_state.isometric;
            }
            Command::SetPencil => self.current_tool = Tool::Pencil,
            Command::SetEraser => self.current_tool = Tool::Eraser,
            Command::SetFill => self.current_tool = Tool::Fill,
            Command::SetColorPicker => self.current_tool = Tool::ColorPicker,
            Command::ZoomIn => {
                self.canvas_state.zoom = (self.canvas_state.zoom * 1.5).min(128.0)
            }
            Command::ZoomOut => {
                self.canvas_state.zoom = (self.canvas_state.zoom / 1.5).max(2.0)
            }
            Command::ResetView => {
                self.canvas_state.zoom = 20.0;
                self.canvas_state.offset = egui::Vec2::ZERO;
            }
            Command::GenerateAI => self.show_ai_dialog = true,
        }
    }

    fn open_file(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(data) = io::native::open_file_dialog() {
                if let Some(sprite) = io::png_to_sprite(&data) {
                    self.status_message =
                        format!("Loaded {}x{} sprite", sprite.width, sprite.height);
                    self.sprite = sprite;
                    self.canvas_state.offset = egui::Vec2::ZERO;
                } else {
                    self.status_message = "Failed to decode image".into();
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            io::web::open_file_dialog();
            self.status_message = "Opening file...".into();
        }
    }

    fn save_file(&mut self) {
        let png_data = io::sprite_to_png(&self.sprite);
        #[cfg(not(target_arch = "wasm32"))]
        {
            if io::native::save_file_dialog(&png_data) {
                self.status_message = "Sprite saved".into();
            } else {
                self.status_message = "Save cancelled".into();
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            io::web::save_file(&png_data, "sprite.png");
            self.status_message = "Downloading sprite...".into();
        }
    }

    fn check_pending_file(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(data) = io::web::check_pending_file() {
                if let Some(sprite) = io::png_to_sprite(&data) {
                    self.status_message =
                        format!("Loaded {}x{} sprite", sprite.width, sprite.height);
                    self.sprite = sprite;
                    self.canvas_state.offset = egui::Vec2::ZERO;
                } else {
                    self.status_message = "Failed to decode image".into();
                }
            }
        }
    }

    fn apply_tool_at(&mut self, x: u32, y: u32) {
        let color_arr = [
            self.primary_color.r(),
            self.primary_color.g(),
            self.primary_color.b(),
            self.primary_color.a(),
        ];

        match self.current_tool {
            Tool::Pencil => {
                self.sprite.set_pixel(x, y, color_arr);
            }
            Tool::Eraser => {
                self.sprite.set_pixel(x, y, [0, 0, 0, 0]);
            }
            Tool::Fill => {
                self.sprite.flood_fill(x, y, color_arr);
            }
            Tool::ColorPicker => {
                let [r, g, b, a] = self.sprite.get_pixel(x, y);
                self.primary_color = Color32::from_rgba_unmultiplied(r, g, b, a);
                self.current_tool = Tool::Pencil;
            }
        }
    }

    fn handle_canvas_response(&mut self, response: canvas::CanvasResponse) {
        // Update status with hover position
        if let Some((x, y)) = response.hovered_pixel {
            let [r, g, b, a] = self.sprite.get_pixel(x, y);
            self.status_message = format!(
                "({}, {})  RGBA({}, {}, {}, {})",
                x, y, r, g, b, a
            );
        }

        // Handle painting with line interpolation
        if !response.painted_pixels.is_empty() {
            for &(x, y) in &response.painted_pixels {
                // Interpolate from last position for smooth lines
                if let Some((lx, ly)) = self.last_paint_pos {
                    let line = tools::line_pixels(
                        lx as i32, ly as i32, x as i32, y as i32,
                    );
                    for (px, py) in line {
                        if px >= 0
                            && py >= 0
                            && (px as u32) < self.sprite.width
                            && (py as u32) < self.sprite.height
                        {
                            self.apply_tool_at(px as u32, py as u32);
                        }
                    }
                } else {
                    self.apply_tool_at(x, y);
                }
                self.last_paint_pos = Some((x, y));
            }
        } else {
            self.last_paint_pos = None;
        }

        // Handle right-click color pick
        if let Some([r, g, b, a]) = response.picked_color {
            self.primary_color = Color32::from_rgba_unmultiplied(r, g, b, a);
            self.status_message = format!("Picked RGBA({}, {}, {}, {})", r, g, b, a);
        }
    }

    fn show_tool_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Tools");
            ui.separator();

            let tools = [Tool::Pencil, Tool::Eraser, Tool::Fill, Tool::ColorPicker];
            for tool in tools {
                let selected = self.current_tool == tool;
                let text = format!("{} {}", tool.icon(), tool.shortcut());
                if ui
                    .selectable_label(selected, &text)
                    .on_hover_text(tool.name())
                    .clicked()
                {
                    self.current_tool = tool;
                }
            }
        });
    }

    fn show_properties_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Properties");
        ui.separator();

        // Color picker
        ui.label("Color");
        ui.color_edit_button_srgba(&mut self.primary_color);
        ui.add_space(8.0);

        // Alpha slider
        let mut alpha = self.primary_color.a() as f32 / 255.0;
        if ui
            .add(egui::Slider::new(&mut alpha, 0.0..=1.0).text("Alpha"))
            .changed()
        {
            let [r, g, b, _] = self.primary_color.to_array();
            self.primary_color =
                Color32::from_rgba_unmultiplied(r, g, b, (alpha * 255.0) as u8);
        }

        ui.add_space(12.0);
        ui.separator();

        // Sprite info
        ui.label("Sprite");
        ui.label(format!(
            "Size: {} x {}",
            self.sprite.width, self.sprite.height
        ));
        ui.add_space(4.0);

        // Pixels per grid
        ui.label("Pixels per grid box");
        let mut ppg = self.canvas_state.pixels_per_grid as f32;
        if ui
            .add(egui::Slider::new(&mut ppg, 1.0..=32.0).integer())
            .changed()
        {
            self.canvas_state.pixels_per_grid = ppg as u32;
        }

        ui.add_space(12.0);
        ui.separator();

        // View options
        ui.label("View");
        ui.checkbox(&mut self.canvas_state.show_grid, "Show Grid (G)");
        ui.checkbox(&mut self.canvas_state.isometric, "Isometric (V)");

        let mut zoom = self.canvas_state.zoom;
        if ui
            .add(egui::Slider::new(&mut zoom, 2.0..=128.0).text("Zoom").logarithmic(true))
            .changed()
        {
            self.canvas_state.zoom = zoom;
        }

        ui.add_space(12.0);
        ui.separator();

        // GenAI section
        ui.label("Generate");
        if ui.button("AI Generate...").clicked() {
            self.show_ai_dialog = true;
        }
    }

    fn show_new_sprite_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_new_dialog;
        egui::Window::new("New Sprite")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    ui.text_edit_singleline(&mut self.new_width);
                });
                ui.horizontal(|ui| {
                    ui.label("Height:");
                    ui.text_edit_singleline(&mut self.new_height);
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Create").clicked() {
                        let w: u32 =
                            self.new_width.parse().unwrap_or(16).clamp(1, 256);
                        let h: u32 =
                            self.new_height.parse().unwrap_or(16).clamp(1, 256);
                        self.sprite = Sprite::new(w, h);
                        self.canvas_state.offset = egui::Vec2::ZERO;
                        self.status_message =
                            format!("Created new {}x{} sprite", w, h);
                        self.show_new_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_new_dialog = false;
                    }
                });
            });
        self.show_new_dialog = open;
    }

    fn load_from_url(&mut self, url: &str) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match io::native::fetch_url(url) {
                Ok(data) => {
                    if let Some(sprite) = io::png_to_sprite(&data) {
                        self.status_message =
                            format!("Loaded {}x{} sprite from URL", sprite.width, sprite.height);
                        self.sprite = sprite;
                        self.canvas_state.offset = egui::Vec2::ZERO;
                    } else {
                        self.status_message = "Failed to decode image from URL".into();
                    }
                }
                Err(e) => {
                    self.status_message = format!("URL fetch error: {e}");
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            io::web::fetch_url(url);
            self.status_message = "Fetching image from URL...".into();
        }
    }

    fn show_url_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_url_dialog;
        egui::Window::new("Load from URL")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Enter image URL:");
                ui.add_space(4.0);
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.url_input)
                        .desired_width(400.0)
                        .hint_text("https://example.com/sprite.png"),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let enter_pressed = response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if ui.button("Load").clicked() || enter_pressed {
                        let url = self.url_input.clone();
                        self.load_from_url(&url);
                        self.show_url_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_url_dialog = false;
                    }
                });
            });
        self.show_url_dialog = open;
    }

    fn show_ai_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_ai_dialog;
        egui::Window::new("Generate Sprite with AI")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Describe the sprite you want to generate:");
                ui.add_space(4.0);
                ui.add(
                    egui::TextEdit::multiline(&mut self.ai_prompt)
                        .desired_rows(3)
                        .desired_width(350.0)
                        .hint_text("e.g. a 16x16 pixel art sword"),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Generate").clicked() {
                        self.status_message =
                            "AI generation not yet connected to a backend"
                                .into();
                        self.show_ai_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_ai_dialog = false;
                    }
                });
                ui.add_space(4.0);
                ui.weak("Connect an API key to enable AI sprite generation.");
            });
        self.show_ai_dialog = open;
    }
}

impl eframe::App for SpriteditApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for async file loads (WASM)
        self.check_pending_file();

        // Global keyboard shortcuts
        self.handle_shortcuts(ctx);

        // Command palette overlay
        if let Some(cmd) = self.command_palette.show(ctx) {
            self.execute_command(cmd);
        }

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Sprite...  Ctrl+N").clicked() {
                        self.show_new_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("Open...  Ctrl+O").clicked() {
                        self.open_file();
                        ui.close_menu();
                    }
                    if ui.button("Load from URL...").clicked() {
                        self.show_url_dialog = true;
                        ui.close_menu();
                    }
                    if ui.button("Save  Ctrl+S").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Pencil  P").clicked() {
                        self.current_tool = Tool::Pencil;
                        ui.close_menu();
                    }
                    if ui.button("Eraser  E").clicked() {
                        self.current_tool = Tool::Eraser;
                        ui.close_menu();
                    }
                    if ui.button("Fill  F").clicked() {
                        self.current_tool = Tool::Fill;
                        ui.close_menu();
                    }
                    if ui.button("Color Picker  I").clicked() {
                        self.current_tool = Tool::ColorPicker;
                        ui.close_menu();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui
                        .checkbox(&mut self.canvas_state.show_grid, "Grid  G")
                        .clicked()
                    {
                        ui.close_menu();
                    }
                    if ui
                        .checkbox(&mut self.canvas_state.isometric, "Isometric  V")
                        .clicked()
                    {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Zoom In  +").clicked() {
                        self.canvas_state.zoom =
                            (self.canvas_state.zoom * 1.5).min(128.0);
                        ui.close_menu();
                    }
                    if ui.button("Zoom Out  -").clicked() {
                        self.canvas_state.zoom =
                            (self.canvas_state.zoom / 1.5).max(2.0);
                        ui.close_menu();
                    }
                    if ui.button("Reset View  0").clicked() {
                        self.canvas_state.zoom = 20.0;
                        self.canvas_state.offset = egui::Vec2::ZERO;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("Command Palette  Cmd+Shift+P").clicked() {
                        self.command_palette.open();
                        ui.close_menu();
                    }
                });
            });
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(24.0)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(&self.status_message);
                    ui.separator();
                    ui.label(format!(
                        "{}x{}",
                        self.sprite.width, self.sprite.height
                    ));
                    ui.separator();
                    ui.label(format!("Tool: {}", self.current_tool.name()));
                    ui.separator();
                    ui.label(format!("Zoom: {:.0}x", self.canvas_state.zoom));
                });
            });

        // Left panel — tools
        egui::SidePanel::left("tools_panel")
            .resizable(false)
            .exact_width(64.0)
            .show(ctx, |ui| {
                self.show_tool_panel(ui);
            });

        // Right panel — properties
        egui::SidePanel::right("properties_panel")
            .default_width(220.0)
            .show(ctx, |ui| {
                self.show_properties_panel(ui);
            });

        // Center — canvas
        egui::CentralPanel::default().show(ctx, |ui| {
            let response =
                canvas::show_canvas(ui, &self.sprite, &mut self.canvas_state);
            self.handle_canvas_response(response);
        });

        // Dialogs
        if self.show_new_dialog {
            self.show_new_sprite_dialog(ctx);
        }
        if self.show_url_dialog {
            self.show_url_dialog(ctx);
        }
        if self.show_ai_dialog {
            self.show_ai_dialog(ctx);
        }
    }
}
