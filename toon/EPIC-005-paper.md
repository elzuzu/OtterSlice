# EPIC-005 — Paper trading & calibration

> **But :** valider la stratégie en mode simulé, analyser les résultats et préparer la décision de go-live.

## Résultats attendus
- Mode paper opérationnel, génère des CSV complets et des rapports horaires.
- Analyse statistique (notebook/script) livrée avec graphiques.
- Paramètres mis à jour suite aux conclusions (config TOML).
- Rapport final prêt pour revue (template complété).

## Sprints
1. **SPRINT-005A — Mode paper trading temps réel**.
2. **SPRINT-005B — Analyse & calibration post-run**.

## Dépendances
- Requiert EPIC-004 terminé (kill-switch, monitoring) pour garantir la sécurité.
- Inputs : scanner/opportunités, modules d'exécution (en simulation) et logs.

## Points de contrôle
- **PC1 :** Exécution paper de 60 min sans crash (log à l'appui).
- **PC2 :** Rapport d'analyse complété avec recommandations chiffrées.
- **PC3 :** Paramètres ajustés et commités, justifiés dans le rapport.

## Risques & mitigations
- **Données corrompues** : mettre en place des checks `assert` dans la simulation + conserver des sauvegardes datées.
- **Conclusion hâtive** : imposer une revue croisée (deux reviewers minimum) avant d'appliquer les modifications de limites.

## Chemin critique
- SPRINT-005A doit produire des logs exploitables avant le lancement de 005B.
- Le passage en go-live (hors scope) ne peut être envisagé qu'après validation des deux sprints et signature du rapport.
