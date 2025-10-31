## EPIC-006 — Auto-tuning & Profitabilité

### Objectif
Mettre en place une **boucle d’optimisation en ligne** (et hors-ligne via replay) qui ajuste des **paramètres finetuables** (seuils, tailles, compute units, etc.) pour **maximiser la PnL nette en bps** sous **contraintes de risque**.

### Portée
- Exposition explicite des **paramètres tunables** (listés ci-dessous) avec **bornes**, **pas**, **valeurs par défaut**.
- Ajout d’un **mode `--mode tune`** (supporté par `scripts/run_bot_mainnet.sh`) qui pilote la boucle d’optimisation.
- Implémentation d’un **tuner** (bandit contextuel / bayésien simple) avec:
  - *Offline seeding* via replay (parquets),
  - *Online exploration* **safe** (caps et guardrails),
  - **Early-stopping** et **rollback**.
- **Objectif unique**: maximiser `Score = E[PNL_net_bps] − λ·RiskPenalty`, sous contraintes (voir Fonction-objectif).

### Fonction-objectif & Contraintes
- **KPI primaires**: `PNL_net_bps/jour`, `hit_rate`, `slippage_p95_bps`, `latency_ms`.
- **Score**: `PNL_net_bps − λ1·max(0, slippage_p95_bps−6) − λ2·max(0, latency_ms−1000) − λ3·RiskEvents`
- **Contraintes dures (hard)**:
  - `slippage_p95_bps ≤ 6`,
  - `latency_ms ≤ 1000`,
  - `max_drawdown_30j ≤ 5%`,
  - `kill_switch` actif (WS down > 1.5s, slot-lag, spread < seuil, échec confirm).
- **Capital d’exploration**: plafonné par **caps** (notional/trade & /min) — fixé dans `run_bot_mainnet.sh`.

### Paramètres finetuables (exposés)
(Tous lisibles via `std::env` **depuis `run_bot_mainnet.sh`** — aucun dur-codage Rust)
- **Seuils & coûts**
  - `SPREAD_MIN_BPS`  (default: 20, bounds: [12, 40])
  - `SPREAD_MIN_BPS_NAIVE` (70, [40, 100])
  - `MARGE_SECURITE_BPS` (4, [0, 12])
  - `GAS_TO_BPS_FACTOR` (auto, [0, 20])
- **Slippage & sizing**
  - `MAX_SLIPPAGE_P95_BPS` (6, [3, 10])
  - `SIZING_USD_PER_TRADE` (200, [50, 2000])  # par paire
  - `OCCURRENCE_FILTER_N_PER_XMIN` (3/5min, N∈[1,10], X∈[1,15])
- **Priorité on-chain**
  - `CU_LIMIT` (200_000, [80k, 400k])
  - `CU_PRICE_MICROLAMPORT` (120_000, [20k, 400k])  # priority fee sans Jito
- **Hedge & exé**
  - `HEDGE_LEVERAGE` (1.0, [0.5, 5.0])
  - `TIME_IN_FORCE` (IOC/FAK; mapping par CEX)
- **Tuning (méta)**
  - `TUNE_STRATEGY` (grid|random|bayes|bandit)
  - `TUNE_BUDGET_ITERS` (200, [20, 1000])
  - `TUNE_EPSILON` (0.08, [0.0, 0.2])  # pour epsilon-greedy
  - `TUNE_ROLLING_HOURS` (24, [6, 72])

### Boucle d’optimisation (spécification)
1) **Offline seeding** (*replay*): le tuner évalue K configurations sur données Parquet et initialise des priors (bayésien) ou un ranking (bandit).
2) **Online tuning**:
   - À chaque **épisode** (ex. 30 min), proposer une config dans l’espace borné,
   - **Respecter** les **guardrails** (caps, contraintes dures),
   - Journaliser métriques → calcul du `Score`,
   - **Mise à jour** des croyances/prior (bandit/bayes),
   - **Early-stop** si `Score` < seuil pendant N épisodes; **rollback** à la meilleure config connue.
3) **Contextes** (optionnel P2): heure locale (bucket), venue, volatilité (low/med/high) comme features du bandit contextuel.
4) **Persist**: écrire la meilleure config dans `./config/tuned/<timestamp>.toml` + symlink `current.toml`.

### Sécurité (mainnet)
- **Tests mainnet réels**: **autorisés sans limitation**.
- Exploration **safe**: nominal réduit, caps stricts, kill-switch actif.
- **Aucun secret/URL dans le Rust**: injection **uniquement** via `scripts/run_bot_mainnet.sh`.

### Livrables
- Crate **`tuner/`** + intégration dans `cli` (`--mode tune`).
- `config/tuning.toml` (bornes, pas par paramètre).
- Persist de résultats `./runs/tuning/*.json` + `./config/tuned/*.toml`.
- Docs module-level + README tuning.

### DoD
- ✅ `--mode tune` exécute **offline seeding** puis **online tuning** (paper ou live).
- ✅ 200 itérations finissent sans crash; best config persistée; rollback testé.
- ✅ Score > baseline (≥ +10% PNL_net_bps sur 24h paper) **ou** réduction slippage p95.
- ✅ CI verte; 0 TODO/UNIMPLEMENTED; clippy zéro warning.
