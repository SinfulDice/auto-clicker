use auto_clicker::app::AutoClickerApp;
use auto_clicker::clicker::{Clicker, EnigoClicker};
use auto_clicker::config::{self, Config};
use auto_clicker::engine::Engine;
use eframe::egui;

fn main() -> eframe::Result {
    let config_path = config::default_path();
    let config = config_path.as_deref().map(Config::load).unwrap_or_default(); // R21, R22

    let engine = Engine::spawn(config, || {
        EnigoClicker::new().map(|clicker| Box::new(clicker) as Box<dyn Clicker>)
    });

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Auto-clicker")
        .with_app_id("auto-clicker")
        .with_inner_size([340.0, 240.0])
        .with_resizable(false);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(auto_clicker::ICON_PNG) {
        viewport = viewport.with_icon(icon); // R20
    }

    eframe::run_native(
        "auto-clicker",
        eframe::NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(move |cc| {
            Ok(Box::new(AutoClickerApp::new(
                &cc.egui_ctx,
                config,
                config_path,
                engine,
            )))
        }),
    )
}
