# Auto-clicker

![icône](assets/icon.png)

Un auto-clicker simple pour **Linux (X11)**, écrit en **Rust**, avec une petite interface graphique.

- Clic **gauche** ou **droit**, à la position actuelle de la souris.
- Vitesse réglable de **1 à 1000 clics par seconde**.
- **Start/Stop** avec le bouton (3 s de délai pour placer la souris) ou la touche **F6** (immédiat, marche même si la fenêtre n'a pas le focus).
- Affiche l'état et le nombre de clics.
- Les réglages sont sauvegardés dans `~/.config/auto-clicker/config.toml`.

## Prérequis

- Une session **X11** (par défaut sur Linux Mint). Wayland n'est pas pris en charge.
- [Rust](https://rustup.rs) (édition 2024).

## Lancer

```sh
cargo run --release
```

## Tests

```sh
cargo test        # tests automatiques (règles de SPEC.md)
```

Voir aussi [TESTING.md](TESTING.md) pour la checklist manuelle et [SPEC.md](SPEC.md) pour la spécification complète.

## Structure

| Fichier | Rôle |
|---|---|
| `src/main.rs` | Point d'entrée : charge la config, lance le moteur et la fenêtre |
| `src/app.rs` | Interface (egui) et touche globale F6 |
| `src/engine.rs` | Fil d'exécution qui envoie les clics |
| `src/session.rs` | Logique pure : états, compte à rebours, rythme sans dérive |
| `src/clicker.rs` | Envoi réel des clics (`enigo`) + faux cliqueur pour les tests |
| `src/config.rs` | Réglages et sauvegarde TOML |
| `assets/icon.svg` | Icône (source), exportée en PNG avec `cargo run --example export_icon` |

## Limites

- X11 seulement.
- À très haute vitesse (plusieurs centaines de CPS), certaines applications ne traitent pas tous les clics.
