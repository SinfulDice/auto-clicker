# Auto-clicker : spécification

Un auto-clicker pour Linux Mint (session **X11**), écrit en **Rust**, avec une petite interface graphique.

Statut : **validée** (2026-10-29)

## Choix techniques

- Langage : Rust (édition 2024).
- Interface : `eframe` / `egui`.
- Simulation des clics : `enigo` (sous X11, il passe par l'extension XTest).
- Touche globale F6 : `global-hotkey`.
- Sauvegarde des réglages : `serde` + `toml`, fichier dans `~/.config/auto-clicker/config.toml`.

## Règles v1

### Réglages
- **R1** : On choisit le bouton de la souris : **gauche** ou **droit**. Par défaut : gauche.
- **R2** : On choisit la vitesse en **clics par seconde (CPS)**, un **entier de 1 à 1000**. Par défaut : 10.
- **R3** : Une valeur de CPS hors limites est ramenée dans la plage autorisée (0 → 1, 5000 → 1000).
- **R4** : Les réglages ne sont pas modifiables pendant que l'auto-clicker tourne ou pendant le compte à rebours (champs grisés).

### Clics
- **R5** : Chaque clic est un **simple clic** (appui puis relâchement) du bouton choisi.
- **R6** : Le clic a lieu **à la position actuelle du curseur**. On peut bouger la souris pendant que ça clique.
- **R7** : L'intervalle entre deux clics vaut `1 / CPS` seconde (10 CPS → 100 ms, 1000 CPS → 1 ms).
- **R8** : Le rythme ne dérive pas : le clic n° k est prévu à `départ + k × intervalle`, sans se baser sur la fin du clic précédent. Si l'ordinateur prend du retard, les clics en retard ne sont **pas** envoyés en rafale pour rattraper : on repart du moment présent.
- **R9** : Les clics continuent **jusqu'à l'arrêt** (pas de limite de nombre ni de durée).

### Démarrer et arrêter
- **R10** : Le bouton **Start/Stop** de la fenêtre démarre ou arrête les clics.
- **R11** : La touche **F6** démarre ou arrête les clics, **même si la fenêtre n'a pas le focus**.
- **R12** : Le démarrage par le **bouton** lance un **compte à rebours de 3 s** avant le premier clic, pour laisser le temps d'éloigner la souris de la fenêtre. Le démarrage par **F6** est **immédiat**.
- **R13** : Appuyer sur Stop (bouton ou F6) pendant le compte à rebours l'annule : aucun clic n'est envoyé.
- **R14** : Après un arrêt, plus aucun clic n'est envoyé (au plus un clic déjà en cours d'envoi).
- **R15** : Fermer la fenêtre arrête les clics et quitte proprement le programme.
- **R16** : Si F6 ne peut pas être réservée (une autre application l'utilise déjà), l'app fonctionne quand même avec le bouton et affiche un avertissement.

### Affichage
- **R17** : La fenêtre affiche l'**état** : « Arrêté », « Démarrage dans N s » ou « En cours ».
- **R18** : La fenêtre affiche le **nombre de clics** envoyés depuis le dernier démarrage. Il repart à 0 à chaque démarrage.
- **R19** : La fenêtre affiche le rappel « F6 = Start/Stop ».
- **R20** : La fenêtre a sa propre **icône** : un curseur de souris avec un éclair jaune. La source est en SVG (`assets/icon.svg`), avec un export PNG pour la fenêtre.

### Sauvegarde
- **R21** : Le bouton et le CPS sont enregistrés dans `~/.config/auto-clicker/config.toml` dès qu'ils changent, et rechargés au lancement.
- **R22** : Si le fichier est absent, illisible ou contient des valeurs invalides, on utilise les valeurs par défaut (ou on les corrige selon R3), **sans planter**.

## Tests

- Les règles de logique (R1–R3, R7, R8, R12–R14, R17, R18, R21, R22) ont des tests automatiques (`cargo test`). Le nom de chaque test contient le numéro de la règle.
- Pour pouvoir tester sans vraie souris, le « cliqueur » est une interface (un *trait*) : les tests utilisent un faux cliqueur qui compte les clics.
- Ce qui touche le vrai bureau (R5, R6, R11, R15, R16, R20) se vérifie à la main, avec la checklist de `TESTING.md`.

## Plus tard (pas en v1)

- Double clic.
- Clic à une position fixe (X, Y).
- Arrêt automatique après N clics ou une durée donnée.
- Touche globale modifiable.
- Support de Wayland.
- Paquet `.deb` ou entrée dans le menu des applications.
