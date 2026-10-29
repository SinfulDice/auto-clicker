//! Envoi des clics. Le trait permet aux tests d'utiliser un faux cliqueur.

use crate::config::MouseButton;

pub trait Clicker {
    /// R5/R6 : un simple clic (appui + relâchement) à la position actuelle du curseur.
    fn click(&mut self, button: MouseButton);
}

/// Le vrai cliqueur : passe par `enigo` (extension XTest sous X11).
pub struct EnigoClicker {
    enigo: enigo::Enigo,
}

impl EnigoClicker {
    pub fn new() -> Result<Self, String> {
        enigo::Enigo::new(&enigo::Settings::default())
            .map(|enigo| Self { enigo })
            .map_err(|e| format!("impossible de se connecter au serveur X11 : {e}"))
    }
}

impl Clicker for EnigoClicker {
    fn click(&mut self, button: MouseButton) {
        use enigo::{Button, Direction, Mouse};
        let button = match button {
            MouseButton::Left => Button::Left,
            MouseButton::Right => Button::Right,
        };
        // Une erreur ponctuelle (ex. serveur X occupé) ne doit pas arrêter la boucle.
        let _ = self.enigo.button(button, Direction::Click);
    }
}

/// Faux cliqueur pour les tests : retient les clics reçus.
#[cfg(test)]
#[derive(Default)]
pub struct FakeClicker {
    pub clicks: Vec<MouseButton>,
}

#[cfg(test)]
impl Clicker for FakeClicker {
    fn click(&mut self, button: MouseButton) {
        self.clicks.push(button);
    }
}
