# SPRINT-002D — Quoteur CLMM (Orca Whirlpools & Raydium Concentrated)

> **But :** construire un moteur local qui calcule les quotes de swap pour les pools concentrés sans dépendre d'APIs externes.

## Pré-requis
- SPRINT-001B terminé.
- Lecture recommandée :
  - Orca Whirlpool math : https://docs.orca.so/developers/whirlpools
  - Raydium CLMM math : https://docs.raydium.io/raydium/clmm/introduction
- Cloner les JSON d'IDL Whirlpool (`whirlpools.json`) et Raydium (`clmm_idl.json`) dans `third-parties/idl/`.

## Livrables
1. Module `crates/dex-clmm` fournissant :
   - `struct ClmmPoolState` (tick_current, sqrt_price_x64, liquidity, fee_rate_bps, positions).
   - `fn quote_swap(direction: SwapDirection, amount: u64, amount_specified_is_input: bool) -> QuoteResult`.
   - `async fn refresh_pool(pubkey: &Pubkey) -> Result<ClmmPoolState>` (lecture via RPC).
2. Script CLI `cargo run -p cli -- --mode quote-clmm --pool orca:SOL/USDC --amount 100`.
3. Tests unitaires comparant les résultats avec ceux du SDK TypeScript Orca (utilise fixtures JSON).

## Étapes guidées
1. **Définir les structures**
   - `ClmmPoolState` doit contenir : `sqrt_price_x64`, `liquidity`, `tick_current_index`, `tick_spacing`, `fee_rate_bps`, `reward_growths`, `timestamp`.
   - Crée `struct QuoteResult { amount_in: u64, amount_out: u64, fee_amount: u64, end_tick_index: i32 }`.
2. **Charger l'état du pool**
   - Utilise `solana_client::rpc_client::RpcClient::get_account_data` (non blocking).
   - Pour Orca : parse la structure via `whirlpool::state::Whirlpool` (IDL). Pour Raydium : se baser sur la doc pour mappe les bytes.
   - Ajoute une fonction `fn parse_whirlpool(data: &[u8]) -> Result<ClmmPoolState>`.
3. **Implémenter la quote**
   - Utilise les formules officielles :
     - `sqrt_price_x64` → prix = `(sqrt_price_x64 / 2^64)^2`.
     - Avancement tick par tick : boucle tant que `amount_remaining > 0`.
   - Ajoute la gestion `amount_specified_is_input` (utilise deux chemins `exact_in` et `exact_out`).
   - Gère les limites `sqrt_price_limit_x64` pour éviter de dépasser le tick suivant.
4. **Prendre en compte les fees**
   - Pour chaque step, calcule `fee = amount_step * fee_rate_bps / 10_000`.
   - Ajoute `fee_growth_global_x64` dans l'état si nécessaire.
5. **Gestion des positions**
   - Les positions du market maker doivent être lues pour connaître `liquidity_net`. Implémente `async fn load_positions(owner: &Pubkey) -> Result<Vec<PositionInfo>>` (stocke pour plus tard).
6. **Tests unitaires**
   - Ajoute un dossier `crates/dex-clmm/tests/fixtures` avec :
     - `orca_sol_usdc_snapshot.json` (extrait depuis `https://api.mainnet.orca.so/v1/whirlpool/list`).
     - `raydium_sol_usdc_snapshot.json` (depuis l'API officielle).
   - Tests :
     - `quote_exact_in` : 100 USDC in → ~1.79 SOL out (vérifie ±0.01).
     - `quote_exact_out` : 1 SOL out → ~55.8 USDC in (±0.05).
7. **CLI**
   - Ajoute un mode `quote-clmm` dans la CLI :
     ```bash
     cargo run -p cli -- --mode quote-clmm --pool orca:SOL/USDC --amount 100 --direction exact-in
     ```
   - La commande doit afficher : `amount_in`, `amount_out`, `fee_amount`, `end_tick_index`.
8. **Journal**
   - `docs/logs/sprint-002D.md` doit contenir : sortie de `cargo test -p dex-clmm`, exemples de CLI, note sur la précision obtenue.

## Critères d'acceptation
- ✅ `cargo test -p dex-clmm` réussit avec les fixtures.
- ✅ `cargo run -p cli -- --mode quote-clmm` renvoie un résultat cohérent (fee > 0, amount_out > 0).
- ✅ Documentation des formules dans le module (lien vers docs Orca/Raydium en commentaires).
- ✅ Journal complété.

## Dépendances
- Les quotes seront utilisées par le scanner (SPRINT-003B) et l'exécution DEX (SPRINT-003D).

## Points d'attention
- Attention à la conversion `u128` lors des multiplications (utilise `checked_mul` + `checked_div`).
- Pour Raydium, certaines données sont little-endian : utilise `u64::from_le_bytes`.
- Ajoute des garde-fous : si la liquidité tombe à zéro, retourne une erreur explicite.
