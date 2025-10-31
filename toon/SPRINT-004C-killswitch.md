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

# SPRINT-004C — Kill-switch `[P0]`

## Objectifs
- Implémenter kill-switch déclenché par EPIC-004 :
  - Émettre signal `SIGUSR1` au process exécution.
  - Annuler ordres en cours (CEX/DEX).
  - Révoquer API keys temporaires (Binance `DELETE /fapi/v1/listenKey`, Bybit `POST /v5/user/cancel-listen-key`).
- Garantir idempotence (appel multiple sans effet secondaire supplémentaire).

## Pipeline
1. `KillSwitch::trigger(reason)` → log `ERROR` + event JSON (format EPIC-004).
2. Exécuter tasks en parallèle (`futures::try_join!`) :
   - `cex.cancel_all()` par venue.
   - `dex.abort_all_pending()`.
   - `system::signal(SIGUSR1)`.
3. Attendre confirmations `CancelAllResponse` (< 1 s).
4. Mettre flag `kill_switch_engaged = true` (AtomicBool).

## Tests
- `monitoring/tests/killswitch_idempotent.rs` : double trigger → seconde fois `AlreadyEngaged`.
- `monitoring/tests/killswitch_api.rs` : mocks HTTP pour deletes.
- `monitoring/tests/killswitch_signal.rs` : capture signal via pipe.

## Exemples valides/invalides
- ✅ `docs/logs/sprint-004C.md` contient chronologie (timestamps ms).
- ❌ Manque revocation listenKey.

## Checklist de validation
- Tests kill-switch OK.
- `just ci` passe.
- CI grep TODO/UNIMPLEMENTED reste actif.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
