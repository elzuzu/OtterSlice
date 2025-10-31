**Directive Claude (à respecter à la lettre)**

* Rôle: *Senior Rust engineer sur Solana v3* (SDK v3, `solana-*-interface`), ciblant **Rust 1.90** sur **macOS M-series**.
* **Génère du code finalisé, zéro placeholder**: **interdit** d’émettre `todo!()`, `unimplemented!()`, `panic!()` non justifiés, sections vides, “exemples à adapter” ou pseudocode.
* **Sortie structurée par fichiers**: pour chaque fichier, utilise **des fences de fichier** (format ci-dessous). Un fichier = contenu intégral.
* **Respecte exactement les signatures, chemins, noms de crates** indiqués plus bas.
* **N’ajoute aucune dépendance** non listée; **Rust stable 1.90 uniquement**.
* Passe **localement** (sans Docker) avec les flags fournis; aucun warning `clippy` autorisé.
* Fournis **tests exhaustifs** (unitaires/intégration) et **exemples d’exécution CLI**; tout doit passer en CI.

Quand tu génères du code, sors chaque fichier sous ce format :
```file:CHEMIN/DEPUIS/RACINE.rs
// contenu entier du fichier, prêt à compiler
```

Ne mets **aucun autre texte** entre les blocs `file:`. Termine par un récapitulatif.

# SPRINT-002A — Connecteurs CEX REST/WS `[P0]`

## Objectifs
- Implémenter clients REST/WS pour Binance Futures, OKX Perp, Bybit Perp en respectant intégralement les endpoints listés (EPIC-002).
- Gérer signature (`HMAC_SHA256`, base64) et throttling (`X-MBX-USED-WEIGHT-1M`, `x-ratelimit-remaining`, `X-Bapi-Limit`).
- Produire tests unitaires avec vecteurs concrets (timestamps/signatures fournis ci-dessous).

## Endpoints à implémenter (copier-coller exact)
Se référer aux tableaux de l’EPIC-002 — tous les chemins/méthodes doivent être codés. Ajouter les endpoints supplémentaires :
- Binance `GET /fapi/v1/exchangeInfo` (cache 60 s).
- OKX `GET /api/v5/public/instruments?instType=SWAP`.
- Bybit `GET /v5/market/tickers?category=linear&symbol=BTCUSDT`.

## Signatures de test
- Binance :
  ```text
  secret = "abc"
  query = "symbol=BTCUSDT&side=BUY&type=LIMIT&timeInForce=IOC&quantity=0.01&price=45000&timestamp=1700000000000"
  signature attendu = "f7af7f69c6f798706df3aa5dbcf96f8dd7d45fb47a832983b472151797f63011"
  ```
- OKX :
  ```text
  secret = "MIIEv...==" (base64),
  timestamp = "2024-01-01T00:00:00.000Z"
  prehash = "2024-01-01T00:00:00.000ZPOST/api/v5/trade/order{\"instId\":\"BTC-USDT-SWAP\",\"tdMode\":\"cross\"}"
  signature attendu (base64) = "Uy6XoCjCQcE3Vq1W1dtJg5vh1J5m7r22+W8cJ7xv60Y="
  ```
- Bybit :
  ```text
  secret = "xyz"
  payload = "1700000000000abc5000{\"category\":\"linear\",\"symbol\":\"BTCUSDT\"}"
  signature attendu = "c9b2bbbe7c7d0df5703d667d49b2a9e8d7580b94b9200ee0d3c79906161efd84"
  ```

## Architecture
- Créer module `crates/cex/src/client.rs` exposant :
  ```rust
  pub struct RestClient { /* http client */ }
  pub struct WsClient { /* ws connection */ }

  impl RestClient {
      pub async fn get_time(&self) -> Result<DateTime<Utc>, ClientError> { /* ... */ }
      pub async fn get_depth(&self, symbol: &str, limit: u16) -> Result<OrderBookSnapshot, ClientError>;
      pub async fn place_order(&self, payload: NewOrderRequest) -> Result<NewOrderResponse, ClientError>;
      // autres endpoints listés
  }
  ```
- `ClientError` doit mapper codes HTTP/WS (`TooManyRequests`, `InvalidSignature`, `Timeout`).
- WebSocket : reconnect automatique (backoff 250 ms → 4 s), ping/pong selon venue.

## Tests
- `cex/tests/signature_vectors.rs` : comparer signatures attendues.
- `cex/tests/orderbook_snapshot.rs` : charger `fixtures/binance/depth_snapshot.json` (fichier à fournir) et vérifier parse.
- `cex/tests/ws_keepalive.rs` : simuler ping/pong.

## Bench & perf
- `cargo bench -p cex --bench rest_latency` : average < 20 ms (mock).
- `cargo test --package cex -- --ignored` pour tests réseau manuels.

## Exemples valides/invalides
- ✅ Endpoint `POST /fapi/v1/order` testé avec signature.
- ❌ Placeholder `todo!()` dans un match d’erreur.

## Checklist de validation
- Tous les endpoints listés ont un test ou une fixture.
- `cargo clippy -p cex --all-targets -D warnings` passe.
- `just ci` inclut ce crate.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
