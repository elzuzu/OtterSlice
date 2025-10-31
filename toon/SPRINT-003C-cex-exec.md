# SPRINT-003C — Exécution CEX (Binance/OKX/Bybit) `[P0]`

> **But :** transformer une opportunité en ordres effectifs côté CEX en respectant les contraintes de risque et de latence.
> **Priorité** : **P0** — obligations : ordres IOC/FAK, slicing p95, retry/backoff, latence mesurée et fiable.

## Pré-requis
- SPRINT-002A (connecteurs) et SPRINT-003B (scanner) terminés.
- Clés API actives avec permission "trade" (pas de withdraw).
- Compte test ou sous-compte avec fonds démo.

## Livrables
1. Module `crates/exec/src/cex.rs` exposant :
   - `pub struct ExecutionRequest { exchange: Exchange, symbol: String, side: Side, quantity: f64, price: f64, post_only: bool }`
   - `pub async fn execute(request: ExecutionRequest) -> Result<ExecutionReport>`
   - Gestion des ordres `IOC`/`FOK` + cancel on timeout.
2. Implémentation de la signature HMAC + header spécifique pour chaque exchange.
3. Tests d'intégration "paper" : simulateur qui répond avec fills partiels.
4. CLI `cargo run -p cli -- --mode dry-run-cex --exchange binance --symbol SOLUSDT --side buy --qty 10`.

## Étapes guidées
1. **Compléter les structures**
   - `ExecutionReport` doit inclure : `order_id`, `status` (`Filled`, `PartiallyFilled`, `Cancelled`, `Rejected`), `executed_qty`, `avg_price`, `fills` (Vec avec `qty`, `price`, `fee`), `latency_ms`.
2. **Gérer les particularités API**
   - Binance : endpoint `POST /api/v3/order` (paramètres `type=LIMIT_MAKER` pour post-only, `timeInForce=IOC` sinon). Ajoute `timestamp` + `recvWindow` 5000.
   - OKX : endpoint `POST /api/v5/trade/order` (payload JSON). Inclure `OK-ACCESS-PASSPHRASE`.
   - Bybit : endpoint `POST /v5/order/create`. Gère le `category=spot`.
3. **Signer les requêtes**
   - Crée un helper `fn sign_binance(params: &HashMap<&str, String>, secret: &str) -> String`.
   - Pour OKX/Bybit, signature basée sur `timestamp + method + path + body`.
   - Ajoute des tests unitaires avec exemples officiels (copie les valeurs depuis la doc pour vérifier que le hash correspond).
4. **Gestion des timeouts et retries**
   - Utilise `tokio::time::timeout(Duration::from_millis(800), client.send())`.
   - Si timeout, envoie un `cancel`.
   - Réessaie au maximum 2 fois sur erreurs réseau (`reqwest::Error::is_connect()`/`is_timeout()`).
5. **Parse des réponses**
   - Convertis en structures `serde` :
     ```rust
     #[derive(Deserialize)]
     struct BinanceOrderResponse {
         orderId: u64,
         status: String,
         fills: Vec<BinanceFill>,
     }
     ```
   - Mappe `status` vers ton enum.
6. **Enrichir le rapport**
   - Calculer `latency_ms` = `Instant::now() - start`.
   - Agréger `executed_qty` en additionnant les fills.
   - Calcule `avg_price` pondéré.
7. **Tests**
   - `tests/binance_signature.rs` : vérifie la signature.
   - `tests/execution_sim.rs` : utilise `wiremock` pour simuler un fill partiel puis cancel.
   - Ajoute un test `#[ignore]` pour un ordre réel sur testnet Binance (utilise l'URL testnet `https://testnet.binance.vision`).
8. **CLI Dry run**
   - Mode `dry-run-cex` qui affiche la requête signée sans l'envoyer (option `--send` pour exécuter réellement).
   - Ajoute un paramètre `--timeout-ms`.
   - Capture la sortie dans `docs/logs/sprint-003C.md`.

## Critères d'acceptation
- ✅ `cargo test -p exec cex` passe.
- ✅ Mode `dry-run-cex` affiche la signature et la requête HTTP.
- ✅ En mode `--send` (sur testnet), un ordre `IOC` est rempli ou rejeté avec un message clair.
- ✅ Journal complété avec captures.

## Dépendances
- Reçoit les signaux du scanner (SPRINT-003B) et enverra les résultats au module hedge (SPRINT-003D).

## Points d'attention
- Vérifie toujours les limites de poids (Binance `X-MBX-USED-WEIGHT-1M`). Si > 90% loggue `warn`.
- Ne loggue jamais les clés API (masque avec `***`).
- Traite explicitement les codes d'erreur (ex: `-2010` insuffisance de fonds → remonte `ExecutionError::InsufficientBalance`).
