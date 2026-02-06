#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod canvas;
mod command_palette;
mod io;
mod sprite;
mod tools;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1280.0, 720.0]),
            ..Default::default()
        };
        eframe::run_native(
            "Spritedit",
            options,
            Box::new(|cc| Ok(Box::new(app::SpriteditApp::new(cc)))),
        )
        .expect("Failed to start Spritedit");
    }

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
        wasm_bindgen_futures::spawn_local(async {
            eframe::WebRunner::new()
                .start(
                    "spritedit_canvas",
                    eframe::WebOptions::default(),
                    Box::new(|cc| Ok(Box::new(app::SpriteditApp::new(cc)))),
                )
                .await
                .expect("Failed to start Spritedit");
        });
    }
}
