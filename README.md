# DietLock

[English](README.en.md)

Suivi quotidien du poids, des macros et de l'hydratation, avec une tendance de poids lissée. App desktop (Tauri + Rust), macOS d'abord.

- Chaque matin : saisie du poids du jour (à jeun ou non).
- Chaque soir : saisie des macros (protéines, glucides, lipides) et de l'eau ; les kcal sont calculées.
- Tendance de poids : moyenne des pesées à jeun sur 7 jours, affichée dès 4 pesées dans la fenêtre.
- Données stockées uniquement en local (SQLite), sans compte ni réseau.

**État (v0.1.0)** : saisie, historique et tendance. Pas encore d'alerte ni de design.

## Installer (macOS, Apple Silicon)

1. Télécharger le `.dmg` depuis la page [Releases](https://github.com/Bertrand2808/DietLock/releases).
2. Ouvrir le `.dmg` et glisser DietLock dans Applications.
3. Premier lancement, l'app n'étant pas signée : clic droit sur DietLock, puis Ouvrir, puis Ouvrir. Si macOS refuse (macOS 15 et plus), aller dans Réglages Système, Confidentialité et sécurité, puis cliquer sur « Ouvrir quand même ».

Les données sont dans `~/Library/Application Support/com.bertrand2808.dietlock/`.

## Compiler

```bash
cargo tauri build   # produit le .app et le .dmg dans target/release/bundle/
```

## Licence

Sous double licence, au choix :

- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT ([LICENSE-MIT](LICENSE-MIT))
