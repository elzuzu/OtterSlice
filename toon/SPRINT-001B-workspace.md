# SPRINT-001B — Bootstrap workspace & configuration TOML

> **Important :** ne saute aucune instruction. Ce sprint doit aboutir à un workspace Rust compilable avec un `cargo build --release` documenté.

## Ce que tu dois avoir terminé avant de commencer
- SPRINT-001A validé, y compris le fichier `docs/logs/sprint-001A.md` et la capture du trousseau.
- Terminal configuré avec `rustc 1.90.0`, `cargo 1.90.0`, `solana --version` fonctionnel.

## Résultats livrables attendus
1. Arborescence du projet conforme au README (`bot/crates/...`, `bot/config`, `bot/.cargo`, `bot/third-parties`).
2. Workspace `Cargo.toml` listant chaque crate, avec dépendances communes et overrides prêts.
3. Fichiers `config/default.toml` et `config/markets.toml` écrits et validés (`toml` syntaxe OK).
4. Build `cargo build --release` réussi, preuve collée dans `docs/logs/sprint-001B.md`.
5. Binaire `target/release/otterslice` imprimant "OtterSlice bootstrap" lorsqu'on l'exécute.

## Étapes détaillées
1. **Créer un journal de sprint**
   - `mkdir -p docs/logs` (si pas déjà fait).
   - `cat > docs/logs/sprint-001B.md <<'MD'` puis colle le modèle :
     ```markdown
     # Sprint 001B — Journal bootstrap workspace
     - Date : <JJ/MM/AAAA>
     - Opérateur : <Ton nom>

     ## Commandes exécutées
     ```
   - Chaque fois que tu exécutes une commande, copie la commande ET sa sortie dans ce fichier.
2. **Initialiser (ou vérifier) le dépôt Git**
   - Dans le dossier racine du projet : `git status` doit répondre.
   - Si le dépôt n'existe pas : `git init` puis `git branch -m main`.
3. **Créer l'arborescence exacte**
   - Commande unique :
     ```bash
     mkdir -p bot/crates/{common,config,cex,dex-clob,dex-clmm,ingest,engine,exec,risk,metrics,paper,cli}
     mkdir -p bot/config bot/third-parties bot/.cargo
     ```
   - Vérifie : `find bot -maxdepth 2 -type d | sort` et colle la sortie dans le journal.
4. **Générer les crates Rust**
   - Pour chaque crate librairie :
     ```bash
     for CRATE in common config cex dex-clob dex-clmm ingest engine exec risk metrics paper; do
       cargo new --lib bot/crates/$CRATE --vcs none
     done
     ```
   - Pour la crate binaire :
     ```bash
     cargo new --bin bot/crates/cli --vcs none --name otterslice
     ```
   - Supprime les dossiers `.git` créés par `cargo new` (`rm -rf bot/crates/*/.git`).
5. **Écrire `bot/Cargo.toml`**
   - Ouvre le fichier et colle le template suivant en adaptant si nécessaire :
     ```toml
     [workspace]
     members = [
       "crates/common", "crates/config", "crates/cex", "crates/dex-clob",
       "crates/dex-clmm", "crates/ingest", "crates/engine", "crates/exec",
       "crates/risk", "crates/metrics", "crates/paper", "crates/cli"
     ]
     resolver = "2"

     [workspace.package]
     edition = "2021"
     version = "0.1.0"
     license = "Proprietary"

     [workspace.dependencies]
     anyhow = { version = "1", default-features = false }
     thiserror = "1"
     serde = { version = "1", features = ["derive"], default-features = false }
     serde_json = { version = "1", default-features = false }
     tokio = { version = "1", features = ["rt-multi-thread", "macros"], default-features = false }
     tracing = "0.1"
     tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"], default-features = false }
     ahash = "0.8"
     smallvec = "1"
     parking_lot = "0.12"

     [patch.crates-io]
     tokio-tungstenite = { path = "third-parties/tokio-tungstenite" }
     http = { path = "third-parties/http" }
     ```
   - Ajoute dans `docs/logs/sprint-001B.md` un bloc ```toml``` avec le contenu final.
6. **Configurer `.cargo/config.toml`**
   - Contenu recommandé :
     ```toml
     [build]
     target = "aarch64-apple-darwin"

     [env]
     RUSTFLAGS = "-C target-cpu=native -C opt-level=3 -C codegen-units=1 -C lto=thin"

     [profile.release]
     lto = "thin"
     codegen-units = 1
     opt-level = 3
     panic = "abort"
     strip = true

     [profile.release-pgo]
     inherits = "release"
     ```
   - Sauvegarde puis exécute `cat bot/.cargo/config.toml` et colle la sortie dans le journal.
7. **Rédiger `config/default.toml`**
   - Crée le fichier `bot/config/default.toml` avec ce contenu :
     ```toml
     [rpc]
     primary_http = "https://api.mainnet-beta.solana.com"
     primary_ws = "wss://api.mainnet-beta.solana.com"
     secondary_http = "https://solana-api.projectserum.com"
     secondary_ws = "wss://solana-api.projectserum.com"

     [keychain]
     cex_service = "OtterSlice"
     solana_wallet_path = "~/.config/otterslice/solana.key"

     [risk]
     max_notional_usd = 25000
     per_market_cap_usd = 5000
     max_open_positions = 3

     [execution]
     max_inflight_txs = 4
     submit_timeout_ms = 1800
     ```
   - Vérifie la syntaxe via `python -c "import tomli; tomli.load(open('bot/config/default.toml','rb'))"` (aucune sortie = OK).
8. **Rédiger `config/markets.toml`**
   - Contenu de base :
     ```toml
     [[pairs]]
     name = "SOL/USDC"
     cex_symbol = "SOLUSDT"
     dex_markets = ["phoenix:SOL/USDC", "orca:SOL/USDC"]
     size_usd = 200

     [[pairs]]
     name = "wBTC/USDC"
     cex_symbol = "BTCUSDT"
     dex_markets = ["openbook:WBTC/USDC", "raydium:WBTC/USDC"]
     size_usd = 300

     [[pairs]]
     name = "wETH/USDC"
     cex_symbol = "ETHUSDT"
     dex_markets = ["openbook:WETH/USDC", "raydium:WETH/USDC"]
     size_usd = 300
     ```
   - Vérifie via `python -c "import tomli; tomli.load(open('bot/config/markets.toml','rb'))"`.
9. **Créer des stubs Rust minimaux**
   - `echo "pub fn init_tracing() {}" > bot/crates/common/src/lib.rs`
   - `cat <<'RS' > bot/crates/cli/src/main.rs`
     ```rust
     fn main() {
         println!("OtterSlice bootstrap");
     }
     ```
     RS
10. **Relier la crate CLI aux autres**
    - Dans `bot/crates/cli/Cargo.toml`, ajoute :
      ```toml
      [dependencies]
      common = { path = "../common" }
      ```
    - Vérifie que `[package]` contient `edition = "2021"`.
11. **Tester le workspace**
    - Depuis `bot/` :
      ```bash
      cargo fmt
      cargo metadata --format-version 1
      cargo build --release
      ```
    - Copie les sorties (réussites) dans `docs/logs/sprint-001B.md`.
12. **Exécuter le binaire**
    - Commande : `./target/release/otterslice`
    - La sortie doit être `OtterSlice bootstrap`. Colle-la dans le journal.

## Tests & critères d'acceptation
- ✅ `cargo metadata --format-version 1` renvoie un JSON valide (copier/coller dans le journal).
- ✅ `cargo build --release` termine sans erreur (note la durée approximative dans le journal).
- ✅ `./target/release/otterslice` affiche "OtterSlice bootstrap".
- ✅ Les vérifications `tomli` sur les fichiers TOML n'émettent aucune exception.
- ✅ `docs/logs/sprint-001B.md` contient les commandes + sorties + observations.

## Dépendances
- Dépend de SPRINT-001A.
- Débloque tous les sprints de l'EPIC-002.

## Actions de secours
- Si `cargo build` échoue pour faute de dépendances externes (`protoc`, `cmake` introuvables), reviens au sprint 001A et réinstalle l'outil manquant.
- Si `cargo fmt` installe rustfmt automatiquement, accepte l'installation (appuie sur `y` quand rustup demande une confirmation).
- En cas d'erreur TOML, utilise `toml-lint` (`brew install taplo-cli`) pour identifier la ligne fautive.
