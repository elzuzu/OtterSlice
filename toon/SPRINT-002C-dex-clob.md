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

# SPRINT-002C — Drivers DEX CLOB `[P0]`

## Objectifs
- Implémenter lecteurs Phoenix/OpenBook (programmes `PHNXsHsS9GK1E58uQJX9m5L7F7mWJ9jH3UX77PG5Fv`, `openBk2L3sNjJKuX3sQYboJjQmpaq4Xrk3R9jFJmMq`).
- Publier `ClobBook` normalisé :
  ```rust
  pub struct ClobBook {
      pub market: Pubkey,
      pub bids: Vec<Level>,
      pub asks: Vec<Level>,
      pub slot: u64,
  }
  ```
- Préparer instructions d’ordre IOC (structures prêtes pour EPIC-003).

## Schémas d’accounts
- Phoenix :
  - `Market` (header at offset 0, lot sizes),
  - `EventQueue` (ring buffer, `SequenceNumber`),
  - `Bid/Ask Queue`.
- OpenBook :
  - `Market` (header + vaults),
  - `EventQueue` (linked events),
  - `Bids`, `Asks` (slab).

## Tâches
1. Mapper comptes -> structures Rust (`phoenix_sdk::state`, `openbook_dex::state` via patch forks).
2. Implémenter conversion lot → prix/quantité :
   - Phoenix : `price = price_lots * tick_size / quote_lot_size`, `size = num_base_lots * base_lot_size`.
   - OpenBook : `price = price_lots * tick_size`, `size = base_quantity_lots * base_lot_size`.
3. Tester `subscribe_orderbook` sur RPC (`get_program_accounts` + WS).
4. Simuler pose d’ordre `ImmediateOrCancel` (sans exécution) : build instruction + check compute budget placeholder (sera rempli sprint 003).

## Tests
- `dex_clob/tests/phoenix_parse.rs` : fixture `fixtures/phoenix/market_account.bin`.
- `dex_clob/tests/openbook_slab.rs` : charge `fixtures/openbook/bids.bin`.
- `dex_clob/tests/slot_progress.rs` : assure `slot` monotone.
- Comparer contre SDK officiel (écart prix < 1e-6).

## DoD spécifique
- `just replay -- --dataset fixtures/replay/openbook_depth.json` produit 5 updates/min.
- `cargo bench -p dex-clob --bench slab_decode` >= 100k nodes/s.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
