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

# Plan de livraison TOON — Projet OtterSlice

> **Rappel Claude Haiku 4.5** : modèle optimisé pour le coding agentique rapide/fiable, mais ne devine pas les non-dits. Spécifie toujours versions, chemins, signatures et seuils numériques. Utilise les sections "Exemples" et "Checklists" pour verrouiller les sorties.

## Vue d'ensemble
- **Objectif** : Livrer un bot d'arbitrage CEX↔DEX (spot/perps) Solana 100 % fonctionnel, exécutable localement sur macOS Apple Silicon en Rust 1.90, sans agrégateur externe.
- **Cadre technique** : architecture multi-crates définie dans `README.md`, dépendances Solana v1.18, DEX Phoenix/OpenBook/Orca/Raydium, CEX Binance/OKX/Bybit.
- **Organisation** : 5 EPICs séquentielles, chacune découpée en sprints ≤ 2 jours avec instructions à pas fins pour un développeur débutant.
- **Chemin critique** : EPIC-001 → EPIC-002 → EPIC-003 → EPIC-004 → EPIC-005. Aucun sprint ne peut commencer sans le quitus formel du précédent (journal + tests).
- **Spec-as-code** : chaque ticket commence par la directive ci-dessus, inclut un bloc "fences de fichiers" et se termine par la DoD gatekeeper.

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
- Monitoring exhaustif + webhook retries.
- Auto-calibration + rapport diffable.
- CI stricte (`just ci`, `cargo audit`, `cargo deny`, `grep` TODO/UNIMPLEMENTED/PANIC) sur chaque PR.

**P2 — Qualité de signal & robustesse (+10 %)**
- Replayer Parquet, auto-calibration adaptative, TUI opérateur.
- Failover RPC, priorisation ComputeBudget dynamique.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
