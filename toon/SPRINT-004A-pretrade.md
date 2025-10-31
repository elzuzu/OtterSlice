# SPRINT-004A — Contrôles pre-trade & allocation de capital `[P0]`

> **Objectif :** garantir qu'avant chaque arbitrage, la taille envoyée respecte les limites de risque et la disponibilité de capital.
> **Priorité** : **P0** — activer caps notionnels, spread minimum, balance checks et exposition nette avant toute exécution réelle.

## Pré-requis
- Modules d'exécution CEX/DEX (SPRINT-003C/003D) et calculateur de frais (SPRINT-003A) disponibles.
- Base de données ou stockage léger (utilise `sled` ou fichiers CSV) pour suivre l'exposition.

## Livrables
1. Module `crates/risk/src/pretrade.rs` exposant :
   - `pub struct PreTradeCheckRequest { market: MarketPair, notional_usd: f64, cex_account: Exchange, dex_venue: DexVenue }`
   - `pub struct PreTradeDecision { allowed: bool, reason: Option<String>, adjusted_notional: f64 }`
   - `pub fn evaluate(request: &PreTradeCheckRequest, limits: &RiskLimits, balances: &BalanceSheet) -> PreTradeDecision`
2. Chargement des limites depuis `config/default.toml` (`risk` section).
3. CLI `cargo run -p cli -- --mode pretrade --market SOL/USDC --notional 500`.
4. Tests unitaires couvrant les cas limites (cap global, cap par marché, max positions, min spread).

## Étapes guidées
1. **Définir les structures de limites**
   - `struct RiskLimits { max_notional_usd: f64, per_market_cap_usd: f64, max_open_positions: u32, max_daily_loss_usd: f64 }`.
   - Charger ces valeurs depuis `config/default.toml` (utilise `serde::Deserialize`).
2. **Construire la feuille de balance**
   - `struct BalanceSheet { cex_balances: HashMap<Exchange, f64>, dex_balances: HashMap<String, f64>, open_positions: HashMap<MarketPair, f64> }`.
   - Implémente `fn load_from_disk(path: &Path) -> Result<Self>` (lecture d'un fichier JSON `state/balance.json`).
3. **Évaluer la disponibilité**
   - Vérifie `notional_usd <= balances.cex_balances[exchange]` et `notional_usd <= balances.dex_balances[market.quote]`.
   - Si insuffisant, réduit `adjusted_notional` à ce qui est disponible.
4. **Vérifier les caps**
   - `if balances.open_positions_total() + adjusted_notional > limits.max_notional_usd` → `allowed=false`.
   - `if balances.open_positions[market] + adjusted_notional > limits.per_market_cap_usd` → `allowed=false`.
5. **Contrôler le net spread**
   - Optionnel : injecte le `net_spread_bps` du scanner. Refuse si `< limits.min_spread_bps`.
6. **Gérer le kill-switch**
   - Si un flag `state/kill_switch.json` contient `true`, retourne `allowed=false` avec raison "kill switch active".
7. **Mise à jour de la feuille de balance**
   - Si autorisé, réserve le montant (`balances.reserve(market, adjusted_notional)`) et sauvegarde le fichier.
8. **Tests**
   - `test_cap_per_market` : cap = 1000, notional 1200 → refuse.
   - `test_adjust_notional` : balance CEX 300, demande 500 → autorise 300.
   - `test_kill_switch` : fichier kill-switch `true` → refuse.
9. **CLI**
   - Mode `pretrade` affiche :
     ```text
     Market: SOL/USDC
     Requested notional: 500
     Decision: allowed (adjusted: 300) — reason: limited by CEX balance
     ```
   - Ajoute `--state state/balance.json`.
10. **Journal**
    - `docs/logs/sprint-004A.md` : captures des tests + exemple CLI.

## Critères d'acceptation
- ✅ `cargo test -p risk pretrade` réussit.
- ✅ CLI `pretrade` renvoie un message compréhensible.
- ✅ Les fichiers `state/balance.json` et `state/kill_switch.json` sont documentés (ajoute `docs/runbooks/state-files.md`).
- ✅ Journal complété.

## Dépendances
- Utilisé par SPRINT-003C/003D avant envoi d'ordres.
- Fournit l'état pour SPRINT-004B (monitoring temps réel).

## Points d'attention
- Gère les valeurs `NaN` ou négatives (retourne `allowed=false` et loggue une erreur).
- Utilise des verrous (`parking_lot::Mutex`) pour éviter les courses lors de la mise à jour du fichier.
- Prévois un test de sérialisation/désérialisation du `BalanceSheet`.
