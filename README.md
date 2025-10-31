# Objectif & périmètre

* **Stratégie** : arbitrage **CEX↔DEX** (spot/perps) côté **Solana**, sans agrégateur (pas de Jupiter) ni bundles MEV (pas de Jito).
* **Plateforme** : **macOS (M‑series)**, build **Rust 1.90** (optimisations CPU Apple M4), **0 abonnement** (RPC publics + fallback).
* **Style** : **Rust only**, exécution **locale**, pas de Docker.
* **Cible v1** : majors (SOL/USDC, wBTC/USDC, wETH/USDC) en **1‑hop**.

---

# Architecture (vue d’ensemble)

```
/bot
  /crates
    /common           # types, time, fees(bps), math, errors
    /config           # lecture TOML/ENV, schema
    /cex              # adapters Binance/OKX/Bybit (REST/WS)
    /dex-clob         # Phoenix + OpenBook v2 (orderbooks, place/cancel)
    /dex-clmm         # Orca Whirlpools + Raydium (math quote locale)
    /ingest           # reconstructeurs L2, index, WS drivers
    /engine           # scanner spreads net-frais + décision
    /exec             # orchestration ordres: IOC/FAK, slicing, hedging
    /risk             # caps, kill-switch, slippage p95, exposure net
    /metrics          # tracing + CSV/Parquet writer + tui (optionnel)
    /paper            # simulateurs fills, backtest offline/replay
    /cli              # binaire principal (run strat, paper, replay)
  /config
    default.toml      # conf générique
    markets.toml      # paires, tailles, caps
  /third-parties      # forks perso optimisés M4 (overrides Cargo)
  .cargo/config.toml  # rustflags M4, PGO/LTO, target-cpu=native
  Cargo.toml          # workspace + [patch.crates-io]
```

---

# Toolchain & build (optimisé M4)

**Rust 1.90** — profils et flags conseillés :

**`.cargo/config.toml`**

```toml
[build]
target = "aarch64-apple-darwin"

[env]
# Active l’auto-vectorisation et les intrinsics CPU de la machine
RUSTFLAGS = "-C target-cpu=native -C opt-level=3 -C codegen-units=1 -C lto=thin"

[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3
panic = "abort"
strip = true

[profile.release-pgo]
inherits = "release"
# Utilisé après collecte PGO (-C profile-use)
```

**PGO (optionnel mais recommandé, local)**

1. **Instrumenter** : `RUSTFLAGS+=" -Cprofile-generate=./pgo"` puis `cargo run --release -- ... --paper --scenario daily` (24–48h).
2. **Compiler profil** : `llvm-profdata merge -output=./pgo/merged.profdata ./pgo/*profraw`.
3. **Rebuild** : `RUSTFLAGS="-C target-cpu=native -C lto=thin -C profile-use=./pgo/merged.profdata" cargo build --profile release-pgo`.

> Astuces perf : évite OpenSSL (préfère `rustls`), réduis les allocations (pools d’objets), utilise `ahash`, `smallvec`.

---

# Intégration de vos forks M4 (répertoire `third-parties/`)

* Placez vos forks (tungstenite, http, parser, etc.) dans `/third-parties/<crate>`.
* **Cargo.toml (workspace)** — utiliser des **overrides** :

```toml
[workspace]
members = [
  "crates/common", "crates/config", "crates/cex", "crates/dex-clob",
  "crates/dex-clmm", "crates/ingest", "crates/engine", "crates/exec",
  "crates/risk", "crates/metrics", "crates/paper", "crates/cli"
]
resolver = "2"

[patch.crates-io]
tokio-tungstenite = { path = "third-parties/tokio-tungstenite" }
http = { path = "third-parties/http" }
# ajoutez ici toutes vos libs forkées optimisées M4
```

* **Désactiver features superflues** dans `Cargo.toml` de chaque crate (`default-features = false`).

---

# Dépendances (v3 Solana, sans Jupiter/Jito)

* **Solana v3 (clients & interfaces)** : `solana-sdk`, `solana-client`, `solana-transaction`, `solana-message`, `solana-pubkey`.
* **Interfaces v3** : `solana-compute-budget-interface`, `solana-system-interface`.
* **SPL (interfaces)** : `spl-token-interface`, `spl-associated-token-account-interface`.
* **CEX stack** : `reqwest`(rustls), `tokio`, `tokio-tungstenite`, `hmac`, `sha2`.
* **Utils** : `serde{,_json}`, `tracing`, `anyhow`, `thiserror`, `ahash`, `smallvec`, `parking_lot`, `itertools`.

> Pas d’agrégateur : pour DEX, on lit **Phoenix/OpenBook** (CLOB) et les **Whirlpools/CLMM** (Orca/Raydium) en direct.

---

# Sécurité & secrets (local macOS)

* Stocker les clés API CEX dans **macOS Keychain** via le crate `keyring`.
* Seeds Solana **offline** (fichier chiffré) + 2e wallet trésorerie.
* Aucune permission **withdraw** sur les clés CEX; IP allowlist; sub-accounts dédiés.

---

# RPC publics & politiques de rate-limit (0 abonnement)

* 2 URLs RPC **primaires** + 1 **failover** (HTTP & WS).
* Utiliser **subscriptions** (program/account/slot) plutôt que polling.
* Cache local **account/pool** et **orderbook**; backoff exponentiel sur 429/5xx.
* Sur congestion: ajouter **ComputeBudget** (`set_compute_unit_limit` + `set_compute_unit_price`) à vos transactions.

---

# Ingestion marché (temps réel)

## CEX (Binance/OKX/Bybit)

* WS **orderbook diff** + **snapshot REST** → reconstructeur L2 déterministe.
* Normaliser timestamps côté **server-time**; token-bucket rate limit.

## DEX CLOB (Phoenix/OpenBook)

* Subscriptions comptes/carnets → **best bid/ask** + profondeur disponible.
* Place/Cancel on-chain avec timeouts & retries; confirmation `confirmed` par défaut, `finalized` pour legs sensibles.

## DEX CLMM (Orca/Raydium)

* Lecture **state des pools** (tick, sqrtPriceX64, liquidity) par WS/HTTP RPC.
* **Math locale** pour `amount_out` à la taille (quote “offline”), pas d’API externe.

---

# Détecteur « spread net‑frais » (critères d’entrée)

Formule **déclencheur** :

```
spread_live ≥ fees_cex + fees_dex + gas_bps + slippage_exp + marge_secu
```

Repères initiaux : **naïf** ≥ 50–70 bps ; **optimisé** ≥ 12–20 bps.

* **Fees→bps** : maker/taker CEX, pool-fee DEX, **gas→bps**.
* **Executable price** CEX : consommer profondeur à la taille (pas VWAP agrégé).
* **Quote DEX** :

  * CLOB → consommer carnet jusqu’au volume requis.
  * CLMM → calculer impact via formules Whirlpools/CLMM.

---

# Exécution & hedge

* **CEX** : ordres **IOC/FAK** par défaut; **slicing** vs profondeur p95; retry/backoff.
* **DEX** :

  * CLOB → `place_order`/`cancel_order` Phoenix/OpenBook.
  * CLMM → instructions de **swap** construites localement; signer & soumettre.
  * Ajouter **ComputeBudget** pour la priorité (pas de Jito requis).
* **Hedge** : prioriser hedge **même venue CEX** (perp) pour réduire la latence; on‑chain ensuite si besoin.
* Boucle **détection → exé → hedge ≤ 1s** (objectif v1).

---

# Risk rails (hard)

* **Caps** : notionnel/trade & /min; **open orders max**; **exposure net** par marché.
* **Slippage p95** (bps) calculé live → **kill** si dépasse seuil.
* **Kill-switch** : WS down > 1.5s, slot‑lag élevé, spread < seuil, échec confirmation DEX.
* **Journal** des décisions + **PnL en bps**.

---

# Observabilité (locale)

* `tracing` (JSON ou pretty) avec niveaux par crate.
* **CSV/Parquet** (trades, quotes, fills, tick‑to‑trade) pour analyse.
* **TUI** (facultatif) : spreads, latence, p95 slippage, hit‑rate.

---

# Tests & qualité

* **Unitaires** : parsers WS, hachage signatures, math CLMM (contre cas connus).
* **Intégration** : reconstructeur L2 (snapshot+diff), place/cancel dummy, simulate DEX.
* **Paper trading** : simulateur de fills (CEX prix exécutable; DEX CLOB/CLMM à profondeur réelle) + PnL net-frais.
* **Replay** : ingestion de PCAP/Parquet pour revalider les seuils.

---

# Fichiers de configuration (exemples)

**`config/default.toml`**

```toml
[env]
region = "ap-sg-1"
rpc_primary = "https://rpc1.example"
rpc_failover = "https://rpc2.example"

[wallet]
path = "~/.config/solana/id.json"
commitment = "confirmed"

[cex.binance]
api_key = "keychain:binance:key"
api_secret = "keychain:binance:secret"

[thresholds_bps]
spread_min_naive = 70
spread_min_optim = 20
max_slippage_p95 = 6

[risk]
max_notional_per_trade = 0.0
max_open_orders = 16
kill_on_ws_down_ms = 1500
```

**`config/markets.toml`**

```toml
[[pairs]]
name = "SOL/USDC"
cex_symbol = "SOLUSDT"
dex_markets = ["phoenix:SOL/USDC", "orca:SOL/USDC"]
size_usd = 200

[[pairs]]
name = "wETH/USDC"
cex_symbol = "ETHUSDT"
dex_markets = ["openbook:WETH/USDC", "raydium:WETH/USDC"]
size_usd = 300
```

---

# Roadmap (calendrier exécutable)

## J0–J1 — Setup & fondations

* Installer **Rust 1.90**, Solana CLI, créer wallets.
* Bootstrap workspace + dossier **third-parties/** + `[patch.crates-io]`.
* Ajouter `.cargo/config.toml` (M4 flags) et profils.
* Valider accès **RPC** (HTTP/WS) & **Keychain** (lecture/écriture).

**Critères d’acceptation** : `cargo build --release` OK, ping RPC, keychain OK.

## J2–J3 — Ingestion CEX

* Impl WS **orderbook diff** + snapshot REST (Binance/OKX/Bybit).
* Reconstructeur L2 (tests de cohérence, latence, drop‑resync).

**CA** : latence WS<->L2 < 25ms médiane; pas de drift > 10 niveaux.

## J4–J5 — DEX CLOB (Phoenix/OpenBook)

* Subscriptions carnets, mapping symboles, best bid/ask.
* Place/Cancel dry‑run (simulation signature/submit sans commit).

**CA** : best bid/ask cohérent vs explorers; place/cancel simulés OK.

## J6 — DEX CLMM (Orca/Raydium)

* Lecture state pools; impl **math quote locale** (amount_out à taille).

**CA** : erreur quote < 5 bps vs tx simulée.

## J7–J8 — Scanner spreads net‑frais

* Impl formule complète fees→bps + gas→bps + slippage_exp.
* Dashboard CLI (spreads, occurrence/min, coûts all‑in).

**CA** : détection signaux réalistes; no false storm (>50/min) en heures creuses.

## J9–J10 — Exécution & Hedge

* CEX : ordres **IOC/FAK**, slicing p95; retry/backoff.
* DEX : transactions avec **ComputeBudget**; soumission + timeout/failover.
* Hedge CEX (perp) minimal.

**CA** : boucle détection→exé→hedge **≤ 1s** en local; confirmations stables.

## J11 — Risk rails & kill‑switch

* Caps notionnels; slippage p95 live; watchdog WS/slot‑lag.

**CA** : kill‑switch déclenché correctement sur scénarios d’erreur.

## J12–J14 — Paper & calibration

* Run **48–72h** en paper; collecte métriques.
* Ajuster seuils (20→30 bps si trop de faux positifs), tailles & compute units.

**CA** (go/no‑go): hit‑rate ≥ 35 %, p95 slippage ≤ 6 bps, PnL net ≥ 10 bps/j (notionnel), zéro incident critique.

---

# Check‑lists (opérationnelles)

## Démarrage quotidien

* [ ] NTP/horloge OK; RPC primaire en bonne santé (slot‑lag < seuil).
* [ ] WS CEX/DEX connectés; reconstructeurs L2 synchros.
* [ ] Risk caps chargés; seuils bps conformes; keychain accessible.

## Avant prod (petit nominal)

* [ ] Paper 72h validé (CA ci‑dessus).
* [ ] Hedge même venue testé; latence  p95 < 1s.
* [ ] Logs & CSV vérifiés; runbooks incidents prêts.

---

# Commandes utiles

```
# Build M4 optimisé
cargo build --release

# Paper trading (ex.)
cargo run --release -- --mode paper --config ./config/default.toml --markets ./config/markets.toml

# Replay
cargo run --release -- --mode replay --parquet ./replay/sol_usdc_2025-xx-xx.parquet

# Profiler PGO (collecte)
RUSTFLAGS+=" -Cprofile-generate=./pgo" cargo run --release -- --mode paper

# Profiler PGO (usage)
llvm-profdata merge -output=./pgo/merged.profdata ./pgo/*.profraw
RUSTFLAGS="-C target-cpu=native -C lto=thin -C profile-use=./pgo/merged.profdata" \
  cargo build --profile release-pgo
```

---

# Notes finales

* **Viabilité** : forte pour un premier jet **local Rust** sans abonnement, si l’on reste sur 1‑hop majors et qu’on privilégie WS + caches.
* **Évolutivité** : avec le nominal, envisager à terme un **RPC dédié** (ou nœud) pour inclusion/priorité robustes — non requis pour v1.
* **Forks M4** : suivez `[patch.crates-io]` + désactivation de features inutiles; gardez une branche synchronisée avec amont pour sécurité.
