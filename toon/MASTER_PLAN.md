# Plan de livraison TOON — Projet OtterSlice

## Vue d'ensemble
- **Objectif** : Livrer un bot d'arbitrage CEX↔DEX (spot/perps) Solana 100 % fonctionnel, exécutable localement sur macOS Apple Silicon en Rust 1.90, sans agrégateur externe.
- **Cadre technique** : architecture multi-crates définie dans `README.md`, dépendances Solana v1.18, DEX Phoenix/OpenBook/Orca/Raydium, CEX Binance/OKX/Bybit.
- **Organisation** : 5 EPICs séquentielles, chacune découpée en sprints ≤ 2 jours avec instructions à pas fins pour un développeur débutant.
- **Chemin critique** : EPIC-001 → EPIC-002 → EPIC-003 → EPIC-004 → EPIC-005. Aucun sprint ne peut commencer sans le quitus formel du précédent (journal + tests).

## RACI simplifié
| Rôle | Responsabilités |
| --- | --- |
| Dev junior (exécutant) | Suivre pas-à-pas les tickets, collecter les preuves dans `docs/logs/*.md`. |
| Dev référent | Relire chaque sprint, valider les critères d'acceptation, tenir la check-list. |
| Ops/Infra | Fournir accès RPC, Slack webhook, surveiller le monitoring. |
| Chef de projet | Vérifier le respect du chemin critique et des milestones. |

## Tableau EPICs & dépendances
| Ordre | EPIC | Sprints inclus | Livrable majeur | Dépend de |
| --- | --- | --- | --- | --- |
| 1 | [EPIC-001 — Fondations toolchain & workspace](EPIC-001-fondations.md) | 001A, 001B | Workspace compilable + configs TOML | Aucun |
| 2 | [EPIC-002 — Ingestion marchés CEX/DEX](EPIC-002-ingestion.md) | 002A → 002D | Flux L2 cohérents + quotes CLMM | EPIC-001 |
| 3 | [EPIC-003 — Décision & exécution arbitrage](EPIC-003-execution.md) | 003A → 003D | Scanner + exécution CEX/DEX | EPIC-002 |
| 4 | [EPIC-004 — Gestion du risque & monitoring](EPIC-004-risque.md) | 004A → 004C | Pré-trade, métriques, kill-switch | EPIC-003 |
| 5 | [EPIC-005 — Paper trading & calibration](EPIC-005-paper.md) | 005A, 005B | Paper mode + rapport calibration | EPIC-004 |

## Milestones & artefacts attendus
| Milestone | Condition de passage | Artefacts à archiver |
| --- | --- | --- |
| M1 | `cargo build --release` réussi + journaux 001A/001B | captures toolchain, log build |
| M2 | Latence ingestion mesurée, replay diffs validé | `docs/logs/latency-benchmark.md`, diagrammes PlantUML |
| M3 | Scanner + exécution CEX/DEX démontrés en CLI | sorties CLI, tests exec, `opportunities.csv` |
| M4 | Monitoring opérationnel + kill-switch testé | capture `/metrics`, dashboard Grafana, runbook signé |
| M5 | Paper trading 72 h + rapport calibration | CSV datés, notebook, rapport signé |

## Règles d'exécution (à rappeler dans chaque sprint)
1. Toujours travailler sur macOS Apple Silicon, `rustup default 1.90.0`.
2. Copier/coller les commandes indiquées, ne pas improviser.
3. Documenter **toutes** les sorties dans `docs/logs/SPRINT-XXX.md` + ajouter captures si demandé.
4. Ne jamais commiter de secrets (utiliser Keychain via `keyring`).
5. Chaque sprint se termine par un `git status`, `cargo fmt`, `cargo test` ciblé, et une revue pair.
6. Le journal sert de preuve : sans preuve → sprint non validé.

## Plan de contrôle du chemin critique
- **Revue quotidienne** : le dev référent vérifie les journaux et coche la check-list.
- **Blocages** : si un sprint échoue à un test, revenir à l'étape correspondante et consigner la résolution.
- **Fallbacks** :
  - RPC down → basculer sur URL secondaire (`config/default.toml`).
  - API CEX limitée → réduire la cadence des tests (`sleep`).
- **Go/No-Go** : à la fin de chaque EPIC, organiser une revue 30 min avec Ops + Chef de projet (note dans `docs/logs/revue-epic-00X.md`).

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

*(Les URL servent de référence. Les développeurs doivent lire les sections indiquées avant de commencer les sprints associés.)*
