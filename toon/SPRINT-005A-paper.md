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

# SPRINT-005A — Paper trading `[P1]`

## Objectifs
- Lancer `toon run --mode paper --from 2024-01-01T00:00:00Z --to 2024-01-03T23:59:59Z --dataset data/paper/btc_usdt_72h.parquet` et collecter métriques.
- Produire rapport `docs/reports/paper.md` (structure EPIC-005) + CSV `docs/reports/paper_metrics.csv`.
- Générer histogramme latence `docs/reports/paper_latency.png` (bin 10 ms).

## Tâches
1. Implémenter `PaperEngine` lisant dataset Parquet via `parquet` crate (déjà dépendance? sinon utiliser arrow2?). Ne pas ajouter dépendance non listée → utiliser `parquet` existante si déjà.
2. Enregistrer métriques `win_rate`, `avg_edge_bps`, `latency_ms_p50/p95`, `slippage_bps_p95`, `notional_turnover`.
3. Exporter CSV (colonnes `metric,value,window_start,window_end`).
4. Actualiser rapport Markdown : sections `Résumé exécutif`, `Métriques`, `Incidents`, `Recommandations`.
5. Commande `just paper` doit encapsuler exécution + génération rapport.
6. Ajouter exécution `toon --mode tune --paper --budget 72h` (exploration safe) avec caps depuis `scripts/run_bot_mainnet.sh`.

## Mode tuning (paper)
- Définir profil `--mode tune` pour replay 60–72h (budget `TUNE_BUDGET_ITERS` respecté).
- Journaliser `Score`, contraintes violées (=0) et config choisie par épisode dans `./runs/tuning/*.json`.
- Générer `config/tuned/<timestamp>.toml` + symlink `config/tuned/current.toml`.

## Exemples valides/invalides
- ✅ Rapport contient référence commit (`git rev-parse HEAD`).
- ❌ Rapport sans section incidents.

## Checklist de validation
- `just paper` produit fichiers sans diff local non commit.
- `docs/logs/paper-trading.md` contient sortie CLI.
- `cargo test -p paper` (tests unitaires sur metrics) passe.
- Rapport tuning ajouté (`docs/reports/paper.md`) avec Score vs. épisodes + config persistée.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
