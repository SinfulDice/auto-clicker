# Tests manuels

Les règles liées au vrai bureau ne se testent pas avec `cargo test`. Vérifie-les à la main avec `cargo run --release`.

Pour tester les clics, ouvre une page de test de CPS dans le navigateur (cherche « cps test »), ou un éditeur de texte pour le clic droit.

- [ ] **R5 / R6** : avec Start, après 3 s, les clics tombent là où est la souris. Bouge la souris : les clics suivent.
- [ ] **R1** : en « Droit », le menu contextuel s'ouvre (clic droit).
- [ ] **R11** : clique ailleurs pour que la fenêtre perde le focus, puis appuie sur F6. Les clics démarrent tout de suite. Appuie encore sur F6 : ils s'arrêtent.
- [ ] **R12** : avec le bouton Start, l'état affiche « Démarrage dans 3 s », puis 2, puis 1, puis « En cours ».
- [ ] **R13** : F6 pendant le compte à rebours → « Arrêté », aucun clic.
- [ ] **R4** : pendant les clics, le choix du bouton et la vitesse sont grisés.
- [ ] **R15** : fermer la fenêtre pendant les clics arrête tout et quitte le programme.
- [ ] **R16** : lance une deuxième instance de l'app. Elle affiche l'avertissement « Touche F6 indisponible », et le bouton marche quand même.
- [ ] **R20** : l'icône curseur + éclair apparaît dans la barre des tâches.
- [ ] **R21** : change la vitesse et le bouton, ferme puis relance l'app. Les réglages sont conservés.
- [ ] **R23** : après `./install.sh`, tape « auto » ou « clic » dans la recherche du menu Mint. L'app apparaît avec son icône et se lance. `./install.sh --remove` la retire du menu.
