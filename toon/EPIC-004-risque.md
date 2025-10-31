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

# EPIC-004 — Risque & Monitoring `[P0]`

> **Objectif :** surveiller en continu la qualité du book, le risque notionnel et déclencher le kill-switch en cas de dérive.
> **Priorité :** P0 (blocage production).

## Règles de contrôle durcies
| Indicateur | Seuil | Action |
|------------|-------|--------|
| Notional long/short cumulé | ≤ 500 000 USDT | Alerte `WARN` à 90%, `ERROR` à 100% + hedge forcé |
| Slippage réalisé p95 | ≤ 6 bps | Si > 6 bps pendant 3 min → suspendre nouveaux trades |
| Slot lag Solana | ≤ 1500 slots | Si > 1500 pendant 10 s → kill-switch |
| WS down-time (CEX/DEX) | ≤ 1.5 s | Si > 1.5 s → kill-switch + resnapshot |
| Latence ingestion p95 | ≤ 200 ms CEX / 500 ms DEX | Si > seuil → passer en mode "monitor-only" |

## Format des alertes
- Event JSON obligatoire :
  ```json
  {
    "ts": "2024-01-01T00:00:00Z",
    "kind": "risk_event",
    "code": "slot_lag",
    "severity": "error",
    "context": {
      "slot_lag": 1700,
      "threshold": 1500,
      "venue": "solana"
    }
  }
  ```
- Destination : `stdout` (logs `tracing`), fichier `logs/risk-events.ndjson`, hook HTTP `POST https://hooks.internal/risk` (payload brut).
- Les tests d’intégration doivent vérifier que chaque déclencheur écrit exactement 1 événement par changement d’état (pas de duplication).

## Monitoring & kill-switch
- `RiskEngine` doit exposer `fn evaluate(&mut self, snapshot: RiskSnapshot) -> Option<RiskAction>`.
- `RiskAction` : `Continue`, `ReduceExposure { target_notional: Decimal }`, `KillSwitch { reason: String }`.
- Kill-switch : `just kill` → envoie un signal `SIGUSR1` à l’exécuteur, ferme toutes les connexions CEX/DEX et invalide les clés temporaires.

## CI renforcée
- Ajouter job GitHub Actions `todo-scan` : `rg --fixed-strings --hidden --glob '!tests/**' --glob '!*.md' "todo" src` + `unimplemented` + `panic!` (échec si sortie).
- Ajouter job `audit` (`cargo audit`) et `deny` (`cargo deny check bans licenses sources advisories`).
- Scripts branchés dans `just ci` (EPIC-001).

## Journaux & métriques
- Exporter `metrics/risk.prom` avec :
  - `risk_notional_long`, `risk_notional_short`, `risk_slippage_p95`, `risk_slot_lag`, `risk_ws_down_seconds`.
  - `risk_last_kill_timestamp` (timestamp UNIX, -1 si aucun).
- Rédiger `docs/logs/risk-incidents.md` (date, cause, action).

## Interactions
- Consomme `ExecutionEvent` (EPIC-003) + `OrderBookView` (EPIC-002).
- Fournit `KillSwitch` au module monitoring (SPRINT-004C).

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
