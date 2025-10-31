# Runbook — Kill-switch OtterSlice

## Objectif
Arrêter immédiatement le bot d'arbitrage lors d'un incident, diagnostiquer la cause et redémarrer en sécurité. Ce guide fournit une timeline minute par minute.

## Triggers automatiques (configurés dans `risk::kill_switch`)
- **RPC slot lag > 50** slots pendant plus de 10 s.
- **Échecs exécution CEX** > 5 par minute.
- **PnL drawdown** inférieur à -30 bps sur 30 minutes glissantes.
- **Alerte opérateur** via CLI `kill-switch --action arm`.

## Timeline d'intervention
| Minute | Actions | Responsable |
| --- | --- | --- |
| 0 | Armement automatique/manuelle détecté. Noter l'heure UTC, marché, raison dans `docs/logs/killswitch-incident.md`. | Dev on-call |
| 1 | Confirmer que les ordres CEX sont annulés : `cargo run -p cli -- --mode dry-run-cex --action status`. Vérifier `orders_open=0`. | Dev on-call |
| 2 | Vérifier les métriques : `curl -s localhost:9898/metrics | grep kill_switch`. Capturer la sortie. | Ops |
| 3 | Vérifier les flux : `engine_signals_total` figé, `ws_connection_status` OK. Si flux actifs, couper manuellement (`kill -SIGTERM <pid>`). | Dev référent |
| 5 | Lancer diagnostic selon le trigger (voir section Diagnostic). Documenter chaque commande dans `docs/logs/killswitch-incident.md`. | Dev référent |
| 10 | Communiquer sur Slack (#trading-bot) : message modèle « Kill-switch ARM – cause suspectée … – ETA resolution ». | Chef de projet |
| 15 | Implémenter correction (switch RPC, désactiver marché, réduire taille). | Dev on-call |
| 20 | Exécuter test de validation (paper 15 min) avant reprise live. | Dev référent |
| 35 | Si tests OK, désarmer via CLI (`kill-switch --action disarm --reason "<raison>"`). | Dev on-call |
| 45 | Reprendre mode live. Surveiller 15 min supplémentaires. | Ops |
| 60 | Rédiger post-mortem court et le déposer dans `docs/logs/killswitch-incident.md`. | Chef de projet |

## Checklist immédiate (Minutes 0-3)
1. Noter l'heure exacte (UTC), marché, raison.
2. Exécuter `cargo run -p cli -- --mode kill-switch --action status` et coller la sortie.
3. Vérifier annulation des ordres CEX (`exec.cancel_all` dans les logs).
4. Confirmer via `/metrics` que le compteur `kill_switch_active` = 1.

## Diagnostic (Minute 5 et plus)
- **RPC slot lag**
  - `solana slot` vs explorer https://explorer.solana.com/.
  - Basculer sur RPC secondaire (`solana config set --url <fallback>`).
  - Tester `solana ping`.
- **Échecs exécution CEX**
  - Ouvrir `logs/exec_failures.log`.
  - Vérifier statut API (Binance https://www.binance.com/en/status, OKX https://www.okx.com/announcements, Bybit https://status.bybit.com/).
  - Réduire `max_inflight_txs` si surcharge.
- **PnL drawdown**
  - Analyser `logs/paper.csv` (`pandas` rapide : `python analysis/quick_check.py`).
  - Identifier marchés déficitaires et appliquer `per_market_cap` réduit.

## Redémarrage sécurisé (Minutes 20-45)
1. Appliquer la correction (ex : changer `config/default.toml`, désactiver une paire).
2. Lancer `cargo run -p cli -- --mode paper --duration 900 --config config/default.toml`.
3. Vérifier que le kill-switch reste désarmé (`kill-switch --action status`).
4. Désarmer : `cargo run -p cli -- --mode kill-switch --action disarm --reason "fix applied"`.
5. Rebasculer en mode live (`cargo run -p cli -- --mode live`) en présence d'un pair.

## Escalade
- Si le kill-switch se réactive en < 30 min ou si la cause n'est pas identifiée, contacter immédiatement l'ingénieur principal (voir annuaire interne) et l'équipe Ops.
- Fournir :
  - Archives `logs/<date>` compressées.
  - Capture `/metrics`.
  - Derniers commits (`git log -1`).

## Historique des incidents
- Tenir `docs/logs/killswitch-incident.md` à jour (date, cause, actions, durée downtime).
- Après chaque incident, organiser un debrief < 24 h.
