# EPIC-002 — Ingestion Marchés CEX/DEX

> **Vision :** disposer d'un pipeline de données fiable qui fournit en permanence des carnets cohérents, normalisés et enrichis pour tous les modules amont.

## Résultats attendus
- `crates/cex`, `crates/dex-clob`, `crates/dex-clmm`, `crates/ingest` opérationnels, documentés, testés (unitaires + intégration).
- Bus interne (`watch`/`broadcast`) alimentant le scanner avec des vues normalisées (`OrderBookView`, `ClobBook`, `QuoteResult`).
- Procédures de resynchronisation écrites (documents + tests) pour gérer les déconnexions WS et les mismatches d'update ID.
- Mesures de latence (<150 ms pour les updates CEX, <500 ms pour DEX) consignées dans `docs/logs/latency-benchmark.md`.

## Plan de réalisation
1. **SPRINT-002A — Connecteurs CEX REST/WS** : implémenter clients, throttling, signatures et tests.
2. **SPRINT-002B — Reconstructeur L2 CEX** : fusionner snapshots + diffs et publier des vues propres.
3. **SPRINT-002C — Drivers DEX CLOB** : lire Phoenix/OpenBook, préparer la pose d'ordres.
4. **SPRINT-002D — Quoteur CLMM** : calculer des quotes locales Orca/Raydium.

Chaque sprint produit un journal (`docs/logs/sprint-002X.md`) et des tests automatisés. Valide un sprint avant de passer au suivant (pair review).

## Dépendances externes
- RPC Solana principaux + fallback (voir README).
- Accès API CEX (clés en lecture/écriture trading) stockées dans Keychain.
- PlantUML pour les diagrammes (optionnel mais recommandé).

## Points de contrôle & critères de sortie
- **PC1 :** latence snapshot/diff mesurée (SPRINT-002A) < 200 ms (consignée).
- **PC2 :** `apply_diff` restitue un carnet cohérent sur 10 000 messages (SPRINT-002B) — test de replay.
- **PC3 :** `subscribe_orderbook` Phoenix/OpenBook renvoie au moins 5 updates/min sans erreur.
- **PC4 :** `quote_swap` Orca/Raydium correspond aux SDK officiels ±1%.
- **PC5 :** Document `docs/diagrams/ingestion-overview.png` mis à jour montrant les flux.

## Risques & mitigations
- **WS instables** : mettre en place backoff exponentiel + alertes (liées à SPRINT-004B).
- **Divergence d'ID** : tester `OutOfSync` et déclencher resnapshot automatique.
- **Latence CLMM** : privilégier caches mémoire, réduire la taille des structures.

## Chemin critique
- Sprints séquentiels (002A → 002B → 002C → 002D). Toute dérive bloque EPIC-003 (scanner/exécution).
- Validation d'un pair obligatoire avec check-list (journal + tests + latence) avant de passer au sprint suivant.
