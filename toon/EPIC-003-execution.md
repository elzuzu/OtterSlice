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

# EPIC-003 — Exécution multi-venues `[P0]`

> **Vision :** enchaîner décision d’arbitrage → exécution DEX → hedge CEX en < 1 s, avec gestion robuste des erreurs.
> **Priorité :** P0, dépend directement des EPICs 001 et 002.

## Structures & signatures obligatoires
- `crates/execution/src/lib.rs` doit exposer :
  ```rust
  pub struct ExecutionConfig {
      pub dex_timeout_ms: u64,
      pub cex_timeout_ms: u64,
      pub max_slippage_bps: u16,
      pub compute_limit: u32,
      pub compute_price_micro_lamports: u64,
  }

  #[derive(Debug, thiserror::Error)]
  pub enum ExecutionError {
      #[error("rate_limit")] RateLimit { venue: Venue, retry_after: Duration },
      #[error("clock_skew")] ClockSkew { offset_ms: i64 },
      #[error("rejected")] Rejected { venue: Venue, code: String, context: String },
      #[error("timeout")] Timeout { venue: Venue, elapsed_ms: u64 },
  }

  pub enum Venue {
      Phoenix,
      OpenBook,
      Orca,
      Raydium,
      BinanceFutures,
      OkxPerp,
      BybitPerp,
  }

  pub struct ExecutionResult {
      pub dex_tx_signature: Signature,
      pub hedge_order_id: String,
      pub filled_base: Decimal,
      pub average_price: Decimal,
      pub latency_ms: u64,
  }

  pub trait ExecutionEngine {
      fn place_and_hedge(&self, plan: ExecutionPlan) -> Result<ExecutionResult, ExecutionError>;
  }
  ```
- `ExecutionPlan` contient `base_symbol`, `quote_symbol`, `side`, `target_size`, `dex_route`, `hedge_venue`, `time_in_force`.
- Les tests doivent instancier chaque variante d’erreur (`RateLimit`, `ClockSkew`, `Rejected`, `Timeout`).

## Scénarios d’erreur obligatoires
1. **Rate-limit** : pour chaque CEX, renvoyer `Retry-After` (Binance header `Retry-After`, OKX `x-ratelimit-remaining`, Bybit code `10006`).
2. **Horloge décalée** : comparer `/time` CEX avec horloge locale; si |offset| > 500 ms, émettre `ExecutionError::ClockSkew`.
3. **Rejet exchange** : propager code brut (`"LOT_SIZE"`, `"51009"`, `"30005"`) dans `context`.
4. **Timeout** : `tokio::time::timeout` 500 ms DEX, 700 ms CEX; log `WARN` et renvoyer `ExecutionError::Timeout`.

## Mapping IOC/FAK/FOK par venue
| Venue | IOC | FAK | FOK |
|-------|-----|-----|-----|
| Binance | `timeInForce=IOC` | `timeInForce=FOK` (simulate fill or kill) | `timeInForce=FOK` + `reduceOnly=false` |
| OKX | `ordType="ioc"` | `ordType="fok"` (aucune FAK, réutiliser FOK) | `ordType="fok"` |
| Bybit | `timeInForce="IOC"` | `timeInForce="PostOnly"` (FAK simulée par annulation) | `timeInForce="FOK"` |
| Phoenix | Instruction `place_order` avec `SelfTradeBehavior::DecrementTake` et `last_valid_slot` | Annulation manuelle après `N=1` slot | `last_valid_slot = current_slot` |
| OpenBook | `place_order` `IOC` flag | `ImmediateOrCancel` + `post_only` false | `ImmediateOrCancel` + `limit=best_bid/ask` |
| Orca | Swap `tick_array_range`, slippage `<= max_slippage_bps` | Non supporté → abort | Non supporté |
| Raydium | `swap_base_in` avec `limit_price` dérivé | Non supporté | Non supporté |

## Instructions ComputeBudget v3
- Ajouter en tête de chaque transaction DEX :
  1. `ComputeBudgetInstruction::set_compute_unit_limit(config.compute_limit)`.
  2. `ComputeBudgetInstruction::set_compute_unit_price(config.compute_price_micro_lamports)`.
- Valeurs par défaut : `compute_limit = 150_000`, `compute_price_micro_lamports = 0` (configurable via `config/compute_budget.toml`).
- Tests d’intégration `tests/compute_budget.rs` doivent valider l’ordre des instructions.

## Hedge CEX & latence
- Endpoint hedge par venue :
  - Binance : `POST /fapi/v1/order` (body indiqué dans EPIC-002) avec `reduceOnly=true`.
  - OKX : `POST /api/v5/trade/order` `side=buy|sell`, `reduceOnly=true`.
  - Bybit : `POST /v5/order/create` `reduceOnly=true`.
- Latence cible : `detection_ts → dex_tx_signature → hedge_order_ack` < **1000 ms** (logs `latency_ms`).
- Logger `INFO` `hedge_submitted`, `hedge_filled` avec latence.

## Tests & benchs
- Tests unitaires `execution/tests/time_in_force_mapping.rs` (couverture exhaustive du tableau).
- Tests d’intégration `tests/execution_roundtrip.rs` simulant plan complet (mocks) et vérifiant latence < 1 s.
- Bench `cargo bench -p execution --bench plan_pipeline` mesurant throughput 200 plans/s.

## Interactions
- Consomme `OrderBookView` EPIC-002.
- Publie événements `ExecutionEvent` pour EPIC-004.
- Utilise `just ci` pour valider pipeline complet.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
