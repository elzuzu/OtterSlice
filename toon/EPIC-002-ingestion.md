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

# EPIC-002 — Ingestion Marchés CEX/DEX `[P0]`

> **Vision :** disposer d'un pipeline de données fiable qui fournit en permanence des carnets cohérents, normalisés et enrichis pour tous les modules amont.
> **Priorité** : **P0** — conditionne le scanner et l'exécution. Sans L2 cohérent, aucun arbitrage n'est autorisé.

## Résultats attendus
- `crates/cex`, `crates/dex-clob`, `crates/dex-clmm`, `crates/ingest` opérationnels, documentés, testés (unitaires + intégration).
- Bus interne (`watch`/`broadcast`) alimentant le scanner avec des vues normalisées (`OrderBookView`, `ClobBook`, `QuoteResult`).
- Procédures de resynchronisation écrites (documents + tests) pour gérer les déconnexions WS et les mismatches d'update ID.
- Mesures de latence (<150 ms pour les updates CEX, <500 ms pour DEX) consignées dans `docs/logs/latency-benchmark.md`.

## Endpoints REST & WebSocket obligatoires
Copie intégrale des définitions suivantes dans les specs de sprint correspondantes (`SPRINT-002A`, `SPRINT-002B`, `SPRINT-002C`, `SPRINT-002D`). Les tests unitaires doivent véhiculer les exemples fournis.

### Binance (futures USDⓈ-M)
| Type | Méthode | Chemin | Query/Body | Auth | Notes |
|------|---------|--------|------------|------|-------|
| REST | `GET` | `/fapi/v1/time` | `recvWindow=5000` (optionnel) | Aucun | Vérifier dérive horloge | 
| REST | `POST` | `/fapi/v1/listenKey` | Body vide | API Key header | Rafraîchir toutes les 60 min |
| REST | `PUT` | `/fapi/v1/listenKey` | Body vide | API Key header | Keep-alive listenKey |
| REST | `DELETE` | `/fapi/v1/listenKey` | Body vide | API Key header | Cleanup |
| REST | `GET` | `/fapi/v1/depth` | `symbol=BTCUSDT&limit=500` | Aucun | Snapshot carnet |
| REST | `POST` | `/fapi/v1/order` | JSON `{ "symbol": "BTCUSDT", "side": "BUY", "type": "LIMIT", "timeInForce": "IOC", "quantity": "0.01", "price": "45000.0", "timestamp": 1700000000000 }` signé (`HMAC_SHA256(secret, queryString)`) | API Key + signature | Utiliser `timestamp` synchrone avec `/time` |
| WS | `wss://fstream.binance.com/ws/<listenKey>` | Subscribe `{ "method": "SUBSCRIBE", "params": ["btcusdt@depth@100ms"], "id": 1 }` | Auth via listenKey | Ping/pong `@pong` |
| WS | `wss://fstream.binance.com/stream?streams=btcusdt@depth@100ms` | Stream multiplexé | Aucun | Utiliser `lastUpdateId`, `pu`, `b`, `a` |

### OKX (perp USDT)
| Type | Méthode | Chemin | Query/Body | Auth | Notes |
|------|---------|--------|------------|------|-------|
| REST | `GET` | `/api/v5/public/time` | None | Aucun | Latence < 100 ms |
| REST | `POST` | `/api/v5/account/set-position-mode` | `{ "posMode": "net_mode" }` | Signature ECDSA (`OK-ACCESS-KEY`, `OK-ACCESS-SIGN`, `OK-ACCESS-TIMESTAMP`, `OK-ACCESS-PASSPHRASE`) | Pré-requis ordres |
| REST | `POST` | `/api/v5/trade/order` | `{ "instId": "BTC-USDT-SWAP", "tdMode": "cross", "side": "buy", "ordType": "ioc", "sz": "1", "px": "45000" }` | Signature  | `timestamp + method + path + body` |
| REST | `GET` | `/api/v5/market/books` | `instId=BTC-USDT-SWAP&sz=400` | Aucun | Snapshot |
| WS | `wss://ws.okx.com:8443/ws/v5/public` | Subscribe `{ "op": "subscribe", "args": [{ "channel": "books5", "instId": "BTC-USDT-SWAP" }] }` | Aucun | Maintenir `seqId`, `prevSeqId` |
| WS | `wss://ws.okx.com:8443/ws/v5/private` | Login `{ "op": "login", "args": [{"apiKey": "...", "passphrase": "...", "timestamp": "2024-01-01T00:00:00.000Z", "sign": "..."}] }` | Signature | Flux fills/ordres |

### Bybit (perp USDT)
| Type | Méthode | Chemin | Query/Body | Auth | Notes |
|------|---------|--------|------------|------|-------|
| REST | `GET` | `/v5/market/time` | None | Aucun | Sync horloge |
| REST | `GET` | `/v5/market/orderbook` | `category=linear&symbol=BTCUSDT&limit=200` | Aucun | Snapshot |
| REST | `POST` | `/v5/order/create` | `{ "category": "linear", "symbol": "BTCUSDT", "side": "Buy", "orderType": "Limit", "qty": "0.01", "price": "45000", "timeInForce": "IOC", "timestamp": 1700000000000 }` signé `HMAC_SHA256(secret, timestamp+apiKey+recvWindow+body)` | API Key + signature | Inclure header `X-BAPI-SIGN-TYPE: 2` |
| REST | `POST` | `/v5/user/create-listen-key` | Body vide | API Key | WS privé |
| WS | `wss://stream.bybit.com/v5/public/linear` | Subscribe `{ "op": "subscribe", "args": ["orderbook.50.BTCUSDT"] }` | Aucun | Diffs incrémentaux `u`, `seq`, `prev_seq` |
| WS | `wss://stream.bybit.com/v5/private` | Auth message `{ "op": "auth", "args": [apiKey, timestamp, signature] }` | Signature `HMAC_SHA256` | Flux ordres |

## Contrats de flux L2
- `DepthBook` (CEX) :
  ```rust
  struct DepthBook {
      bids: Vec<Level>,
      asks: Vec<Level>,
      last_update_id: u64,
      exchange_ts: DateTime<Utc>,
  }

  struct Level {
      price: Decimal,
      quantity: Decimal,
  }
  ```
- Règles :
  1. Appliquer snapshot complet (`lastUpdateId`) puis diffs (`U`, `u`, `pu` selon venue) en garantissant `pu + 1 == U`.
  2. En cas de gap (`pu + 1 != U`) ou latence > 1 s, déclencher resnapshot.
  3. Test d’intégrité : `tests/replay/binance_depth.rs` doit rejouer `fixtures/binance/btcusdt_depth_snapshot.json` + `fixtures/binance/btcusdt_depth_diffs.json` et vérifier cohérence (`best_bid <= best_ask`, volumes agrégés > 0).

## DEX CLOB & CLMM — spécifications minimales
- **Phoenix** programme `PHNXsHsS9GK1E58uQJX9m5L7F7mWJ9jH3UX77PG5Fv`: comptes `Market`, `MarketHeader`, `EventQueue`, `Bid/Ask Queue`. Calcul des prix via `price_lots / quote_lot_size`.
- **OpenBook v2** programme `openBk2L3sNjJKuX3sQYboJjQmpaq4Xrk3R9jFJmMq`: comptes `Market`, `RequestQueue`, `EventQueue`, `Bids`, `Asks`. Maintenir mapping `PriceLot` ↔ `f64`.
- **Orca Whirlpools** programme `whirLb7sGJGZveHFkNjEcVuJ39MF4RduCMsZ7M7P2`: formules CLMM `sqrt_price_x64`, `liquidity`, `tick_spacing`. Implémenter `quote_swap(token_in, amount, slippage_bps)` sans placeholder.
- **Raydium CLMM** programme `CLMMvx4u1S6C9G18JNpDutLCRa14Q6gttYwjdJawVcc`: calcul `dy = (liquidity * (sqrt_price_upper - sqrt_price_lower)) / (sqrt_price_upper * sqrt_price_lower)`.

## Bench & contraintes de performance
- Bench `cargo bench -p ingest --bench depth_replay` doit atteindre **≥ 500 updates/s** avec dataset `fixtures/replay/binance_btcusdt_1k.json`.
- Latence mesurée par `ingest::metrics::LatencyRecorder` < 150 ms pour CEX, < 500 ms pour DEX (95e percentile).

## Tests requis
- Tests unitaires pour chaque parser REST/WS vérifiant la signature (vecteurs fournis dans `fixtures/signatures/*.json`).
- Tests d’intégration `tests/cex_depth_consistency.rs`, `tests/dex_swap_parity.rs` comparant aux SDK officiels (écart < 5 bps).
- Test de résilience : désynchronisation volontaire (gap > 5 messages) → resnapshot automatique vérifié.

## Journalisation & monitoring
- Publier logs `tracing` niveau `INFO` pour `snapshot_loaded`, `diff_applied`, `resync_triggered`.
- Alimenter `docs/logs/latency-benchmark.md` avec histogrammes (bin 50 ms) issus de `just replay`.

## Plan de réalisation
1. **SPRINT-002A — Connecteurs CEX REST/WS** : implémenter clients, throttling, signatures et tests.
2. **SPRINT-002B — Reconstructeur L2 CEX** : fusionner snapshots + diffs et publier des vues propres.
3. **SPRINT-002C — Drivers DEX CLOB** : lire Phoenix/OpenBook, préparer la pose d'ordres.
4. **SPRINT-002D — Quoteur CLMM** : calculer des quotes locales Orca/Raydium.

Chaque sprint produit un journal (`docs/logs/sprint-002X.md`) et des tests automatisés. Valide un sprint avant de passer au suivant (pair review).

## Points de contrôle & critères de sortie
- **PC1 :** latence snapshot/diff mesurée (SPRINT-002A) < 200 ms (consignée).
- **PC2 :** `apply_diff` restitue un carnet cohérent sur 10 000 messages (SPRINT-002B) — test de replay.
- **PC3 :** `subscribe_orderbook` Phoenix/OpenBook renvoie au moins 5 updates/min sans erreur.
- **PC4 :** `quote_swap` Orca/Raydium correspond aux SDK officiels ±1% (5 bps recommandé).
- **PC5 :** Document `docs/diagrams/ingestion-overview.png` mis à jour montrant les flux.

## Risques & mitigations
- **WS instables** : backoff exponentiel + alertes (liées à SPRINT-004B).
- **Divergence d'ID** : tester `OutOfSync` et déclencher resnapshot automatique.
- **Latence CLMM** : caches mémoire, réduire la taille des structures.

## Interactions & dépendances
- EPIC-001 fournit `just replay` et la CI.
- EPIC-003 consomme `OrderBookView` pour la décision d'exécution.
- EPIC-004 lit les métriques `metrics/ingest.prom`.

---

✅ `cargo build --release` (Rust **1.90**), **0 warnings**: `cargo clippy -D warnings`.
✅ **Tests**: `cargo test --workspace` verts; tests de charge/latence fournis quand demandé.
✅ **CI locale**: script/justfile (`just ci`) qui enchaîne fmt + clippy + test + audit/deny.
✅ **Aucun** `todo!()`, `unimplemented!()`, `panic!()` ou commentaires “à faire plus tard”.
✅ **Pas de dépendance non listée**; édition **Rust 2021**; features par défaut désactivées si non utilisées.
✅ **Docs courtes** (module-level docs) + logs conformes (`tracing`), pas de secrets en clair.
