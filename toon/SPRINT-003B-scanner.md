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

# SPRINT-003B — Scanner opportunités `[P0]`

## Objectifs
- Consommer `OrderBookView` (CEX/DEX) et `ClobBook` pour détecter arbitrages.
- Filtrer signaux : au moins `n_signaux = 3` occurrences sur fenêtre 60 s.
- Supporter débit ≥ 500 updates/s sans backlog (`tokio::mpsc::channel(1024)`).

## Pipeline
1. Normaliser quotes en basis points (`edge_bps`).
2. Appliquer filtres :
   - `edge_bps >= 20` (warning) puis >= 70 (target).
   - Vérifier `effective_fee_bps` (SPRINT-003A).
   - Vérifier `slippage_estimate_bps <= 6`.
3. Maintenir mémoire circulaire 60 s pour compter occurrences.
4. Publier `ScannerSignal` :
   ```rust
   pub struct ScannerSignal {
       pub opportunity_id: Uuid,
       pub edge_bps: i32,
       pub venue_in: Venue,
       pub venue_out: Venue,
       pub notional: Decimal,
       pub occurrence_count: u32,
       pub generated_at: DateTime<Utc>,
   }
   ```

## Paramètres finetuables & traçabilité
- **Expose** `SPREAD_MIN_BPS`, `SPREAD_MIN_BPS_NAIVE`, `MARGE_SECURITE_BPS`, `OCCURRENCE_FILTER_N_PER_XMIN`, `GAS_TO_BPS_FACTOR` via TOML + override **ENV** (injectés depuis `scripts/run_bot_mainnet.sh`).
- Lire les valeurs à l'init (`env` -> fallback TOML) et logguer les valeurs retenues en début de run (`tracing::info!`).
- Respecter les bornes définies dans `config/tuning.toml`; refuser le démarrage si hors bornes.

## Tests
- `scanner/tests/filtering.rs` : vecteurs (edge 10 → rejet, edge 80 + 3 occurrences → accept).
- `scanner/tests/rate.rs` : simuler 600 updates/s, vérifier backlog 0 (`channel.len() == 0`).
- `scanner/tests/window.rs` : 2 occurrences < 60 s → rejet.

## Bench
- `cargo bench -p scanner --bench pipeline -- --updates 1000` doit afficher `throughput >= 500/s`.

## Exemples valides/invalides
- ✅ Log `opportunity_id` + `edge_bps` dans `docs/logs/sprint-003B.md`.
- ❌ Utilisation de `panic!()` pour un edge insuffisant.

## Checklist de validation
- Tests & bench OK.
- `just ci` passe.
- `docs/logs/sprint-003B.md` contient captures throughput.
- Log en début de run des paramètres finetuables exposés + preuve 500 updates/s.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
