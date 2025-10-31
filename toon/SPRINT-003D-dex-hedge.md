# SPRINT-003D — Exécution DEX (hedge CLOB + CLMM) `[P0]`

> **But :** envoyer les ordres de couverture côté DEX immédiatement après l'exécution CEX pour neutraliser l'exposition.
> **Priorité** : **P0** — inclut ComputeBudget + priority fees (sans Jito) et hedge même venue CEX pour sécuriser la position.

## Pré-requis
- SPRINT-002C (CLOB), SPRINT-002D (CLMM) et SPRINT-003B (scanner) validés.
- Compte Solana devnet avec fonds (`solana airdrop 10` deux fois si besoin).

## Livrables
1. Module `crates/exec/src/dex.rs` exposant :
   - `pub enum DexVenue { Phoenix, OpenBook, Orca, Raydium }`
   - `pub struct HedgeRequest { venue: DexVenue, market: MarketInfo, side: Side, base_amount: f64, limit_price: f64 }`
   - `pub async fn execute(request: HedgeRequest) -> Result<HedgeReport>`
2. Gestion du `ComputeBudget` et de la priorité des transactions.
3. Tests d'intégration sur devnet (flag `#[ignore]`).
4. CLI `cargo run -p cli -- --mode dry-run-dex --venue phoenix --market SOL/USDC --side sell --size 5`.

## Étapes guidées
1. **Préparer les dépendances**
   - Dans `Cargo.toml` de `exec`, ajoute `solana-client`, `solana-sdk`, `spl-token`, `spl-associated-token-account`.
   - Réutilise les helpers du sprint 002C/002D (imports via `crate::dex_clob`, `crate::dex_clmm`).
2. **Construire les transactions**
   - Phoenix/OpenBook : assemble un `Transaction` avec l'instruction `place_order`. Ajoute `ComputeBudgetInstruction::set_compute_unit_limit(400_000)`.
   - Orca/Raydium : appelle la fonction `swap_exact_in` générée depuis l'IDL.
   - Ajoute `recent_blockhash` via `rpc_client.get_latest_blockhash()`.
3. **Signer**
   - Utilise la clé `~/.config/otterslice/dev.json` (charge via `read_keypair_file`).
   - Ajoute la possibilité d'injecter un signer custom (tests).
4. **Envoyer & confirmer**
   - Utilise `rpc_client.send_and_confirm_transaction_with_spinner_and_config` avec `CommitmentConfig::confirmed()`.
   - Si la transaction échoue, récupère les logs (`simulate_transaction`) et remonte une erreur lisible.
5. **HedgeReport**
   - Champs : `signature`, `status` (`Submitted`, `Confirmed`, `Finalized`, `Failed`), `slot`, `fills_estimated`, `fees_paid` (en lamports), `latency_ms`.
6. **Tests**
   - `tests/dry_run.rs` : simule la génération d'une transaction et vérifie que les comptes requis sont présents.
   - `tests/devnet.rs` (ignored) : envoie un swap 0.1 SOL → USDC et vérifie que `status == Confirmed`.
7. **CLI**
   - Mode `dry-run-dex` affiche la transaction base64 et les instructions.
   - Option `--send` pour exécuter (devnet). Ajoute `--priority-fee 5000`.
   - Consigne la signature dans `docs/logs/sprint-003D.md`.
8. **Coordination avec CEX**
   - Ajoute une fonction `pub async fn hedge_after_cex(report: ExecutionReport, hedge_req: HedgeRequest)` qui attend `report.status == Filled` avant d'envoyer la couverture.
   - Gère le cas partiellement rempli : ajuster `base_amount` à `report.executed_qty`.

## Critères d'acceptation
- ✅ `cargo test -p exec dex` passe (hors tests `#[ignore]`).
- ✅ Mode CLI `dry-run-dex` produit une transaction avec `ComputeBudget` en première instruction.
- ✅ Test devnet (manuel) confirme une transaction.
- ✅ Journal rempli avec signature + logs.

## Dépendances
- Reçoit l'opportunité du scanner et le rapport CEX (SPRINT-003C).
- Sera utilisé par SPRINT-004A (pre-trade risk) et SPRINT-004B (monitoring).

## Points d'attention
- Vérifie la disponibilité des comptes token : crée-les si besoin via `spl_associated_token_account::create_associated_token_account`.
- Sur Orca, certaines transactions requièrent `tick_array` : précharge-les.
- Ajoute un mode `simulate` qui appelle `simulate_transaction` avant envoi réel pour obtenir le gas estimé.
