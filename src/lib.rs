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
