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

# EPIC-005 — Paper Trading & Calibration `[P1]`

> **Vision :** disposer d’une boucle de calibration automatique basée sur des datasets contrôlés et un rapport diffable.
> **Priorité :** P1, dépend des EPICs 002–004.

## Dataset & scripts requis
- Dataset principal `data/paper/btc_usdt_72h.parquet` (72 h sliding window, champs : `ts`, `venue`, `bid_px`, `ask_px`, `mid_px`, `depth_bps`, `fills`...).
- Script `scripts/paper/run.rs` lancé via `just paper -- --from 2024-01-01T00:00:00Z --to 2024-01-03T23:59:59Z --dataset data/paper/btc_usdt_72h.parquet`.
- Replayer e2e `just replay -- --dataset fixtures/replay/binance_btcusdt_1k.json` (connecté à EPIC-002).

## Rapport structuré
- Fichier Markdown `docs/reports/paper.md` avec sections :
  1. **Résumé exécutif** (5 bullet points max).
  2. **Métriques clés** : `win_rate`, `avg_edge_bps`, `latency_ms_p50/p95`, `slippage_bps_p95`, `notional_turnover`.
  3. **Incidents & mitigations** (table).
  4. **Recommandations** : liste numérotée liée à des commits (`git rev-parse HEAD`).
- Export CSV `docs/reports/paper_metrics.csv` (colonnes `metric`, `value`, `window_start`, `window_end`).

## Auto-calibration (Phase 2)
- Algorithme :
  - Si `false_positive_rate > 0.05` → augmenter `min_edge_bps += 10`.
  - Si `fill_rate < 0.30` → diminuer `target_size` de 15%.
  - Sinon, conserver paramètres.
- Implémenter dans `crates/calibration/src/lib.rs` fonction `fn auto_tune(params: &mut StrategyParams, stats: &BacktestStats)`.
- Test unitaire `calibration/tests/auto_tune.rs` couvrant les trois branches.

## Commandes obligatoires
- `toon run --mode paper --from <ISO8601> --to <ISO8601> --dataset data/paper/btc_usdt_72h.parquet` (documenter exemple complet).
- `toon run --mode calibration --dataset data/paper/btc_usdt_72h.parquet --iterations 5`.

## Journalisation
- `docs/logs/paper-trading.md` : insérer extraits `just paper` et `just replay`.
- Garder `data/paper/README.md` à jour (taille dataset, checksum SHA256).

## DoD additionnel
- Runs de 60–72 h en continu (paper) avec histogramme latence (bin 10 ms) stocké dans `docs/reports/paper_latency.png`.
- Diff `git diff` du rapport attaché à la MR (section "Recommandations").

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
