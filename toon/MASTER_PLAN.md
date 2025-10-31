# Plan de livraison TOON — Projet OtterSlice

## Vue d'ensemble
- **Objectif** : Livrer un bot d'arbitrage CEX↔DEX (spot/perps) Solana 100 % fonctionnel, exécutable localement sur macOS Apple Silicon en Rust 1.90, sans agrégateur externe.
- **Cadre technique** : architecture multi-crates définie dans `README.md`, dépendances Solana v1.18, DEX Phoenix/OpenBook/Orca/Raydium, CEX Binance/OKX/Bybit.
- **Organisation** : 5 EPICs séquentielles, chacune découpée en sprints ≤ 2 jours avec instructions à pas fins pour un développeur débutant.
- **Chemin critique** : EPIC-001 → EPIC-002 → EPIC-003 → EPIC-004 → EPIC-005. Aucun sprint ne peut commencer sans le quitus formel du précédent (journal + tests).

### Priorisation P0/P1/P2 (+Stretch)
- **P0 — 100 % fonctionnel** : livrer une application exploitable et sûre couvrant les EPIC-001 à EPIC-003, le pré-trade & kill-switch de l'EPIC-004 et la campagne paper 60–72 h de l'EPIC-005.
- **P1 — +10 % hardening & opérabilité** : monitoring complet, calibration détaillée et ajouts DX/perf (auto-recovery WS, token-bucket affiné, failover RPC, autotune priority fees, `justfile`, CI strict).
- **P2 — +10 % qualité de signal & robustesse** : occurrence filter, sizing adaptatif, replayer e2e, redondance DEX, TUI opérateur.
- **Stretch (130–140 %)** : hedge Drift (paper→faible nominal), PID priority fees, watchdogs horaires/RPC/PnL.

## RACI simplifié
| Rôle | Responsabilités |
| --- | --- |
| Dev junior (exécutant) | Suivre pas-à-pas les tickets, collecter les preuves dans `docs/logs/*.md`. |
| Dev référent | Relire chaque sprint, valider les critères d'acceptation, tenir la check-list. |
| Ops/Infra | Fournir accès RPC, Slack webhook, surveiller le monitoring. |
| Chef de projet | Vérifier le respect du chemin critique et des milestones. |

## Tableau EPICs & dépendances
| Ordre | EPIC | Priorité | Sprints inclus | Livrable majeur | Dépend de |
| --- | --- | --- | --- | --- | --- |
| 1 | [EPIC-001 — Fondations toolchain & workspace](EPIC-001-fondations.md) | **P0** | 001A, 001B | Workspace compilable + configs TOML | Aucun |
| 2 | [EPIC-002 — Ingestion marchés CEX/DEX](EPIC-002-ingestion.md) | **P0** | 002A → 002D | Flux L2 cohérents + quotes CLMM | EPIC-001 |
| 3 | [EPIC-003 — Décision & exécution arbitrage](EPIC-003-execution.md) | **P0** | 003A → 003D | Scanner + exécution CEX/DEX | EPIC-002 |
| 4 | [EPIC-004 — Gestion du risque & monitoring](EPIC-004-risque.md) | **P0/P1** | 004A → 004C | Pré-trade, métriques, kill-switch | EPIC-003 |
| 5 | [EPIC-005 — Paper trading & calibration](EPIC-005-paper.md) | **P0/P1** | 005A, 005B | Paper mode + rapport calibration | EPIC-004 |

### Backlog priorisé (objectif 120 %)
**P0 — App fonctionnelle & sûre (100 %)**
- SPRINT-001A toolchain, SPRINT-001B workspace.
- SPRINT-002A → 002D : connecteurs REST/WS CEX, reconstructeur order book, DEX CLOB & CLMM.
- SPRINT-003A → 003D : conversion fees en bps, scanner net_spread, exécution CEX, hedge DEX avec ComputeBudget sans Jito.
- SPRINT-004A pré-trade, SPRINT-004C kill-switch (slot-lag, échecs exé, DD bps) + runbook & CLI d'armement.
- SPRINT-005A paper trading 60–72 h avec KPIs : hit-rate ≥ 35 %, p95 ≤ 6 bps, PnL net ≥ 10 bps/j.

**P1 — Hardening, opérabilité & perf (+10 %)**
- SPRINT-004B monitoring `/metrics`, dashboard Grafana, alertes Slack/webhook.
- SPRINT-005B calibration : analyse stats, rapport, mise à jour `config/*.toml`.
- Ajouts DX/perf : auto-recovery WS (resubscribe + gap-fill), token-bucket RL affiné, failover RPC + backoff exp, autotune `compute_unit_price`, `justfile` build/paper/replay/PGO, CI fmt+clippy `-D warnings`+tests+audit/deny.

**P2 — Qualité de signal & robustesse (+10 %)**
- Occurrence filter & sizing adaptatif suivant profondeur.
- Auto-calibration hebdo des seuils.
- Replayer e2e (Parquet) pour qualification offline.
- Redondance DEX par paire (Phoenix↔OpenBook ou Orca↔Raydium).
- TUI opérateur minimal (`toon/`).

**Stretch (130–140 %)**
- Hedge on-chain Drift (paper puis faible nominal).
- PID simple pour auto-throttle compute units/priority fees.
- Watchdogs horloge/RPC health/écart PnL vs attendu + alertes locales.

## Milestones & artefacts attendus
| Milestone | Priorité | Condition de passage | Artefacts à archiver |
| --- | --- | --- | --- |
| M1 | P0 | `cargo build --release` réussi + journaux 001A/001B | captures toolchain, log build |
| M2 | P0 | Latence ingestion mesurée, replay diffs validé | `docs/logs/latency-benchmark.md`, diagrammes PlantUML |
| M3 | P0 | Scanner + exécution CEX/DEX démontrés en CLI | sorties CLI, tests exec, `opportunities.csv` |
| M4 | P1 | Monitoring opérationnel + kill-switch testé | capture `/metrics`, dashboard Grafana, runbook signé |
| M5 | P1 | Paper trading 72 h + rapport calibration | CSV datés, notebook, rapport signé |

## Règles d'exécution (à rappeler dans chaque sprint)
1. Toujours travailler sur macOS Apple Silicon, `rustup default 1.90.0`.
2. Copier/coller les commandes indiquées, ne pas improviser.
3. Documenter **toutes** les sorties dans `docs/logs/SPRINT-XXX.md` + ajouter captures si demandé.
4. Ne jamais commiter de secrets (utiliser Keychain via `keyring`).
5. Chaque sprint se termine par un `git status`, `cargo fmt`, `cargo test` ciblé, et une revue pair.
6. Le journal sert de preuve : sans preuve → sprint non validé.
7. Respecter la priorisation : pas de tâche P1/P2 tant que tout le périmètre P0 n'est pas livré (pre-trade actif, kill-switch testé, paper 60–72 h validé).

## Plan de contrôle du chemin critique
- **Revue quotidienne** : le dev référent vérifie les journaux et coche la check-list.
- **Blocages** : si un sprint échoue à un test, revenir à l'étape correspondante et consigner la résolution.
- **Fallbacks** :
  - RPC down → basculer sur URL secondaire (`config/default.toml`).
  - API CEX limitée → réduire la cadence des tests (`sleep`).
- **Go/No-Go P0** : autorisation de nominal uniquement si les KPIs paper (hit-rate ≥ 35 %, p95 ≤ 6 bps, PnL net ≥ 10 bps/j) sont atteints, le pré-trade est actif et le kill-switch armé/testé.
- **Go/No-Go P1** : monitoring `/metrics` + dashboard Grafana opérationnels, rapport de calibration appliqué.
- **Go/No-Go P2** : occurrence filter et sizing adaptatif déployés, replayer e2e validé sur 24 h.

## Documentation & ressources officielles
- Solana CLI & RPC : https://docs.solanalabs.com/cli
- Phoenix SDK Rust : https://github.com/Ellipsis-Labs/phoenix-sdk
- OpenBook v2 : https://openbook.dex.so/docs/
- Orca Whirlpools : https://docs.orca.so/developer-resources/whirlpools
- Raydium CLMM : https://docs.raydium.io/developer-resources/amm-and-clmm
- Binance Spot API : https://binance-docs.github.io/apidocs/spot/en/
- OKX API v5 : https://www.okx.com/docs-v5/en
- Bybit Unified API : https://bybit-exchange.github.io/docs
- Crate `keyring` : https://docs.rs/keyring
- Tokio runtime : https://docs.rs/tokio

## Micro-gaps à surveiller (objectif 120 %)
1. **CI & `justfile`** : non codifiés à date — à ajouter pendant SPRINT-004B (P1 hardening).
2. **Autotune priority fees & occurrence filter** : prévoir les hooks dès P1/P2 pour éviter la dette technique.
3. **Redondance DEX** : planifier la double connectivité (Phoenix↔OpenBook, Orca↔Raydium) lors du passage en P2.
4. **TUI opérateur** : à prototyper en P2 pour la supervision locale (tableau spreads/fills/latence).

*(Les URL servent de référence. Les développeurs doivent lire les sections indiquées avant de commencer les sprints associés.)*
