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

# SPRINT-001A — Installation toolchain `[P0]`

## Objectifs
- Installer Rust 1.90, `clippy`, `rustfmt`, Solana CLI 1.18.16+, `llvm-profdata`, `protobuf`, `cmake`, `pkg-config`.
- Générer le fichier `rust-toolchain.toml` (contenu imposé EPIC-001) et valider `rustup show`.
- Préparer le `justfile` (recettes `fmt`, `lint`, `test`, `audit`, `deny`, `ci`, `pgo-collect`, `pgo-use`).

## Étapes détaillées
1. Vérifier la machine (`uname -m` → `arm64`, `sw_vers` documenté).
2. Installer toolchain :
   ```bash
   rustup toolchain install 1.90.0 --component clippy rustfmt
   rustup default 1.90.0
   brew install protobuf cmake llvm pkg-config
   sh -c "$(curl -sSfL https://release.solana.com/v1.18.16/install)"
   ```
3. Ajouter `~/.profile` exports :
   ```bash
   export PATH="$HOME/.cargo/bin:$HOME/.local/share/solana/install/releases/1.18.16/solana-release/bin:$PATH"
   ```
4. Créer `rust-toolchain.toml` exactement comme spécifié.
5. Initialiser `justfile` minimal :
   ```make
   default: ci

   fmt:
   cargo fmt --all

   lint:
   cargo clippy --all-targets --workspace -D warnings

   test:
   cargo test --workspace

   audit:
   cargo audit

   deny:
   cargo deny check bans licenses sources advisories

   pgo-collect:
   cargo build --profile release-with-pgo-collect

   pgo-use:
   cargo build --profile release-with-pgo-use

   ci:
   just fmt
   just lint
   just test
   just audit
   just deny
   ```
6. Capturer les sorties `rustc --version`, `cargo --version`, `solana --version`, `protoc --version` dans `docs/logs/sprint-001A.md`.

## Exemples valides/invalides
- ✅ Log `rustc 1.90.0 (abcd123 2024-05-01)` collé dans le journal.
- ❌ Mention "installer rust" sans preuve exécutable.

## Livrables
- `rust-toolchain.toml` (contenu exact).
- `justfile` initialisé (sera complété sprint 001B).
- Journal `docs/logs/sprint-001A.md`.

## Checklist de validation
- `rustc --version` == `rustc 1.90.0`.
- `solana --version` >= `1.18.16`.
- `just ci` tourne jusqu’aux commandes `deny` (même si crates vides).
- `docs/logs/sprint-001A.md` contient captures + checksum `shasum -a 256 rust-toolchain.toml`.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
