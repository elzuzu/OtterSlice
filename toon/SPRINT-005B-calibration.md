# SPRINT-005B — Analyse & calibration post paper trading `[P1]`

> **But :** exploiter les logs du mode paper pour calibrer les paramètres (frais, slippage, limites) et produire un rapport décisionnel.
> **Priorité** : **P1** — démarre seulement après validation du run paper P0. Prépare la phase P2 (auto-calibration hebdo, occurrence filter) via recommandations chiffrées.

## Pré-requis
- SPRINT-005A (paper) a produit au moins un fichier CSV `logs/paper.csv`.
- Outils d'analyse disponibles (`python3`, `pandas` ou `polars`).

## Livrables
1. Notebook ou script `analysis/paper_calibration.ipynb` (ou `.py`) calculant :
   - Distribution du spread observé vs net spread.
   - PnL cumulé, drawdown, Sharpe simplifié.
   - Taux de réussite (opportunités > 0 net bps).
2. Rapport `docs/reports/paper_calibration.md` complété (template déjà présent).
3. Mise à jour des paramètres dans `config/default.toml` (sections `risk`, `paper`, `execution`).
4. Réunion de revue (note dans `docs/logs/sprint-005B.md`).

## Étapes guidées
1. **Préparer l'environnement Python**
   - Crée un virtualenv : `python3 -m venv .venv && source .venv/bin/activate`.
   - Installe `pip install pandas polars matplotlib rich jupyter`.
2. **Charger les données**
   - Notebook :
     ```python
     import pandas as pd
     df = pd.read_csv("logs/paper.csv", parse_dates=["timestamp"])
     ```
   - Vérifie les colonnes attendues (ajoute un assert).
3. **Calculer les métriques principales**
   - `df['cum_pnl'] = df['pnl_usd'].cumsum()`.
   - `max_drawdown = (df['cum_pnl'].cummax() - df['cum_pnl']).max()`.
   - `win_rate = (df['pnl_usd'] > 0).mean()`.
   - `avg_spread = df['signal_spread_bps'].mean()`.
4. **Visualisations**
   - Graph `cum_pnl` vs temps.
   - Histogramme du `fill_ratio`.
   - Scatter `signal_spread_bps` vs `pnl_usd`.
   - Sauvegarde sous `docs/reports/img/pnl_curve.png`.
5. **Calibration**
   - Si `win_rate < 0.6`, envisager d'augmenter `min_spread_bps`.
   - Si `max_drawdown > limites`, réduire `per_market_cap_usd`.
   - Documente chaque décision dans le rapport (section "Ajustements recommandés").
6. **Mettre à jour la configuration**
   - Modifie `config/default.toml` : ajuster `risk.max_notional_usd`, `execution.max_inflight_txs`, etc. (justifie les changements).
7. **Rapport**
   - Complète `docs/reports/paper_calibration.md` : Résumé, Observations clés, Ajustements, Prochaines étapes (incluant axes P2 : occurrence filter, sizing adaptatif, replayer e2e, TUI opérateur).
   - Ajoute un tableau `Avant/Après` pour les paramètres modifiés.
   - Insère une section "Pont vers P2" listant les actions hebdomadaires d'auto-calibration proposées.
8. **Journal**
   - `docs/logs/sprint-005B.md` : note la date, la durée d'analyse, les personnes présentes en revue, et un lien vers le notebook.

## Critères d'acceptation
- ✅ Notebook/script exécutable (pas d'erreur) et versionné.
- ✅ Rapport complété avec captures et décisions.
- ✅ Paramètres mis à jour dans la configuration (diff visible).
- ✅ Journal complété.

## Dépendances
- Fournit les inputs à la décision de go-live.
- Boucle de rétroaction avec les modules de risque/exécution.

## Points d'attention
- Conserve le CSV original (`logs/paper.csv`) dans un dossier daté (`logs/paper/2024-xx-xx.csv`).
- Documente le contexte (marchés, heures) pour interpréter correctement les résultats.
- Vérifie l'intégrité des données (pas de doublons, pas de timestamp manquant).

## Pont vers P2
- Fournis un tableau des faux positifs/faux négatifs pour éclairer l'implémentation de l'occurrence filter.
- Estime le gain potentiel d'un sizing adaptatif (vs. profondeur disponible) et propose une formule initiale.
- Liste les datasets à capturer (fichiers Parquet) pour un replayer e2e et attribue un responsable.
- Suggère les métriques à exposer dans la future TUI opérateur (latence, spreads, fills).
