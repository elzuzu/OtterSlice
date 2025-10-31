# EPIC-004 — Gestion du risque & monitoring

> **But :** encadrer l'exécution via des contrôles pre-trade, une visibilité temps réel et des mécanismes d'arrêt d'urgence.

## Résultats attendus
- Contrôles pre-trade appliqués à chaque opportunité (caps, balances, spread minimum).
- Monitoring en place : métriques Prometheus + dashboard Grafana + alertes Slack.
- Kill-switch fonctionnel avec runbook détaillé et intégration dans tous les modules.
- Journaux de sprint `docs/logs/sprint-004X.md` complétés avec preuves (captures, logs, tests).

## Sprints
1. **SPRINT-004A — Contrôles pre-trade & allocation**.
2. **SPRINT-004B — Monitoring temps réel & alerting**.
3. **SPRINT-004C — Kill-switch & incidents**.

## Dépendances
- Repose sur EPIC-003 (exécution) pour les rapports d'ordres.
- Conditionne EPIC-005 (paper/calibration) : aucun passage en paper sans kill-switch opérationnel.

## Points de contrôle
- **PC1 :** `cargo test -p risk` (pre-trade + kill-switch) verts.
- **PC2 :** Endpoint `/metrics` accessible, dashboard JSON + capture commités.
- **PC3 :** Exercice kill-switch documenté (simulateur) avec timestamp + rôles.

## Risques & mitigations
- **Faux positifs monitoring** : calibrer les seuils, mettre en place un mode silence (maintenance).
- **Kill-switch non accessible** : script de fallback manuel (écriture directe du fichier `state/kill_switch.json`).
- **Pré-trade trop strict** : ajouter un mode simulation pour valider les limites sur 7 jours de paper.

## Chemin critique
- Séquence imposée : 004A → 004B → 004C. Le kill-switch dépend des métriques (alertes) et des contrôles pre-trade.
- Chaque sprint requiert un runbook ou un document de support mis à jour.
