//! Réglages de l'utilisateur et leur sauvegarde sur disque (R1–R3, R21, R22).

use serde::Serialize;
use std::path::{Path, PathBuf};

pub const MIN_CPS: u32 = 1;
pub const MAX_CPS: u32 = 1000;
pub const DEFAULT_CPS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MouseButton {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Config {
    pub button: MouseButton,
    pub cps: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            button: MouseButton::default(),
            cps: DEFAULT_CPS,
        }
    }
}

/// R3 : ramène un CPS dans la plage autorisée.
pub fn clamp_cps(cps: i64) -> u32 {
    cps.clamp(MIN_CPS as i64, MAX_CPS as i64) as u32
}

impl Config {
    /// Lit la config depuis du texte TOML. Chaque champ absent ou invalide
    /// reprend sa valeur par défaut au lieu de faire échouer tout le fichier (R22).
    pub fn from_toml(text: &str) -> Self {
        let mut config = Config::default();
        let Ok(table) = text.parse::<toml::Table>() else {
            return config;
        };
        match table.get("button").and_then(|v| v.as_str()) {
            Some("left") => config.button = MouseButton::Left,
            Some("right") => config.button = MouseButton::Right,
            _ => {}
        }
        if let Some(cps) = table.get("cps").and_then(|v| v.as_integer()) {
            config.cps = clamp_cps(cps);
        }
        config
    }

    pub fn to_toml(&self) -> String {
        toml::to_string(self).expect("la config est toujours sérialisable")
    }

    /// R21/R22 : charge depuis un fichier ; fichier absent ou illisible → défauts.
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .map(|text| Self::from_toml(&text))
            .unwrap_or_default()
    }

    /// R21 : écrit la config, en créant le dossier si besoin.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, self.to_toml())
    }
}

/// `~/.config/auto-clicker/config.toml`
pub fn default_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("auto-clicker").join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r1_r2_defaults_are_left_button_and_10_cps() {
        let config = Config::default();
        assert_eq!(config.button, MouseButton::Left);
        assert_eq!(config.cps, 10);
    }

    #[test]
    fn r2_r3_cps_is_clamped_to_1_1000() {
        assert_eq!(clamp_cps(0), 1);
        assert_eq!(clamp_cps(-5), 1);
        assert_eq!(clamp_cps(1), 1);
        assert_eq!(clamp_cps(500), 500);
        assert_eq!(clamp_cps(1000), 1000);
        assert_eq!(clamp_cps(5000), 1000);
    }

    #[test]
    fn r21_save_then_load_gives_same_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub/dir/config.toml");
        let config = Config {
            button: MouseButton::Right,
            cps: 250,
        };
        config.save(&path).unwrap();
        assert_eq!(Config::load(&path), config);
    }

    #[test]
    fn r21_file_format_is_readable_toml() {
        let text = Config {
            button: MouseButton::Right,
            cps: 42,
        }
        .to_toml();
        assert!(text.contains("button = \"right\""));
        assert!(text.contains("cps = 42"));
    }

    #[test]
    fn r22_missing_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(Config::load(&dir.path().join("nope.toml")), Config::default());
    }

    #[test]
    fn r22_garbage_file_gives_defaults() {
        assert_eq!(Config::from_toml("ceci n'est pas du toml [[["), Config::default());
    }

    #[test]
    fn r22_invalid_values_fall_back_or_are_clamped() {
        let config = Config::from_toml("button = \"milieu\"\ncps = 99999");
        assert_eq!(config.button, MouseButton::Left);
        assert_eq!(config.cps, 1000);

        let config = Config::from_toml("button = \"right\"\ncps = \"vite\"");
        assert_eq!(config.button, MouseButton::Right);
        assert_eq!(config.cps, DEFAULT_CPS);
    }
}
