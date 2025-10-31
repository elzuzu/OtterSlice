# SPRINT-003A — Calculateur de frais & coûts transactionnels `[P0]`

> **Mission :** fournir des fonctions prêtes à l'emploi qui retournent les frais totaux (CEX + DEX + gas + slippage provisionnel) en basis points.
> **Priorité** : **P0** — clé pour la règle `net_spread ≥ fees + gas_bps + slip_exp + marge` utilisée par le scanner et le pre-trade.

## Pré-requis
- SPRINT-001B (workspace) et SPRINT-002A/002C/002D (sources de market data) complétés.
- Connaître les grilles de frais Binance/OKX/Bybit (taker) et les fees DEX (0.05% Phoenix, 0.3% Raydium, etc.).

## Livrables
1. Module `crates/common/src/fees.rs` exposant :
   - `pub struct FeeBreakdown { pub cex_bps: f64, pub dex_bps: f64, pub gas_bps: f64, pub slippage_bps: f64, pub total_bps: f64 }`
   - `pub fn compute_total(params: &FeeParams) -> FeeBreakdown`
   - `pub fn load_cex_fees(exchange: Exchange, account_tier: &str) -> FeeTable`
2. Fichier de configuration `config/fees.toml` définissant les frais par exchange/pool.
3. Tests unitaires validant les conversions (fees absolus → bps) et la prise en compte du volume.

## Étapes guidées
1. **Créer `config/fees.toml`**
   ```toml
   [binance]
   taker_bps = 10
   maker_bps = 5

   [okx]
   taker_bps = 8
   maker_bps = 4

   [bybit]
   taker_bps = 10
   maker_bps = 0

   [dex]
   phoenix_bps = 5
   openbook_bps = 3
   orca_bps = 6
   raydium_bps = 7

   [gas]
   sol_lamports_per_cu = 5000
   average_cu_per_tx = 220000
   usdc_per_sol = 150
   ```
   - Vérifie la syntaxe via `python -c "import tomli; tomli.load(open('config/fees.toml','rb'))"`.
2. **Implémenter `FeeParams`**
   - Champs : `exchange`, `cex_fee_model`, `dex_market`, `trade_notional_usd`, `expected_slippage_bps`, `compute_unit_price`, `compute_unit_consumption`.
   - Ajoute une méthode `fn from_trade(trade: &TradeContext) -> Self`.
3. **Calculer les frais CEX**
   - Utilise `trade_notional_usd * cex_taker_bps / 10_000` → convertis ensuite en bps (`value / trade_notional_usd * 10_000`).
   - Pour futures (Bybit), ajoute un paramètre `funding_bps` (par défaut 0).
4. **Calculer les frais DEX**
   - Phoenix/OpenBook : `fee_bps` lu dans `config/fees.toml`.
   - CLMM : extrais `fee_rate_bps` depuis `ClmmPoolState` (module 002D).
5. **Estimer le gas**
   - Convertis `compute_unit_price` (micro-lamports) et `compute_unit_consumption` en USD :
     `gas_usd = (compute_unit_price * compute_unit_consumption / 1_000_000) * (1e-9 SOL/lamport) * usdc_per_sol`.
   - Convertis ensuite en bps.
6. **Slippage provisionnel**
   - Ajoute `slippage_bps = max(expected_slippage_bps, observed_depth_bps)` (observed venant du reconstructeur CEX/DEX).
7. **Retourner `FeeBreakdown`**
   - `total_bps = cex_bps + dex_bps + gas_bps + slippage_bps`.
   - Ajoute `assert!(total_bps.is_finite())` pour éviter les NaN.
8. **Tests unitaires**
   - `test_fee_breakdown_spot`: notional 10_000 USDC sur Binance + Phoenix → attends `cex_bps=10`, `dex_bps=5`, `gas_bps≈7`, `total≈22`.
   - `test_fee_breakdown_clmm`: notional 5_000, `fee_rate_bps=30`, `compute_unit_price=2_000`, `compute_unit_consumption=250_000`.
   - `test_slippage_floor`: `expected_slippage_bps=4`, `observed=6` → résultat 6.
9. **Documentation**
   - Ajoute un README `crates/common/README.md` expliquant comment mettre à jour les frais (copie des sources officielles).
10. **Journal**
   - `docs/logs/sprint-003A.md` : captures des tests et d'un exemple CLI `cargo run -p cli -- --mode show-fees` (ajoute un mode qui imprime le breakdown pour SOL/USDC).

## Critères d'acceptation
- ✅ `cargo test -p common fees` passe.
- ✅ Mode CLI `show-fees` affiche chaque composant avec 2 décimales.
- ✅ `config/fees.toml` versionné + validé.
- ✅ Documentation sur la mise à jour des frais rédigée.

## Dépendances
- Fournit les données au SPRINT-003B (scanner). Utilisé par SPRINT-004A (pre-trade check).

## Points d'attention
- Ne mélange pas `bps` (basis points) et pourcentage : `1% = 100 bps`.
- `compute_unit_price` se mesure en **micro-lamports** : multiplie par `1e-6` avant conversion.
- Rends les fonctions pures (sans `async`) pour faciliter les tests.
