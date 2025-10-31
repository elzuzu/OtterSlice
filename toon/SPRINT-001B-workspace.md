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

# SPRINT-001B — Bootstrap workspace `[P0]`

## Objectifs
- Initialiser workspace multi-crates (`Cargo.toml` racine + `crates/{ingest,execution,calibration}/Cargo.toml`).
- Créer `.cargo/config.toml` complet (voir EPIC-001) + `config/compute_budget.toml` (`compute_limit = 150000`, `compute_price_micro_lamports = 0`).
- Ajouter `[patch.crates-io]` vers `third-parties/openbook-dex` et `third-parties/phoenix-sdk`.
- Étendre `justfile` avec recettes `paper`, `replay`, `ci` orchestrant `fmt`→`lint`→`test`→`audit`→`deny`→`todo-scan`.

## Étapes détaillées
1. Créer dossiers : `mkdir -p crates/{ingest,execution,calibration}/src config docs/logs .github/workflows`.
2. Générer `Cargo.toml` racine :
   ```toml
   [workspace]
   members = [
     "crates/ingest",
     "crates/execution",
     "crates/calibration",
   ]
   resolver = "2"

   [workspace.package]
   edition = "2021"
   version = "0.1.0"
   authors = ["OtterSlice"]
   license = "Apache-2.0"

   [patch.crates-io]
   openbook-dex = { path = "third-parties/openbook-dex" }
   phoenix-sdk = { path = "third-parties/phoenix-sdk" }
   ```
3. Créer `.cargo/config.toml` avec le contenu imposé.
4. `config/compute_budget.toml` :
   ```toml
   compute_limit = 150000
   compute_price_micro_lamports = 0
   ```
5. Mettre à jour `justfile` :
   ```make
   todo-scan:
   rg --fixed-strings --hidden --glob '!tests/**' --glob '!*.md' "todo" src
   rg --fixed-strings --hidden --glob '!tests/**' --glob '!*.md' "unimplemented" src
   rg --fixed-strings --hidden --glob '!tests/**' --glob '!*.md' "panic!" src

   paper:
   cargo run -p paper -- --from 2024-01-01T00:00:00Z --to 2024-01-03T23:59:59Z --dataset data/paper/btc_usdt_72h.parquet

   replay:
   cargo run -p ingest --bin replayer -- --dataset fixtures/replay/binance_btcusdt_1k.json

   ci:
   just fmt
   just lint
   just test
   just audit
   just deny
   just todo-scan
   ```
6. Créer workflow `.github/workflows/ci.yml` :
   ```yaml
   name: CI

   on:
     push:
       branches: [ main ]
     pull_request:

   jobs:
     build:
       runs-on: macos-latest
       steps:
         - uses: actions/checkout@v4
         - uses: actions/cache@v4
           with:
             path: |
               ~/.cargo/registry
               ~/.cargo/git
               target
             key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
         - name: Install toolchain
           uses: dtolnay/rust-toolchain@stable
           with:
             toolchain: 1.90.0
             components: clippy, rustfmt
         - name: Install cargo-audit
           run: cargo install cargo-audit --locked
         - name: Install cargo-deny
           run: cargo install cargo-deny --locked
         - name: Just CI
           run: just ci
   ```
7. Vérifier `cargo metadata` et `just ci` (logs dans `docs/logs/sprint-001B.md`).

## Exemples valides/invalides
- ✅ `cargo metadata --format-version 1` renvoie `"packages": []` (workspace vide) et log collé.
- ❌ Workflow CI sans job `todo-scan`.

## Checklist de validation
- `cargo metadata` OK.
- `just ci` passe sans warnings.
- Workflow `ci.yml` validé par `act` ou review (coller log `just ci`).
- `docs/logs/sprint-001B.md` contient diff `git diff` sur `Cargo.toml` et `.cargo/config.toml`.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
