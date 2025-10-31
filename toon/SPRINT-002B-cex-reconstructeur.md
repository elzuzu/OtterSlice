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

# SPRINT-002B — Reconstructeur L2 CEX `[P0]`

## Objectifs
- Rejouer snapshots/diffs Binance, OKX, Bybit pour publier `DepthBook` cohérent.
- Gérer cas d’échec : snapshot perdu, gap > N messages, resync forcé.
- Bench throughput ≥ 500 updates/s (dataset `fixtures/replay/binance_btcusdt_1k.json`).

## Pipeline
1. Recevoir snapshot (`lastUpdateId`/`seqId`/`seq`) et l’appliquer.
2. Appliquer diffs ordonnés, vérifier `prev_id + 1 == current_start`.
3. Émettre `DepthBook` vers bus interne (`tokio::sync::watch`).
4. Sur gap > 5 messages ou latence > 1000 ms → resnapshot (log `WARN resync`).

## Cas d’échec à couvrir
- Snapshot perdu : lever `RebuildError::SnapshotExpired` → resnapshot.
- Gaps > N (N=5) : `RebuildError::GapDetected { missing: u64 }` + redémarrer.
- WS reconnect : `ResyncReason::WebsocketReconnect` (tester).

## Tests
- `ingest/tests/binance_replay.rs` : rejoue fixtures snapshot + 1000 diffs.
- `ingest/tests/gap_detection.rs` : vérifie resnapshot.
- `ingest/tests/latency_budget.rs` : `Instant::now()` + simulate delay > 1 s → resnapshot.

## Bench
- `cargo bench -p ingest --bench depth_replay -- --dataset fixtures/replay/binance_btcusdt_1k.json`
  - Doit afficher `processed=1000 throughput>=500/s`.

## Exemples valides/invalides
- ✅ Journal `docs/logs/sprint-002B.md` contient graphe latence (gnuplot) + commande bench.
- ❌ Diff appliqué sans vérifier `prev_id`.

## Checklist de validation
- Tests & bench OK.
- Logs `resync` déclenchés sur gap.
- `just replay` restitue `best_bid <= best_ask` en continu.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
