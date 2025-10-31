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

# SPRINT-005B — Auto-calibration `[P1]`

## Objectifs
- Implémenter `auto_tune` (EPIC-005) avec règles :
  - `false_positive_rate > 0.05` → `min_edge_bps += 10`.
  - `fill_rate < 0.30` → `target_size *= 0.85`.
  - Sinon inchangé.
- Runner `toon run --mode calibration --dataset data/paper/btc_usdt_72h.parquet --iterations 5`.

## Structures
```rust
pub struct StrategyParams {
    pub min_edge_bps: i32,
    pub target_size: Decimal,
}

pub struct BacktestStats {
    pub false_positive_rate: f64,
    pub fill_rate: f64,
}
```

## Tâches
1. Implémenter `auto_tune` (aucun TODO) + logs `INFO` pour chaque ajustement.
2. Écrire tests `calibration/tests/auto_tune.rs` couvrant 3 branches + idempotence.
3. `just calibration` (alias `just run-calibration`) exécute boucle.
4. Append résultats dans `docs/reports/calibration.log` (format `timestamp|min_edge_bps|target_size|fpr|fill_rate`).
5. Orchestrer `toon --mode tune --paper --budget 72h` en fin de calibration pour comparer config auto-calibration vs. tuner.

## Intégration tuning
- Inclure dans `docs/reports/calibration.log` la meilleure config `tuner` (score, paramètres, contraintes respectées).
- Copier la config optimale dans `config/tuned/<timestamp>.toml` + mettre à jour `config/tuned/current.toml`.
- Ajouter graphique Score/itération (PNG) dans `docs/reports/calibration_tuning.png`.

## Exemples valides/invalides
- ✅ Ajustement +10 bps quand FPR 0.06.
- ❌ Ignorer fill_rate < 0.30.

## Checklist de validation
- `cargo test -p calibration` passe.
- `just run-calibration` produit log.
- Rapport `docs/reports/paper.md` met à jour section recommandations.
- `config/tuned/current.toml` + `calibration_tuning.png` présents; contraintes violées = 0.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
