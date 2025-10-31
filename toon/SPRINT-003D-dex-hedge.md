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

# SPRINT-003D — Hedge DEX `[P0]`

## Objectifs
- Construire transactions Phoenix/OpenBook/Orca/Raydium avec instructions ComputeBudget v3 en tête.
- Respecter latence `détection→exec→hedge <= 1 s` (logs partagés avec SPRINT-003C).
- Mapper codes d’erreur en enums typed (`ExecutionError`).

## Pipeline transaction Solana
1. Préparer `ComputeBudgetInstruction::set_compute_unit_limit(150_000)` puis `set_compute_unit_price(config.compute_price_micro_lamports)`.
2. Ajouter instructions DEX (Phoenix `place_order`, OpenBook `new_order_v3`, Orca `swap`, Raydium `swap_base_in`).
3. Signer via `Keypair` local; logs `tx_signature`.
4. Soumettre via RPC `send_transaction` (timeout 500 ms, 3 retries exponential backoff).

## Tâches
- Implémenter `DexHedger::submit(plan: &ExecutionPlan) -> Result<DexExecution, ExecutionError>`.
- Supporter overrides config (`config/dex/hedge.toml`) pour compute limit/price.
- Journaliser latence `dex_tx_latency_ms`.
- Coder enums d’erreur : `InsufficientLiquidity`, `SlippageExceeded`, `AccountMismatch`.

## Paramètres finetuables & guardrails
- **Expose** `SIZING_USD_PER_TRADE`, `MAX_SLIPPAGE_P95_BPS`, `TIME_IN_FORCE` via TOML + override **ENV** (`scripts/run_bot_mainnet.sh`).
- Rendre `CU_LIMIT` et `CU_PRICE_MICROLAMPORT` pilotables par ENV (fallback TOML) et logguer les valeurs appliquées pour chaque transaction.
- Appliquer les bornes de `config/tuning.toml`; si hors plage → `ExecutionError::ConfigOutOfRange` + abort.

## Tests
- `execution/tests/dex_compute_budget.rs` : ordre des instructions.
- `execution/tests/dex_error_mapping.rs` : map codes (Phoenix `OrdersAreLocked`, OpenBook `0x1770`, Orca `0x12c`, Raydium `0x1f4`).
- `execution/tests/dex_latency.rs` : latence < 1 s.

## Bench
- `cargo bench -p execution --bench dex_pipeline -- --plans 200` throughput ≥ 150 plans/s.

## Exemples valides/invalides
- ✅ `docs/logs/sprint-003D.md` contient exemple transaction (signature + latence).
- ❌ Oubli ComputeBudget instructions.

## Checklist de validation
- Tests & bench OK.
- `just ci` passe.
- Config `config/dex/hedge.toml` versionnée avec valeurs par défaut.
- Logs `CU_LIMIT`/`CU_PRICE_MICROLAMPORT` + sizing/slippage/TIF; garde-fous bornes actifs.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
