# EPIC-001 — Fondations toolchain & workspace

> **But pédagogique** : fournir un guide pas-à-pas pour un développeur débutant afin de disposer d'un dépôt Rust compilable sur macOS Apple Silicon, aligné sur les contraintes de performance du projet.

## Résultats attendus
- Machine Apple Silicon prête avec toolchain Rust 1.90, Solana CLI 1.18.16+, et outils système (`protobuf`, `cmake`, `llvm-profdata`).
- Workspace Rust multi-crates initialisé (`Cargo.toml`, `.cargo/config.toml`, dossiers `crates/*`, `config/*.toml`, `third-parties/`).
- Documentation interne : journaux `docs/logs/sprint-001A.md` et `docs/logs/sprint-001B.md` complétés avec captures de commandes et captures d'écran.
- Preuve qu'un `cargo build --release` (workspace vide) compile sur la machine du junior.

## Étapes à très haute visibilité
1. SPRINT-001A — Installer la toolchain & les dépendances système.
2. SPRINT-001B — Bootstrap workspace & configuration TOML.

Chaque sprint possède sa checklist détaillée. Ne commence pas le sprint suivant tant que les critères du précédent ne sont pas validés (quitus de pair obligatoire).

## Dépendances externes
- Accès internet non filtré (pour `curl`, `brew`, `git clone`).
- Comptes GitHub (lecture) si l'on souhaite récupérer des templates de crates.
- Droits administrateur macOS pour installer la toolchain.

## Chemin critique & points de contrôle
- **CP1 :** validation des installations (`rustc`, `cargo`, `solana`, `protoc`) consignée dans `docs/logs/sprint-001A.md` (bloqueur si absent).
- **CP2 :** workspace Rust créé, `cargo metadata` exécuté avec succès, fichiers TOML commités (`docs/logs/sprint-001B.md`).
- **CP3 :** sortie `cargo build --release` collée dans le journal (commande doit réussir sans warnings bloquants).

## Signaux d'alerte & actions
- Si `cargo build` échoue pour cause de dépendance manquante, revenir à la checklist SPRINT-001A pour vérifier `pkg-config`, `cmake`, `llvm`.
- Si `cargo metadata` est lent ou échoue, vérifier la présence des overrides `third-parties/` (ne jamais pointer vers des chemins inexistants).
- Escalade immédiate si la machine n'est pas Apple Silicon : certains flags (`target-cpu=native`) seront invalides.
