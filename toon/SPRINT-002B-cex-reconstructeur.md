# SPRINT-002B — Reconstructeur d'order book CEX (L2) `[P0]`

> **Tu construis ici la brique qui transforme les flux REST/WS en carnet cohérent. Respecte chaque étape, sinon l'algorithme dérivera.**
> **Priorité** : **P0** — essentiel pour livrer un carnet CEX cohérent, base du scanner et des contrôles pre-trade.

## Pré-requis
- SPRINT-002A validé (les connecteurs REST/WS renvoient des snapshots et des diffs).
- Avoir lu la documentation Binance sur la reconstruction L2 : https://binance-docs.github.io/apidocs/spot/en/#diff-depth-stream
- Journal `docs/logs/sprint-002B.md` créé avec sections `Commandes`, `Tests automatisés`, `Tests manuels`.

## Livrables
1. Module `crates/ingest/src/cex_orderbook.rs` exposant :
   - `struct CexOrderBook { bids: BTreeMap<f64, f64>, asks: BTreeMap<f64, f64>, last_update_id: u64 }`
   - `impl CexOrderBook { fn apply_snapshot(...); fn apply_diff(...); fn best_bid(); fn best_ask(); }`
2. Tâche asynchrone `async fn run_cex_orderbook(exchange: Exchange, symbol: &str) -> Result<()>` qui consomme les flux du sprint 002A et publie les updates via `tokio::sync::watch::Sender<OrderBookView>`.
3. Tests unitaires et tests d'intégration validant :
   - Application correcte des `u` (update IDs) Binance.
   - Cas de rattrapage (si `first_update_id > last_update_id + 1`, on redemande un snapshot).
4. Documentation interne : diagramme de séquence `docs/diagrams/cex-reconstructor.plantuml`.

## Étapes guidées
1. **Initialiser le module**
   - Crée `crates/ingest/src/lib.rs` si absent, re-exporte `pub mod cex_orderbook;`.
   - Dans `Cargo.toml` de `ingest`, ajoute les dépendances `tokio`, `tracing`, `ahash`, `serde`, `anyhow`, `thiserror` via `workspace`.
2. **Définir la structure de carnet**
   - Utilise `BTreeMap<f64, f64>` pour conserver l'ordre de prix.
   - Ajoute une méthode `fn depth(&self, side: Side, depth: usize) -> Vec<OrderBookLevel>` qui retourne les `depth` premiers niveaux.
   - Documente chaque méthode avec `///` en précisant ce qu'elle retourne.
3. **Implémenter `apply_snapshot`**
   - Paramètres : `&mut self`, `RestSnapshot`.
   - Vide les maps actuelles (`self.bids.clear()` etc.) puis insère chaque niveau (ignore ceux avec quantité `0.0`).
   - Positionne `self.last_update_id = snapshot.last_update_id`.
4. **Implémenter `apply_diff`**
   - Paramètres : `&mut self`, `WsDiffEvent`.
   - Vérifie la condition `event.first_update_id <= self.last_update_id + 1 && event.final_update_id >= self.last_update_id + 1` avant d'appliquer (sinon ignore le diff et retourne `DiffOutcome::OutOfSync`).
   - Pour chaque niveau :
     - Si quantité == 0 → `remove` la clé.
     - Sinon → `insert` la nouvelle quantité.
   - Mets à jour `self.last_update_id = event.final_update_id`.
5. **Gestion du rattrapage**
   - Crée une énumération `enum DiffOutcome { Applied, Ignored, OutOfSync }`.
   - Lorsque tu reçois `OutOfSync`, redemande un snapshot via le client REST (utilise `retry` 3 fois maximum). Utilise `tokio::time::sleep(Duration::from_millis(250))` entre chaque essai.
6. **Publier les vues**
   - Crée `struct OrderBookView { spread_bps: f64, best_bid: f64, best_ask: f64, mid: f64, timestamp: DateTime<Utc> }` (dépendance `chrono`).
   - Après chaque snapshot et chaque diff appliqué, envoie `watch_sender.send(view.clone())?`.
   - Mets un compteur `metrics::increment_counter!("cex_orderbook_updates", "exchange" => exchange.as_str())` (prépare le module metrics même si encore stub).
7. **Tests**
   - `tests/binance_replay.rs` :
     - Charge un fichier JSON `fixtures/binance_sol_depth.json` contenant un snapshot + une série de diffs.
     - Applique `apply_snapshot`, puis boucle sur les diffs et vérifie que `best_bid` évolue selon les valeurs attendues.
   - `tests/out_of_sync.rs` :
     - Simule un diff avec `first_update_id` trop grand → vérifie que `DiffOutcome::OutOfSync` est retourné.
   - Utilise `#[tokio::test]` pour tester le flux complet (mocke le client REST/WS via `async_channel`).
   - Ajoute `fixtures/` sous `crates/ingest/` et documente leur provenance dans un README (sources officielles).
8. **Diagramme de séquence**
   - Crée `docs/diagrams/` si nécessaire.
   - Fichier `cex-reconstructor.plantuml` :
     ```plantuml
     @startuml
     participant CLI
     participant CexClient
     participant CexOrderBook

     CLI -> CexClient : fetch_snapshot()
     CexClient --> CLI : RestSnapshot
     CLI -> CexOrderBook : apply_snapshot()
     loop WS diffs
       CexClient --> CLI : WsDiffEvent
       CLI -> CexOrderBook : apply_diff()
       CexOrderBook -> CLI : OrderBookView
     end
     @enduml
     ```
   - Génère le diagramme via `plantuml docs/diagrams/cex-reconstructor.plantuml` (si PlantUML installé) et stocke l'image PNG.
9. **Validation manuelle**
   - Ajoute dans la CLI un mode `--mode watch-cex` affichant `best_bid/best_ask` toutes les 500 ms.
   - Lancer la commande pendant 30 secondes et vérifier que les valeurs évoluent sans erreur.
   - Colle 10 lignes de sortie dans `docs/logs/sprint-002B.md`.

## Critères d'acceptation
- ✅ Tests unitaires/integration `cargo test -p ingest` verts (joins la sortie dans le journal).
- ✅ Mode CLI `watch-cex` affichant un spread logique (best_bid < best_ask, spread_bps > 0).
- ✅ Diagramme PlantUML commité (fichier `.puml` + `.png`).
- ✅ Log d'exécution démontrant un rattrapage automatique (note l'horodatage dans le journal).

## Dépendances
- Consomme les flux du sprint 002A.
- Fournit le flux normalisé au sprint 003B (scanner spreads).

## Points d'attention
- Les prix/quantités doivent être arrondis à 1e-9 pour éviter le bruit (utilise `((value * 1e9).round() / 1e9)`).
- N'applique pas un diff si `event.final_update_id < self.last_update_id` (cas "ancienne mise à jour"). Loggue l'évènement.
- Documente chaque fonction publique avec un exemple minimal (doctest) pour aider le junior qui lira plus tard.
