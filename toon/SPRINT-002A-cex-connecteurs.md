# SPRINT-002A — Connecteurs CEX REST/WS

> **Public visé :** développeur junior découvrant les APIs CEX. Toutes les étapes sont détaillées pour éviter les confusions.

## Préparation avant de coder
- Lis les pages officielles d'API :
  - Binance Spot API : https://binance-docs.github.io/apidocs/spot/en
  - OKX API v5 : https://www.okx.com/docs-v5/en
  - Bybit Unified v5 : https://bybit-exchange.github.io/docs/spot/public
- Dans le trousseau `OtterSlice`, crée les entrées suivantes (si absentes) :
  - `binance_api_key` / `binance_api_secret`
  - `okx_api_key` / `okx_api_secret` / `okx_api_passphrase`
  - `bybit_api_key` / `bybit_api_secret`
  Note leurs noms exacts, on les utilisera dans le code.
- Ouvre `docs/logs/sprint-002A.md` et prépare le journal (date, opérateur, sections "Commandes" et "Tests")
  ```markdown
  # Sprint 002A — Journal connecteurs CEX
  - Date : <JJ/MM/AAAA>
  - Opérateur : <Nom>

  ## Commandes

  ## Tests manuels
  ```

## Objectifs livrables
1. Module `crates/cex` exposant :
   - `struct RestSnapshot { bids: Vec<OrderBookLevel>, asks: Vec<OrderBookLevel>, last_update_id: u64 }`
   - `async fn fetch_snapshot(exchange: Exchange, symbol: &str) -> anyhow::Result<RestSnapshot>`
   - `async fn subscribe_orderbook(exchange: Exchange, symbol: &str, handler: impl FnMut(WsDiffEvent))`
2. Gestion centralisée des credentials via `keyring`.
3. Tests unitaires `cargo test -p cex` couvrant la désérialisation REST + WS.
4. Script de démonstration `cargo run -p cli -- --mode smoke-cex` affichant un snapshot puis 3 diffs.

## Étapes guidées
1. **Mettre à jour les dépendances de `crates/cex/Cargo.toml`**
   - Ajoute :
     ```toml
     [dependencies]
     anyhow.workspace = true
     thiserror.workspace = true
     serde.workspace = true
     serde_json.workspace = true
     tokio.workspace = true
     tracing.workspace = true
     reqwest = { version = "0.11", default-features = false, features = ["json", "rustls-tls"] }
     tokio-tungstenite = { version = "0.21", default-features = false, features = ["rustls-tls-webpki-roots"] }
     hmac = "0.12"
     sha2 = "0.10"
     url = "2"
     flate2 = { version = "1", features = ["gzip"] }
     async-trait = "0.1"
     keyring = "2"
     ```
   - Exécute `cargo check -p cex` pour vérifier qu'il n'y a pas d'erreur (colle la sortie dans le journal).
2. **Définir les types communs**
   - Dans `crates/cex/src/lib.rs`, crée :
     ```rust
     pub mod model {
         use serde::Deserialize;

         #[derive(Debug, Clone, Deserialize)]
         pub struct OrderBookLevel {
             pub price: f64,
             pub quantity: f64,
         }

         #[derive(Debug, Clone, Deserialize)]
         pub struct RestSnapshot {
             pub bids: Vec<OrderBookLevel>,
             pub asks: Vec<OrderBookLevel>,
             pub last_update_id: u64,
         }

         #[derive(Debug, Clone)]
         pub struct WsDiffEvent {
             pub bids: Vec<OrderBookLevel>,
             pub asks: Vec<OrderBookLevel>,
             pub first_update_id: u64,
             pub final_update_id: u64,
         }

         #[derive(Debug, Copy, Clone)]
         pub enum Exchange {
             Binance,
             Okx,
             Bybit,
         }
     }
     ```
   - Ajoute également un module `error` pour encapsuler `thiserror::Error`.
3. **Créer un trait de connecteur**
   - Dans `crates/cex/src/connectors/mod.rs` :
     ```rust
     #[async_trait::async_trait]
     pub trait RestConnector {
         async fn fetch_snapshot(&self, symbol: &str) -> Result<RestSnapshot>;
     }

     #[async_trait::async_trait]
     pub trait WsConnector {
         async fn subscribe<F>(&self, symbol: &str, mut handler: F) -> Result<()>
         where
             F: FnMut(WsDiffEvent) + Send + 'static;
     }
     ```
   - Implémente le trait pour chaque bourse dans des fichiers dédiés (`binance.rs`, `okx.rs`, `bybit.rs`).
4. **Implémenter le client REST pour Binance**
   - Point d'API : `GET https://api.binance.com/api/v3/depth`.
   - Utilise `reqwest::Client::builder().timeout(Duration::from_secs(5))`.
   - Exemple de parsing :
     ```rust
     #[derive(Deserialize)]
     struct BinanceDepthResponse {
         lastUpdateId: u64,
         bids: Vec<(String, String)>,
         asks: Vec<(String, String)>,
     }
     ```
   - Convertis `String` → `f64` avec `parse::<f64>()?`. Loggue les erreurs avec `tracing::error!` et remonte `anyhow::Context`.
5. **Implémenter les clients REST OKX et Bybit**
   - OKX nécessite le paramètre `instId=SOL-USDT`. Les quantités sont des `String`.
   - Bybit renvoie `result.list` ; prends l'élément `[0]`.
   - Ajoute une fonction utilitaire `fn to_levels(raw: &[(String, String)]) -> Result<Vec<OrderBookLevel>>`.
6. **Gestion du throttling REST**
   - Dans `crates/cex/src/throttle.rs`, crée une structure `RateLimiter` avec `tokio::sync::Semaphore` et `tokio::time::Instant`.
   - Initialise une limite par exchange (`BINANCE_WEIGHT: 1200/min`, `OKX_REQS: 60/s`, `BYBIT_REQS: 50/s`).
   - Ajoute des tests unitaires qui simulent 3 appels simultanés et vérifient que le délai minimum est respecté (utilise `tokio::time::pause`).
7. **Implémenter la connexion WebSocket**
   - Crée une fonction `async fn ws_stream(url: &str) -> Result<WebSocketStream<...>>` qui applique un timeout de connexion 10s.
   - Pour Binance : abonne-toi au canal `format!("{}@depth@100ms", symbol.to_lowercase())`.
   - Pour OKX : envoie un message `{"op":"subscribe","args":[{"channel":"books","instId":"SOL-USDT"}]}`.
   - Pour Bybit : message `{"op":"subscribe","args":["orderbook.1.SOLUSDT"]}`.
   - Ajoute une boucle `loop { match stream.next().await { ... } }` qui :
     - Décode `gzip` (OKX) via `flate2::bufread::GzDecoder`.
     - Convertit le JSON en `WsDiffEvent`.
     - Gère les pings (`{"event":"ping"}` → réponds `{"event":"pong"}`).
     - Sur erreur réseau, attend 2, 4, 8 secondes (backoff exponentiel) avant de reconnecter.
8. **Gestion des clés API et signature HMAC**
   - Crée `fn read_key(service: &str, account: &str) -> Result<String>` utilisant `keyring::Entry::new(service, account)?`.
   - Implémente `fn sign(secret: &str, payload: &str) -> String` renvoyant une chaîne hex.
   - Pour l'instant les endpoints utilisés sont publics, mais le test d'intégration doit vérifier que `sign("secret", "payload")` retourne la valeur attendue (`hmac::Mac` + `hex::encode`).
9. **Écrire des tests unitaires**
   - Dans `crates/cex/src/tests.rs`, ajoute :
     - Test de parsing d'une réponse Binance mockée (utilise `serde_json::from_str`).
     - Test de décompression OKX (`base64` d'un payload gzip → `GzDecoder`).
     - Test du rate limiter (`tokio::time::pause`, `advance`).
   - Lance `cargo test -p cex` et colle la sortie dans le journal.
10. **Créer un mode CLI de démonstration**
    - Dans `crates/cli/src/main.rs`, ajoute une commande `--mode smoke-cex`.
    - Implémente :
      ```rust
      if args.mode == Mode::SmokeCex {
          let snapshot = cex::binance::Client::default().fetch_snapshot("SOLUSDT").await?;
          println!("snapshot bids={} asks={}", snapshot.bids.len(), snapshot.asks.len());
          cex::binance::Client::default()
              .subscribe("SOLUSDT", |event| {
                  println!("diff {} {}", event.first_update_id, event.final_update_id);
              })
              .await?;
      }
      ```
    - L'objectif est de démontrer que le flux fonctionne pendant ~5 secondes (ajoute un `tokio::time::sleep(Duration::from_secs(5))`).
11. **Validation manuelle**
    - Démarre `cargo run -p cli -- --mode smoke-cex --exchange binance --symbol SOLUSDT` (ajoute des arguments pour choisir l'exchange).
    - Observe dans le terminal :
      - "snapshot bids=1000 asks=1000"
      - Trois lignes `diff` consécutives.
    - Capture la sortie dans le journal.

## Critères d'acceptation
- ✅ `cargo test -p cex` passe sans échec.
- ✅ `cargo run -p cli -- --mode smoke-cex --exchange binance --symbol SOLUSDT` affiche un snapshot puis des diffs.
- ✅ Les logs `tracing` montrent la reconnexion automatique (force une coupure réseau en débranchant le câble ou en stoppant le Wi-Fi 10 s, puis vérifie que la boucle reprend).
- ✅ Le journal `docs/logs/sprint-002A.md` contient les sorties de tests + capture de logs.

## Dépendances & suites
- Dépend du SPRINT-001B (workspace).
- Fournit les flux bruts au SPRINT-002B (reconstructeur L2). Partage les types `RestSnapshot` et `WsDiffEvent`.

## Points d'attention
- Toujours convertir les prix/quantités en `f64` **après** avoir vérifié que la chaîne n'est pas vide. Utilise `with_context(|| format!("invalid price {value}"))`.
- Loggue les tentatives de reconnexion avec `tracing::warn!` (cela sert de preuve lors des tests).
- Sur `429` REST, dors 1000 ms et retente, mais limite le nombre de retries à 5 pour éviter les boucles infinies.
