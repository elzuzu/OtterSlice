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

# SPRINT-003C — Exécution CEX `[P0]`

## Objectifs
- Implémenter `CexExecutor` pour Binance/OKX/Bybit utilisant clients du SPRINT-002A.
- Respecter mapping IOC/FAK/FOK (table EPIC-003) sans dépendances supplémentaires.
- Gérer latence `detection→hedge_ack <= 1s` (logs `latency_ms`).

## API
```rust
pub struct CexExecutor<'a> {
    rest: &'a RestClient,
}

impl<'a> CexExecutor<'a> {
    pub async fn place(&self, plan: &ExecutionPlan) -> Result<CexExecution, ExecutionError>;
}

pub struct CexExecution {
    pub order_id: String,
    pub filled_qty: Decimal,
    pub avg_price: Decimal,
    pub latency_ms: u64,
}
```

## Tâches
1. Mapper `ExecutionPlan.time_in_force` vers paramètres API (IOC/FAK/FOK).
2. Signer requêtes via fonctions SPRINT-002A.
3. Vérifier dérive horloge `< 500 ms` avant envoi (`/time`).
4. Log `INFO` `cex_order_submitted` + `cex_order_filled`.
5. Si `status = FILLED` → succès; `status = PARTIALLY_FILLED` + TIF FOK → `ExecutionError::Rejected`.

## Tests
- `execution/tests/cex_executor.rs` : mocks HTTP (WireMock) pour codes `429`, `51009`, `30005`.
- `execution/tests/latency.rs` : mesurer latence < 1 s.
- `execution/tests/time_sync.rs` : offset > 500 ms → `ExecutionError::ClockSkew`.

## Bench
- `cargo bench -p execution --bench cex_executor -- --plans 200` throughput ≥ 200 plans/s.

## Exemples valides/invalides
- ✅ Journal `docs/logs/sprint-003C.md` contient latence moyenne.
- ❌ `panic!()` pour erreurs API.

## Checklist de validation
- Tests & bench OK.
- `just ci` passe.
- Logs alignés (timezone UTC).

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
