pub mod app;
pub mod clicker;
pub mod config;
pub mod engine;
pub mod session;

/// R20 : icône de la fenêtre, générée depuis `assets/icon.svg`.
pub const ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

#[cfg(test)]
mod tests {
    #[test]
    fn r20_window_icon_is_a_valid_square_png() {
        let icon = eframe::icon_data::from_png_bytes(super::ICON_PNG).unwrap();
        assert_eq!((icon.width, icon.height), (256, 256));
    }

    #[test]
    fn r20_icon_svg_source_exists() {
        let svg = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.svg")).unwrap();
        assert!(svg.contains("<svg"));
    }
}

#[cfg(test)]
mod desktop_entry_tests {
    const DESKTOP: &str = include_str!("../assets/auto-clicker.desktop");

    #[test]
    fn r23_desktop_entry_is_searchable_in_menu() {
        for line in [
            "[Desktop Entry]",
            "Type=Application",
            "Name=Auto-clicker",
            "Exec=auto-clicker",
            "Icon=auto-clicker",
        ] {
            assert!(DESKTOP.lines().any(|l| l == line), "ligne manquante : {line}");
        }
        assert!(DESKTOP.contains("Keywords=") && DESKTOP.contains("click;"));
    }

    #[test]
    fn r23_r20_desktop_wm_class_matches_window_app_id() {
        // Sans ça, la barre des tâches n'associe pas la fenêtre à son icône.
        let main = include_str!("main.rs");
        assert!(DESKTOP.lines().any(|l| l == "StartupWMClass=auto-clicker"));
        assert!(main.contains(".with_app_id(\"auto-clicker\")"));
    }
}
