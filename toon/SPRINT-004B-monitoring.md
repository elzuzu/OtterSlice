# SPRINT-004B — Monitoring temps réel & alerting

> **But :** suivre l'état du bot (latence, succès d'ordres, exposition) en temps réel et déclencher des alertes.

## Pré-requis
- Les modules exec/risk produisent déjà des événements (`tracing`, `metrics`).
- Disposer d'un endpoint Prometheus local ou, a minima, écrire dans des fichiers CSV.

## Livrables
1. Module `crates/metrics` mettant à disposition :
   - `fn init_metrics(config: &MetricsConfig)` (configure Prometheus + logs JSON).
   - `fn record_order(report: &ExecutionReport, venue: Venue)`.
   - `fn record_latency(metric: &str, duration: Duration)`.
2. Dashboard Grafana (`docs/monitoring/dashboard.json`) avec panels : spreads, latence CEX/DEX, volume cumulé, nombre d'alertes.
3. Runbook `docs/runbooks/monitoring.md` expliquant comment interpréter les graphiques.
4. CLI `cargo run -p cli -- --mode metrics-demo --duration 60` générant des métriques factices.

## Étapes guidées
1. **Configurer Prometheus exporter**
   - Ajoute la dépendance `prometheus = "0.13"`, `axum = "0.7"`.
   - Implémente un serveur HTTP (`metrics::serve`) qui expose `/metrics`.
   - Ajoute un compteur `orders_total{venue="cex"|"dex"}` et un histogramme `latency_ms_bucket`.
2. **Brancher dans les modules existants**
   - Dans `exec::cex` et `exec::dex`, appelle `record_order` après chaque exécution.
   - Dans `engine::scanner`, enregistre le temps de calcul (`record_latency("scanner", elapsed)`).
3. **Alertes**
   - Implémente un watcher qui surveille `latency_ms > 500` ou `orders_failed_total` > 0 sur 1 minute : envoie une notification Slack (utilise `reqwest` POST vers un webhook) ou loggue `ERROR`.
   - Ajoute un fichier de configuration `config/monitoring.toml` contenant `prometheus_bind_addr`, `slack_webhook_url` (optionnel).
4. **Dashboard**
   - Crée `docs/monitoring/dashboard.json` (export Grafana) avec panels :
     - Graph latence CEX vs DEX.
     - Table opportunités vs net spread.
     - Stat panel sur `cex_orders_failed_total`.
   - Documente les IDs de datasource.
5. **Tests**
   - `cargo test -p metrics` :
     - Test que l'endpoint `/metrics` renvoie un contenu non vide.
     - Test que `record_order` incrémente bien les compteurs.
   - Ajoute un test `metrics_demo` (exécute `init_metrics`, enregistre des ordres, vérifie que le compteur augmente).
6. **CLI demo**
   - Mode `metrics-demo` :
     - Initialise les métriques.
     - Génère des opportunités factices et enregistre des ordres toutes les 2s.
     - Affiche l'URL `http://localhost:9898/metrics`.
7. **Journal**
   - `docs/logs/sprint-004B.md` : capture du contenu `/metrics`, screenshot Grafana (ajoute `docs/monitoring/dashboard.png`).

## Critères d'acceptation
- ✅ Endpoint `/metrics` accessible et contient les compteurs/histogrammes.
- ✅ Dashboard JSON versionné + capture PNG.
- ✅ Runbook monitoring détaillé.
- ✅ Journal complété.

## Dépendances
- S'appuie sur les rapports d'exécution et le scanner.
- Fournit les données au sprint 004C (kill-switch).

## Points d'attention
- Ne mets pas de secrets dans `monitoring.toml`. Pour Slack, stocke le webhook dans le trousseau et charge-le via `keyring`.
- Utilise des buckets d'histogramme adaptés (ex: [50, 100, 200, 500, 1000] ms).
- Vérifie que le serveur HTTP se ferme proprement (utilise `tokio::signal::ctrl_c`).
