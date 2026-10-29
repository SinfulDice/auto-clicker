//! La fenêtre (egui) et la touche globale F6.

use crate::config::{Config, MAX_CPS, MIN_CPS, MouseButton};
use crate::engine::{Engine, EngineHandle};
use crate::session::{StartSource, Status, status_label};
use global_hotkey::hotkey::{Code, HotKey};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct AutoClickerApp {
    config: Config,
    config_path: Option<PathBuf>,
    engine: Engine,
    /// Doit rester vivant : le détruire libère la touche F6.
    _hotkeys: Option<GlobalHotKeyManager>,
    hotkey_warning: Option<String>,
    save_error: Option<String>,
}

impl AutoClickerApp {
    pub fn new(
        ctx: &eframe::egui::Context,
        config: Config,
        config_path: Option<PathBuf>,
        engine: Engine,
    ) -> Self {
        let (hotkeys, hotkey_warning) = match register_f6(engine.handle(), ctx.clone()) {
            Ok(manager) => (Some(manager), None),
            // R16 : sans F6, l'app reste utilisable avec le bouton.
            Err(e) => (
                None,
                Some(format!("Touche F6 indisponible ({e}). Utilise le bouton.")),
            ),
        };
        Self {
            config,
            config_path,
            engine,
            _hotkeys: hotkeys,
            hotkey_warning,
            save_error: None,
        }
    }

    fn save_config(&mut self) {
        self.engine.handle().set_config(self.config);
        if let Some(path) = &self.config_path {
            self.save_error = self
                .config
                .save(path)
                .err()
                .map(|e| format!("Sauvegarde impossible : {e}"));
        }
    }
}

/// R11 : F6 démarre/arrête, même sans le focus. L'appel se fait dans le fil de
/// `global-hotkey`, indépendamment de la fenêtre.
fn register_f6(engine: EngineHandle, ctx: eframe::egui::Context) -> Result<GlobalHotKeyManager, String> {
    let manager = GlobalHotKeyManager::new().map_err(|e| e.to_string())?;
    manager
        .register(HotKey::new(None, Code::F6))
        .map_err(|e| e.to_string())?;
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
        if event.state == HotKeyState::Pressed {
            engine.toggle(StartSource::Hotkey);
            ctx.request_repaint();
        }
    }));
    Ok(manager)
}

impl eframe::App for AutoClickerApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        use eframe::egui;
        let engine = self.engine.handle();
        let status = engine.status();
        let stopped = status == Status::Stopped;

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Auto-clicker");
            ui.add_space(8.0);

            // R4 : réglages grisés tant que ça tourne.
            let before = self.config;
            ui.add_enabled_ui(stopped, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Bouton :");
                    ui.radio_value(&mut self.config.button, MouseButton::Left, "Gauche");
                    ui.radio_value(&mut self.config.button, MouseButton::Right, "Droit");
                });
                ui.horizontal(|ui| {
                    ui.label("Vitesse :");
                    ui.add(
                        egui::Slider::new(&mut self.config.cps, MIN_CPS..=MAX_CPS)
                            .logarithmic(true)
                            .suffix(" clics/s"),
                    );
                });
            });
            if self.config != before {
                self.save_config(); // R21
            }

            ui.add_space(8.0);
            let label = if stopped { "Start" } else { "Stop" };
            let enabled = engine.error().is_none();
            if ui
                .add_enabled(
                    enabled,
                    egui::Button::new(label).min_size(egui::vec2(120.0, 32.0)),
                )
                .clicked()
            {
                engine.toggle(StartSource::Button); // R10, R12
            }

            ui.add_space(8.0);
            ui.label(format!("État : {}", status_label(status, Instant::now()))); // R17
            ui.label(format!("Clics : {}", engine.clicks())); // R18
            ui.weak("F6 = Start/Stop"); // R19

            let warn = egui::Color32::from_rgb(220, 120, 40);
            for message in [
                engine.error(),
                self.hotkey_warning.clone(),
                self.save_error.clone(),
            ]
            .into_iter()
            .flatten()
            {
                ui.colored_label(warn, message);
            }
        });

        // Rafraîchit l'état et le compteur tant que ça tourne.
        if !stopped {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
        }
    }
}
