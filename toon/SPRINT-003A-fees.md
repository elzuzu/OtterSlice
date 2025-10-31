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

# SPRINT-003A — Calcul des frais `[P0]`

## Objectifs
- Convertir tous les frais (gas Solana, pool fee, maker/taker CEX) en basis points.
- Maintenir deux seuils : `target_fee_bps = 70`, `warning_fee_bps = 20`.
- Exposer API `fn effective_fee_bps(context: FeeContext) -> u32`.

## Entrées & structures
```rust
pub struct FeeContext {
    pub venue: Venue,
    pub lamports_per_signature: u64,
    pub compute_units_consumed: u32,
    pub compute_unit_price: u64,
    pub pool_fee_bps: u16,
    pub maker_fee_bps: Option<i16>,
    pub taker_fee_bps: Option<i16>,
}
```
- `Venue` réutilise enum EPIC-003.
- Gas→bps : `fee_bps = ((lamports + cu_cost) * 10_000) / notional_lamports`.

## Tâches
1. Implémenter conversion lamports → USD via `config/fee/oracle.toml` (fournir structure, fallback 180 USD/SOL).
2. Ajouter tests `fees/tests/solana_gas.rs` (cas compute price 0 et 10 microLamports).
3. Couvrir CEX : `maker_fee_bps`, `taker_fee_bps` signés (Bybit -2 bps).
4. Documenter dans `docs/fees.md` (table venue→fee).

## Exemples valides/invalides
- ✅ `effective_fee_bps` retourne 65 pour compute price 0.
- ❌ Frais négatifs ignorés.

## Checklist de validation
- `cargo test -p fees` passe.
- `just ci` inclut module.
- `docs/logs/sprint-003A.md` contient exemples calculés.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
