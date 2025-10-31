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

# SPRINT-004B — Monitoring temps réel `[P0]`

## Objectifs
- Agréger métriques ingestion, exécution, risque et publier alertes JSON formatées (voir EPIC-004).
- Sorties : `stdout`, fichier `logs/risk-events.ndjson`, webhook.
- CI renforce : job `todo-scan`, `cargo audit`, `cargo deny` (déjà dans EPIC-001/004) → vérifier présents.

## Schéma d’événement
Réutiliser bloc EPIC-004. Chaque événement doit inclure `kind`, `code`, `severity`, `context`.

## Tâches
1. Implémenter `Monitor::emit(event: RiskEvent)` écrivant simultanément dans les trois destinations.
2. Ajouter back-pressure : si webhook échoue, log `ERROR` et retry (1s, 5 tentatives).
3. Exposer `fn record_metric(name: &str, value: f64)` écrivant dans `metrics/risk.prom`.
4. Tests :
   - `monitoring/tests/event_output.rs` : vérifie 3 destinations.
   - `monitoring/tests/retry.rs` : webhook échoue 4 fois → 4 retries + event `severity=error`.
5. Ajouter job CI `grep` (shell) qui échoue si `todo|unimplemented|panic!` dans `src/` (hors tests) + documenter dans `just ci`.

## Guardrails monitorés
| KPI | Seuil | Source | Notes |
| --- | --- | --- | --- |
| `slippage_p95_bps` | ≤ 6 | risk module | rolling `ROLLING_HOURS` |
| `latency_ms` | ≤ 1000 | exécution | reporter médiane/p95 |
| `ws_down_seconds` | ≤ 1.5 | ingestion | alert fatal si dépassé |
| `max_drawdown_30j` | ≤ 5% | risk analytics | recalcul horaire |

- Consommer `ROLLING_HOURS` via ENV/TOML (mêmes conventions que SPRINT-004A) pour aligner fenêtres.
- Ajouter un tableau synthétique des guardrails dans `docs/logs/sprint-004B.md`.

## Exemples valides/invalides
- ✅ `docs/logs/sprint-004B.md` contient extrait NDJSON.
- ❌ Écriture partielle (fichier sans newline).

## Checklist de validation
- Tests monitoring OK.
- `just ci` passe (vérifier job grep TODO/UNIMPLEMENTED).
- `metrics/risk.prom` contient metrics listées EPIC-004.
- Table guardrails + export `ROLLING_HOURS` vérifiés (logs + docs).

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
