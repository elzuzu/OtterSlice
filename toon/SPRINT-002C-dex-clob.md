# SPRINT-002C — Drivers DEX CLOB (Phoenix & OpenBook v2) `[P0]`

> **Objectif :** fournir un module unique capable de lire les carnets Phoenix/OpenBook et d'envoyer des ordres IOC/FAK. Pas de raccourci : suis chaque étape.
> **Priorité** : **P0** — requis pour observer best bid/ask DEX et tester les ordres dry-run avant hedge réel.

## Pré-requis
- SPRINT-001B : workspace prêt.
- Avoir lu :
  - Phoenix SDK : https://docs.phoenix.trade/developer-resources
  - OpenBook v2 : https://github.com/openbook-dex/openbook-v2/blob/main/PROGRAM.md
- Solana CLI configuré avec un compte dev (utilise `solana-keygen new --outfile ~/.config/otterslice/dev.json`).

## Livrables
1. Module `crates/dex-clob` avec :
   - `async fn load_markets(rpc: &RpcClient) -> Result<Vec<MarketInfo>>`
   - `async fn subscribe_orderbook(market: &MarketInfo, sender: watch::Sender<ClobBook>)`
   - `async fn place_order(params: PlaceOrderParams) -> Result<Signature>`
2. Scripts CLI `cargo run -p cli -- --mode list-clob` et `--mode dry-run-clob`.
3. Tests d'intégration simulant un envoi d'ordre sur le devnet (utilise `solana-test-validator`).

## Étapes guidées
1. **Mettre à jour `Cargo.toml`**
   - Dépendances clés :
     ```toml
     solana-client = "1.18"
     solana-sdk = "1.18"
     solana-transaction = "1.18"
     solana-program = "1.18"
     spl-token = { version = "4", features = ["no-entrypoint"] }
     ```
   - Ajoute `phoenix-sdk = { git = "https://github.com/Ellipsis-Labs/phoenix-rs", rev = "main" }` (pinne un commit précis via `rev`).
   - Exécute `cargo check -p dex-clob`.
2. **Décrire les marchés**
   - Crée `struct MarketInfo { name: String, address: Pubkey, base_mint: Pubkey, quote_mint: Pubkey, lot_size: u64, tick_size: u64 }`.
   - Charge les adresses depuis `config/markets.toml` (ajoute un champ `phoenix_market` et `openbook_market`).
3. **Initialiser un `RpcClient`**
   - Utilise `solana_client::nonblocking::rpc_client::RpcClient::new_with_commitment`.
   - Lis les URLs depuis `config/default.toml` (`primary_http`).
4. **Souscrire aux carnets**
   - Phoenix : utilise `phoenix_sdk::client::PhoenixClient::subscribe_all_orderbooks`.
   - OpenBook : combine `rpc.get_account_data_async` + `solana_account_decoder::parse_token::token_amount_to_ui_amount`.
   - Normalise les niveaux dans une structure `ClobBook { bids: Vec<Level>, asks: Vec<Level>, slot: u64 }`.
5. **Publier les updates**
   - Chaque update doit être envoyé via `watch::Sender<ClobBook>`.
   - Ajoute un champ `source` (`Phoenix` ou `OpenBook`) pour les logs.
6. **Placer un ordre IOC**
   - Implémente `PlaceOrderParams { market: MarketInfo, side: Side, price_lots: u64, size_lots: u64 }`.
   - Construit la transaction avec `solana_sdk::instruction::Instruction` (Phoenix `place_order` ou OpenBook `place_order`).
   - Utilise `ComputeBudgetInstruction::set_compute_unit_price(2_000)` pour éviter le throttling.
   - Signe la transaction avec la clé `~/.config/otterslice/dev.json`.
7. **Tests d'intégration**
   - Lance un `solana-test-validator` local (commande : `solana-test-validator --reset --bpf-program ...`).
   - Déploie les programmes mocks (Phoenix et OpenBook ne sont pas fournis : utilise les versions officielles ou un mock minimal qui renvoie une réponse).
   - Pour simplifier, crée un test `#[ignore]` qui nécessite un validator local.
8. **CLI de démonstration**
   - `--mode list-clob` : affiche tous les marchés configurés avec leur lot/tick.
   - `--mode dry-run-clob --market phoenix:SOL/USDC --side buy --price 55.12 --size 10` : calcule `price_lots` et `size_lots` puis affiche la transaction (ne l'envoie pas si `--dry`).
   - Ajoute une option `--send` pour réaliser l'envoi réel (devnet uniquement pour l'instant).
9. **Journalisation**
   - Dans `docs/logs/sprint-002C.md`, consigne :
     - Résultats de `cargo check -p dex-clob`.
     - Sortie de `cargo run -p cli -- --mode list-clob`.
     - Exemple de transaction signée (hash).

## Critères d'acceptation
- ✅ `cargo check -p dex-clob` et `cargo test -p dex-clob` passent.
- ✅ `cargo run -p cli -- --mode list-clob` liste au moins 6 marchés (Phoenix + OpenBook pour 3 paires).
- ✅ Mode `dry-run-clob` calcule correctement `price_lots`/`size_lots` (vérifie avec la formule du README Phoenix).
- ✅ Journal `docs/logs/sprint-002C.md` complété avec captures.

## Dépendances
- Utilise la configuration du SPRINT-001B.
- Alimente le SPRINT-003D (hedge DEX) et SPRINT-003B (scanner spreads).

## Points d'attention
- Respecte la précision des lots (arrondi vers le bas pour `buy`, vers le haut pour `sell`).
- Pour Phoenix, la transaction doit inclure l'instruction `log_authority_check` si le programme l'exige (vérifie la doc).
- Ne commit pas de clés privées : utilise les chemins configurables.
