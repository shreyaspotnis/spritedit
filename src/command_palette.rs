#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Command {
    NewSprite,
    OpenFile,
    SaveFile,
    ToggleGrid,
    ToggleIsometric,
    SetPencil,
    SetEraser,
    SetFill,
    SetColorPicker,
    ZoomIn,
    ZoomOut,
    ResetView,
    GenerateAI,
}

pub struct CommandEntry {
    pub name: &'static str,
    pub shortcut: &'static str,
    pub command: Command,
}

pub struct CommandPalette {
    pub is_open: bool,
    pub query: String,
    pub selected_index: usize,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self {
            is_open: false,
            query: String::new(),
            selected_index: 0,
        }
    }
}

impl CommandPalette {
    pub fn commands() -> Vec<CommandEntry> {
        vec![
            CommandEntry {
                name: "New Sprite",
                shortcut: "Ctrl+N",
                command: Command::NewSprite,
            },
            CommandEntry {
                name: "Open File...",
                shortcut: "Ctrl+O",
                command: Command::OpenFile,
            },
            CommandEntry {
                name: "Save File",
                shortcut: "Ctrl+S",
                command: Command::SaveFile,
            },
            CommandEntry {
                name: "Toggle Grid",
                shortcut: "G",
                command: Command::ToggleGrid,
            },
            CommandEntry {
                name: "Toggle Isometric View",
                shortcut: "V",
                command: Command::ToggleIsometric,
            },
            CommandEntry {
                name: "Pencil Tool",
                shortcut: "P",
                command: Command::SetPencil,
            },
            CommandEntry {
                name: "Eraser Tool",
                shortcut: "E",
                command: Command::SetEraser,
            },
            CommandEntry {
                name: "Fill Tool",
                shortcut: "F",
                command: Command::SetFill,
            },
            CommandEntry {
                name: "Color Picker Tool",
                shortcut: "I",
                command: Command::SetColorPicker,
            },
            CommandEntry {
                name: "Zoom In",
                shortcut: "+",
                command: Command::ZoomIn,
            },
            CommandEntry {
                name: "Zoom Out",
                shortcut: "-",
                command: Command::ZoomOut,
            },
            CommandEntry {
                name: "Reset View",
                shortcut: "0",
                command: Command::ResetView,
            },
            CommandEntry {
                name: "Generate with AI...",
                shortcut: "",
                command: Command::GenerateAI,
            },
        ]
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.query.clear();
        self.selected_index = 0;
    }

    /// Show the command palette overlay. Returns a command if one was executed.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<Command> {
        if !self.is_open {
            return None;
        }

        let mut executed = None;
        let commands = Self::commands();

        egui::Area::new(egui::Id::new("command_palette"))
            .anchor(egui::Align2::CENTER_TOP, [0.0, 60.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style())
                    .shadow(egui::epaint::Shadow {
                        offset: egui::vec2(0.0, 4.0),
                        blur: 16.0,
                        spread: 4.0,
                        color: egui::Color32::from_black_alpha(80),
                    })
                    .show(ui, |ui| {
                        ui.set_min_width(450.0);
                        ui.set_max_width(450.0);

                        // Search input
                        let input = ui.add(
                            egui::TextEdit::singleline(&mut self.query)
                                .hint_text("Type a command...")
                                .desired_width(f32::INFINITY),
                        );
                        input.request_focus();

                        // Escape to close
                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            self.is_open = false;
                            return;
                        }

                        // Filter commands
                        let query_lower = self.query.to_lowercase();
                        let filtered: Vec<&CommandEntry> = commands
                            .iter()
                            .filter(|c| c.name.to_lowercase().contains(&query_lower))
                            .collect();

                        // Arrow key navigation
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                            self.selected_index = (self.selected_index + 1)
                                .min(filtered.len().saturating_sub(1));
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                            self.selected_index = self.selected_index.saturating_sub(1);
                        }
                        self.selected_index =
                            self.selected_index.min(filtered.len().saturating_sub(1));

                        // Enter to execute
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if let Some(entry) = filtered.get(self.selected_index) {
                                executed = Some(entry.command);
                                self.is_open = false;
                            }
                        }

                        // Command list
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                for (i, entry) in filtered.iter().enumerate() {
                                    let selected = i == self.selected_index;
                                    let response = ui.horizontal(|ui| {
                                        let label = ui.selectable_label(selected, entry.name);
                                        if !entry.shortcut.is_empty() {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.weak(entry.shortcut);
                                                },
                                            );
                                        }
                                        label
                                    });
                                    if response.inner.clicked() {
                                        executed = Some(entry.command);
                                        self.is_open = false;
                                    }
                                }
                            });
                    });
            });

        executed
    }
}
