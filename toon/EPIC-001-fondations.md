**Directive Claude (à respecter à la lettre)**

* Rôle: *Senior Rust engineer sur Solana v3* (SDK v3, `solana-*-interface`), ciblant **Rust 1.90** sur **macOS M-series**.
* **Génère du code finalisé, zéro placeholder**: **interdit** d’émettre `todo!()`, `unimplemented!()`, `panic!()` non justifiés, sections vides, “exemples à adapter” ou pseudocode.
* **Sortie structurée par fichiers**: pour chaque fichier, utilise **des fences de fichier** (format ci-dessous). Un fichier = contenu intégral.
* **Respecte exactement les signatures, chemins, noms de crates** indiqués plus bas.
* **N’ajoute aucune dépendance** non listée; **Rust stable 1.90 uniquement**.
* Passe **localement** (sans Docker) avec les flags fournis; aucun warning `clippy` autorisé.
* Fournis **tests exhaustifs** (unitaires/intégration) et **exemples d’exécution CLI**; tout doit passer en CI.

Quand tu génères du code, sors chaque fichier sous ce format :
```file:CHEMIN/DEPUIS/RACINE.rs
// contenu entier du fichier, prêt à compiler
```

Ne mets **aucun autre texte** entre les blocs `file:`. Termine par un récapitulatif.

# EPIC-001 — Fondations toolchain & workspace `[P0]`

> **But pédagogique** : fournir un guide pas-à-pas pour un développeur débutant afin de disposer d'un dépôt Rust compilable sur macOS Apple Silicon, aligné sur les contraintes de performance du projet.
> **Priorité** : **P0** — gating pour toute la suite (aucun sprint aval ne démarre sans validation des journaux 001A/001B).

## Résultats attendus
- Machine Apple Silicon prête avec toolchain Rust 1.90, Solana CLI 1.18.16+, et outils système (`protobuf`, `cmake`, `llvm-profdata`).
- Workspace Rust multi-crates initialisé (`Cargo.toml`, `.cargo/config.toml`, dossiers `crates/*`, `config/*.toml`, `third-parties/`).
- Documentation interne : journaux `docs/logs/sprint-001A.md` et `docs/logs/sprint-001B.md` complétés avec captures de commandes et captures d'écran.
- Preuve qu'un `cargo build --release` (workspace vide) compile sur la machine du junior.

## Exigences "spec-as-code" & prompts robustes
- Fournir une **spécification modulaire** : chaque sprint a sa section avec inputs/outputs, dépendances et validations mesurables.
- **Acceptance criteria** : toutes les métriques citées ci-dessous doivent être recopiées dans la section `## Checklist de validation` de chaque sprint.
- Ajouter une section `### Exemples valides/invalides` lorsque le format de sortie est strict (fences de fichiers, rapports de logs).
- Réutiliser les extraits ci-dessous dans les tickets `SPRINT-001A` et `SPRINT-001B` pour que Claude Haiku 4.5 applique les mêmes rails.

## Livrables obligatoires
1. `rust-toolchain.toml` ancré sur Rust 1.90 + composants `clippy` et `rustfmt`. Fournir ce contenu exact :
   ```toml
   [toolchain]
   channel = "1.90.0"
   components = ["clippy", "rustfmt"]
   profile = "default"
   targets = ["aarch64-apple-darwin"]
   ```
2. `.cargo/config.toml` complet incluant flags Apple Silicon M-series et pipeline PGO (collect + use). Reproduire précisément :
   ```toml
   [build]
   target = "aarch64-apple-darwin"
   rustflags = [
     "-Ctarget-cpu=native",
     "-Clto=thin",
     "-Cembed-bitcode=yes",
     "-Cpanic=abort",
   ]

   [profile.release]
   debug = false
   lto = "thin"
   panic = "abort"
   codegen-units = 1

   [profile.release-with-pgo-collect]
   inherits = "release"
   debug = true
   strip = false
   profile = "generate"

   [profile.release-with-pgo-use]
   inherits = "release"
   strip = "debuginfo"
   lto = "fat"
   profile = "use"

   [target.aarch64-apple-darwin]
   linker = "clang"
   rustflags = [
     "-Ctarget-cpu=native",
     "-Clink-arg=-fuse-ld=lld",
     "-Clink-arg=-Wl,-dead_strip",
   ]
   ```
3. Bloc `[patch.crates-io]` dans `Cargo.toml` racine pointant vers les forks internes :
   ```toml
   [patch.crates-io]
   openbook-dex = { path = "third-parties/openbook-dex" }
   phoenix-sdk = { path = "third-parties/phoenix-sdk" }
   ```
4. Workflow GitHub Actions `.github/workflows/ci.yml` réalisant `fmt`, `clippy -D warnings`, `cargo test --workspace`, `cargo audit`, `cargo deny`, et un job `grep` interdisant `todo|unimplemented|panic!` dans `src/` hors dossiers de tests.
5. `justfile` avec les recettes `ci`, `fmt`, `lint`, `test`, `audit`, `deny`, `pgo-collect`, `pgo-use`, `paper`, `replay` et documentation d'usage.

## Étapes à très haute visibilité
1. SPRINT-001A — Installer la toolchain & les dépendances système.
2. SPRINT-001B — Bootstrap workspace & configuration TOML.

Chaque sprint possède sa checklist détaillée. Ne commence pas le sprint suivant tant que les critères du précédent ne sont pas validés (quitus de pair obligatoire).

## Chemin critique & points de contrôle
- **CP1 :** validation des installations (`rustc`, `cargo`, `solana`, `protoc`) consignée dans `docs/logs/sprint-001A.md` (bloqueur si absent).
- **CP2 :** workspace Rust créé, `cargo metadata` exécuté avec succès, fichiers TOML commités (`docs/logs/sprint-001B.md`).
- **CP3 :** sortie `cargo build --release` collée dans le journal (commande doit réussir sans warnings bloquants).

## Signaux d'alerte & actions
- Si `cargo build` échoue pour cause de dépendance manquante, revenir à la checklist SPRINT-001A pour vérifier `pkg-config`, `cmake`, `llvm`.
- Si `cargo metadata` est lent ou échoue, vérifier la présence des overrides `third-parties/` (ne jamais pointer vers des chemins inexistants).
- Escalade immédiate si la machine n'est pas Apple Silicon : certains flags (`target-cpu=native`) seront invalides.

## Journalisation & preuves
- Captures d'écran ou exports de terminal à déposer dans `docs/logs/sprint-001A.md` et `docs/logs/sprint-001B.md`.
- Lien vers le run `just ci` réussi (copie du résumé GitHub Actions ou sortie locale).
- Archive PGO générée (`target/pgo/default.profraw` → `default.profdata`) référencée et versionnée dans `docs/logs/pgo-notes.md`.

## Checklist de validation
- `rustc --version` == 1.90.0.
- `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`, `cargo audit`, `cargo deny`, `just ci` exécutés et archivés.
- `grep -R "todo\|unimplemented\|panic!" src --exclude-dir tests` ne retourne rien.

## Documentation à tenir à jour
- `README.md` : section "Toolchain" précisant la commande `rustup component add` et l'usage du `justfile`.
- `docs/diagrams/toolchain-pipeline.drawio` : diagramme PGO (collect/use) incluant `llvm-profdata`.
- `docs/logs/ci-history.md` : journal des runs `just ci`.

## Interactions avec les autres EPICs
- EPIC-002 dépend de la disponibilité du `justfile` (task `just replay`).
- EPIC-003 exige le job `ComputeBudget` paramétrable depuis `config/compute_budget.toml` initialisé ici.
- EPIC-004 lit les sorties `just ci` pour valider les seuils d'alertes (grep TODO/UNIMPLEMENTED).

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
