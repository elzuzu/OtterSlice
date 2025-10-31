# SPRINT-005A — Mode paper trading temps réel

> **But :** exécuter la stratégie en conditions quasi réelles sans envoyer d'ordres live, en journalisant chaque décision.

## Pré-requis
- Tous les sprints précédents jusqu'à EPIC-004 validés (scanner, exécution, monitoring, kill-switch).
- Disponibilité d'un flux de marché stable (CEX/DEX).

## Livrables
1. Module `crates/paper/src/lib.rs` exposant :
   - `pub struct PaperEngine { recorder: CsvWriter, position_tracker: PositionTracker }`
   - `pub async fn run(duration: Duration, config: PaperConfig) -> Result<()>`
   - Simulation des ordres CEX/DEX à partir du scanner (latence configurable).
2. Fichier `docs/reports/paper_template.md` (template de rapport journalier) à compléter.
3. CLI `cargo run -p cli -- --mode paper --duration 3600 --output logs/paper.csv`.

## Étapes guidées
1. **Définir la configuration**
   - `struct PaperConfig { latency_cex_ms: u64, latency_dex_ms: u64, fill_ratio: f64, slippage_bps: f64 }`.
   - Charger depuis `config/default.toml` (section `[paper]`).
2. **Brancher le scanner**
   - Réutilise `Scanner` (SPRINT-003B). Lorsque le scanner envoie une opportunité, crée deux ordres simulés.
3. **Simuler les fills**
   - Utilise les carnets actuels pour estimer l'exécution :
     - CEX : consomme les niveaux jusqu'à `size`. Applique `fill_ratio` (ex : 0.85 signifie 85% rempli).
     - DEX : consomme le CLOB ou utilise `quote_clmm`.
   - Applique la latence : `tokio::time::sleep(Duration::from_millis(latency_cex_ms))` avant de "remplir".
4. **Position tracking**
   - `PositionTracker` maintient `inventory_sol`, `inventory_usdc`, `pnl_usd`, `max_drawdown_usd`.
   - Après chaque trade, met à jour la position et calcule le PnL.
5. **Enregistrement CSV**
   - Colonnes : `timestamp,market,signal_spread_bps,fill_ratio,notional,cex_price,dex_price,pnl_usd,inventory_sol`.
   - Utilise `csv::Writer`.
6. **Alertes**
   - Si `max_drawdown_usd` dépasse un seuil (config), loggue `ERROR` et déclenche `KillSwitch::arm()` (simulateur).
7. **Tests**
   - `test_simulated_fill`: opportunité 50 bps, notional 1000, `fill_ratio=1.0` → `pnl_usd` positif.
   - `test_drawdown_trigger`: pertes cumulées > seuil → kill-switch armé.
8. **CLI**
   - Mode `paper` : permet `--duration`, `--latency-cex`, `--latency-dex`, `--fill-ratio`.
   - Affiche un résumé toutes les 60s (`trades=xx, pnl=yy, drawdown=zz`).
9. **Rapport**
   - Remplis `docs/reports/paper_template.md` avec sections : Résumé, Hypothèses, Résultats, Incidents.
   - Journal `docs/logs/sprint-005A.md` : capture CSV + résumé CLI.

## Critères d'acceptation
- ✅ `cargo test -p paper` passe.
- ✅ `cargo run -p cli -- --mode paper --duration 120` génère un fichier CSV non vide.
- ✅ Rapport `paper_template.md` complété avec au moins un exemple de run.
- ✅ Journal complété.

## Dépendances
- Fournit les données pour SPRINT-005B (calibration & analyse post-run).

## Points d'attention
- Utilise `chrono::Utc::now()` pour timestamp (ISO8601).
- Nettoie les fichiers CSV précédents (`truncate` avant d'écrire) pour éviter les doublons.
- Garde le mode paper isolé : aucune fonction ne doit appeler directement les exécutors live.
