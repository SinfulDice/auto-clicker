#!/usr/bin/env bash
# R23 : installe l'auto-clicker pour l'utilisateur courant (pas besoin de sudo).
#   ./install.sh            installe ou met à jour
#   ./install.sh --remove   désinstalle
set -euo pipefail
cd "$(dirname "$0")"

APPS="$HOME/.local/share/applications"
ICONS="$HOME/.local/share/icons/hicolor"

if [[ "${1:-}" == "--remove" ]]; then
    cargo uninstall auto-clicker || true
    rm -f "$APPS/auto-clicker.desktop" "$ICONS/256x256/apps/auto-clicker.png" "$ICONS/scalable/apps/auto-clicker.svg"
    echo "Auto-clicker désinstallé."
    exit 0
fi

# 1. Compile en mode release et copie le programme dans ~/.cargo/bin
cargo install --path . --locked

# 2. Icône (PNG + SVG) dans le thème d'icônes de l'utilisateur
install -Dm644 assets/icon.png "$ICONS/256x256/apps/auto-clicker.png"
install -Dm644 assets/icon.svg "$ICONS/scalable/apps/auto-clicker.svg"

# 3. Fiche du menu, avec le chemin complet du programme (le menu ne connaît pas toujours ~/.cargo/bin)
install -d "$APPS"
sed "s|^Exec=.*|Exec=$HOME/.cargo/bin/auto-clicker|" assets/auto-clicker.desktop > "$APPS/auto-clicker.desktop"

update-desktop-database "$APPS" 2>/dev/null || true
gtk-update-icon-cache -q "$ICONS" 2>/dev/null || true
echo "Installé ! Cherche « Auto-clicker » dans le menu."
