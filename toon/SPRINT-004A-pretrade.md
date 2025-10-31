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

# SPRINT-004A — Garde-fous pré-trade `[P0]`

## Objectifs
- Implémenter `PreTradeChecker` appliquant seuils :
  - Notional max 500k USDT (par sens).
  - Slot lag ≤ 1500.
  - Slippage p95 ≤ 6 bps.
  - WS downtime ≤ 1.5 s.
- Retourner `PreTradeDecision::{Allow, Reject { reason }}`.

## Entrées
```rust
pub struct PreTradeInput {
    pub planned_notional: Decimal,
    pub current_notional_long: Decimal,
    pub current_notional_short: Decimal,
    pub slot_lag: u64,
    pub slippage_p95_bps: u16,
    pub ws_down_seconds: f32,
}
```

## Tâches
1. Implémenter `fn evaluate(&self, input: &PreTradeInput) -> PreTradeDecision`.
2. Log `INFO` sur acceptation, `WARN` sur rejet.
3. Publier métriques `pretrade_reject_total` (labels `reason`).
4. Tests pour chaque seuil (>= / >).

## Guardrails & fenêtres
| KPI | Seuil | Unité | Notes |
| --- | --- | --- | --- |
| `slippage_p95_bps` | ≤ 6 | bps | calcul rolling `ROLLING_HOURS` |
| `slot_lag` | ≤ 1500 | slots | kill-switch si dépassé |
| `ws_down_seconds` | ≤ 1.5 | s | déclenche kill-switch |
| `max_drawdown_30j` | ≤ 5 | % | verrouillé par risk (SPRINT-004B) |

- Exposer `ROLLING_HOURS` via TOML + override **ENV** (injecté par `scripts/run_bot_mainnet.sh`) pour le calcul des p95/DD.
- Documenter les valeurs lues dans les logs (`tracing::info!`) au démarrage.

## Exemples valides/invalides
- ✅ Rejet notional > 500k consigné dans `docs/logs/sprint-004A.md`.
- ❌ Tolérance slot lag > 1500.

## Checklist de validation
- `cargo test -p risk` couvre tous les cas.
- `just ci` passe.
- Métriques exportées vers `metrics/risk.prom`.
- Guardrails (`ROLLING_HOURS`, seuils) affichés au boot + respect de la table ci-dessus.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
