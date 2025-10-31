# SPRINT-003B — Scanner d'opportunités net-frais `[P0]`

> **Objectif :** parcourir en continu les books CEX/DEX pour calculer le spread net des frais et déclencher un signal exploitable.
> **Priorité** : **P0** — pivot décisionnel : la règle `net_spread ≥ fees + gas_bps + slip_exp + marge` doit être appliquée avant toute exécution.

## Pré-requis
- SPRINT-002B (reconstructeur CEX), SPRINT-002C (CLOB), SPRINT-002D (CLMM) et SPRINT-003A (frais) finalisés.
- Les modules doivent exposer des `watch::Receiver` pour les vues orderbook.

## Livrables
1. Module `crates/engine/src/scanner.rs` exposant :
   - `pub struct ScannerConfig { poll_interval_ms: u64, min_spread_bps: f64, warmup_ticks: usize }`
   - `pub struct Opportunity { market: MarketPair, spread_bps: f64, net_spread_bps: f64, cex_side: Side, dex_side: Side, size: f64 }`
   - `pub async fn run_scanner(cfg: ScannerConfig, channels: ScannerInputs, sender: mpsc::Sender<Opportunity>)`
2. Intégration dans la CLI `cargo run -p cli -- --mode scan --duration 30s`.
3. Tests unitaires + test de charge (bench) montrant que le scanner traite >500 updates/s sans backlog.

## Étapes guidées
1. **Structurer les entrées**
   - `ScannerInputs` contient : `cex_book: watch::Receiver<OrderBookView>`, `clob_book: watch::Receiver<ClobBook>`, `clmm_quote: watch::Receiver<QuoteResult>`, `fees: FeeBreakdown`.
   - Implémente un builder pour simplifier l'injection lors des tests (`ScannerInputs::builder()`).
2. **Boucle principale**
   - Utilise `tokio::time::interval(Duration::from_millis(cfg.poll_interval_ms))`.
   - À chaque tick :
     - Lit la dernière valeur de chaque channel (`borrow().clone()`).
     - Calcule la profondeur exécutable :
       - CEX → consomme les niveaux jusqu'à `size_usd` (prend en compte `depth.bids` ou `depth.asks`).
       - DEX CLOB → idem.
       - CLMM → utilise `QuoteResult` pour la taille.
     - Calcule `spread_bps = ((cex_price - dex_price) / dex_price) * 10_000` (ou inverse selon sens).
     - `net_spread_bps = spread_bps - fees.total_bps`.
   - Si `net_spread_bps >= cfg.min_spread_bps` et `warmup_ticks` dépassé, envoie une `Opportunity`.
3. **Gestion du sens (CEX buy vs sell)**
   - Détermine la meilleure combinaison `cex buy + dex sell` et `dex buy + cex sell`. Choisis la meilleure net spread.
   - Stocke le sens choisi dans l'opportunité (`cex_side`, `dex_side`).
4. **Limiter les faux positifs**
   - Ajoute un filtre : n'envoie pas deux opportunités sur le même marché dans un intervalle < 500 ms (utilise `HashMap<MarketPair, Instant>`).
   - Ajoute un `min_depth_usd` (ex: 100 USDC) pour ignorer les signaux trop faibles.
5. **Tests unitaires**
   - `test_positive_opportunity`: injecte des vues artificielles (prix CEX 55.5, DEX 55.0, fees 20 bps) → vérifie qu'une opportunité est envoyée.
   - `test_negative_spread`: `net_spread_bps` < 0 → aucun message.
   - `test_warmup`: premier `warmup_ticks` ignoré.
   - Utilise `tokio::test` et `mpsc::channel(1)`.
6. **Test de performance**
   - Ajoute un bench (feature `bench`) ou un test `#[tokio::test]` avec 1000 updates en rafale. Mesure `elapsed` < 100 ms.
7. **CLI**
   - Mode `scan` :
     ```bash
     cargo run -p cli -- --mode scan --duration 30s --min-spread-bps 25
     ```
   - Affiche les opportunités sous forme de tableau (utilise `tabled` ou simple `println!`).
   - Ajoute une option `--export opportunities.csv`.
8. **Journal**
   - `docs/logs/sprint-003B.md` :
     - Sorties `cargo test -p engine scanner`.
     - Capture d'un run CLI avec au moins une opportunité.
     - Note les paramètres utilisés (poll interval, min spread).

## Critères d'acceptation
- ✅ Les tests scanner passent en < 2 s.
- ✅ Le CLI affiche un minimum de 1 opportunité lors d'un test sur données mockées.
- ✅ `opportunities.csv` contient les colonnes `timestamp,market,spread_bps,net_spread_bps,size`.
- ✅ Journal complété avec evidence.

## Dépendances
- Les sorties alimentent SPRINT-003C (execution CEX) et SPRINT-003D (hedge DEX).

## Points d'attention
- Utilise des `f64` mais garde un `Decimal` (via `rust_decimal`) pour éviter les erreurs cumulées si nécessaire.
- Gère les `Option` : si un channel n'a pas encore publié ( `has_changed()` false ), skip le tick et loggue `tracing::debug!`.
- Prépare des hooks pour la télémétrie (compteur du nombre d'opportunités, latence de calcul).
