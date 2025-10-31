# EPIC-003 — Décision & exécution arbitrage `[P0]`

> **But global :** convertir les données d'ingestion en signaux exploitables puis en ordres coordonnés CEX↔DEX tout en respectant les limites de risque.
> **Priorité** : **P0** — composante cœur métier : aucun lancement sans scanner net_fees et exécution fiable (IOC/FAK, hedge DEX).

## Résultats attendus
- Scanner des spreads net-frais configuré et éprouvé (tests de charge >500 updates/s).
- Modules d'exécution CEX/DEX capables de gérer fills partiels, retries, signature HMAC/transactions Solana.
- Calculateur de frais centralisé pour alimenter le scanner et les contrôles pre-trade.
- Journaux détaillés dans `docs/logs/sprint-003X.md` + captures de sorties CLI.

## Sprints de l'EPIC
1. **SPRINT-003A — Calculateur de frais & coûts transactionnels** : transformer les frais en basis points.
2. **SPRINT-003B — Scanner net-frais** : produire des opportunités filtrées.
3. **SPRINT-003C — Exécution CEX** : envoyer les ordres via les API REST, gérer latence, erreurs.
4. **SPRINT-003D — Exécution DEX (hedge)** : placer les ordres sur Phoenix/OpenBook/Orca/Raydium.

## Dépendances
- Nécessite EPIC-002 complet (ingestion).
- Alimente EPIC-004 (gestion du risque, monitoring) et EPIC-005 (paper).

## Points de contrôle
- **PC1 :** `cargo test` sur `common`, `engine`, `exec` passent (rapports attachés).
- **PC2 :** Mode CLI `scan` détecte au moins 1 opportunité sur données mockées.
- **PC3 :** Dry-run CEX/DEX génèrent les requêtes/transactions attendues (capture des logs).
- **PC4 :** Temps de réaction scanner→ordre < 250 ms (mesuré, consigné).

## Risques & mitigation
- **Erreurs API** : implémenter un mapping clair des codes d'erreur + retries bornés.
- **Fill partiel** : ajuster automatiquement la couverture DEX (SPRINT-003D).
- **Frais variables** : centraliser dans `config/fees.toml` et ajouter une tâche hebdomadaire de vérification.

## Chemin critique
- L'ordre des sprints est strict (003A → 003B → 003C → 003D). Chaque sprint doit fournir une démonstration CLI.
- Sans validation du scanner, pas d'exécution; sans exécution CEX, pas de hedge DEX.
