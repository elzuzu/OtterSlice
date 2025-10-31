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

# SPRINT-002D — Quoteur DEX CLMM `[P0]`

## Objectifs
- Implémenter quote local Orca Whirlpools (`whirLb7sGJGZveHFkNjEcVuJ39MF4RduCMsZ7M7P2`) et Raydium CLMM (`CLMMvx4u1S6C9G18JNpDutLCRa14Q6gttYwjdJawVcc`).
- Calculer swap `in_amount/out_amount` avec tolérance < 5 bps par rapport aux SDK officiels.
- Bench 500 updates/s (quote recalculations) sans backlog.

## Formules
- Orca :
  ```text
  sqrt_price = sqrt_price_x64 as f64 / 2^64
  price = (sqrt_price^2) * (10^{decimals_base - decimals_quote})
  out_amount = amount_in * (1 - fee_bps/10_000) * liquidity_adjustment
  ```
  Liquidity adjustment : intégrer ticks actifs `lower_tick`, `upper_tick` via `sqrt_price_lower/upper`.
- Raydium :
  ```text
  dy = liquidity * (sqrt_price_upper - sqrt_price_lower) / (sqrt_price_upper * sqrt_price_lower)
  price = (sqrt_price_current^2)
  fee = max(platform_fee_bps, protocol_fee_bps)
  out_amount = dy * (1 - fee/10_000)
  ```

## Schémas
- Orca : comptes `Whirlpool`, `TickArray`, `PositionList`. Charger via `whirlpool_program::state` (fork interne si besoin).
- Raydium : comptes `AmmConfig`, `AmmV3PoolState`, `PersonalPosition`.

## Tâches
1. Parser comptes depuis fixtures (`fixtures/orca/whirlpool.bin`, `fixtures/raydium/pool.bin`).
2. Implémenter `fn quote_swap(params: QuoteParams) -> QuoteResult` (structure finalisée, sans TODO).
3. Comparer `QuoteResult` aux résultats fournis par `fixtures/orca/expected.json`, `fixtures/raydium/expected.json`.
4. Bench `cargo bench -p dex-clmm --bench quote_pipeline`.

## Tests
- `dex_clmm/tests/orca_quote.rs` : 3 cas (in, out, slippage violation).
- `dex_clmm/tests/raydium_quote.rs` : 3 cas.
- `dex_clmm/tests/resync.rs` : reload pool si tick array manquant.

## Exemples valides/invalides
- ✅ Diff < 5 bps documenté dans `docs/logs/sprint-002D.md`.
- ❌ Placeholder "TODO compute fees".

## Checklist de validation
- `cargo clippy -p dex-clmm --all-targets -D warnings` passe.
- Bench >= 500 updates/s.
- Rapport `docs/logs/sprint-002D.md` contient table comparant aux SDK.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
