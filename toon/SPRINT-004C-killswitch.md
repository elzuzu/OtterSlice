# SPRINT-004C — Kill-switch & gestion des incidents

> **But :** pouvoir arrêter immédiatement le bot en cas d'incident et documenter les étapes de reprise.

## Pré-requis
- SPRINT-004A (pre-trade) et SPRINT-004B (monitoring) terminés.
- Runbook kill-switch existant (`docs/runbooks/killswitch.md`) à compléter.

## Livrables
1. Module `crates/risk/src/kill_switch.rs` exposant :
   - `pub struct KillSwitch { flag_path: PathBuf, notifier: broadcast::Sender<KillEvent> }`
   - `pub fn arm(&self) -> Result<()>`, `pub fn disarm(&self) -> Result<()>`, `pub fn is_active(&self) -> bool`
   - `pub async fn watch(self) -> Result<()>` surveillant un fichier `state/kill_switch.json`.
2. Commande CLI `cargo run -p cli -- --mode kill-switch --action arm --reason "RPC degraded"`.
3. Tests unitaires pour vérifier l'armement/désarmement et la notification des consommateurs.
4. Mise à jour du runbook `docs/runbooks/killswitch.md` avec une checklist minute par minute.

## Étapes guidées
1. **Créer la structure**
   - `KillSwitch` stocke le chemin du fichier et un `broadcast::Sender<KillEvent>`.
   - `KillEvent` contient `timestamp`, `reason`, `actor`.
2. **Armement**
   - `arm()` écrit `{"active":true,"reason":"...","timestamp":"..."}` dans `state/kill_switch.json`.
   - `disarm()` écrit `{"active":false}`.
   - Utilise `serde_json` + `fs::write` avec `sync_all()` pour garantir la persistance.
3. **Watcher**
   - Utilise `notify::recommended_watcher` pour détecter les modifications de fichier.
   - À chaque changement, relit le JSON et diffuse l'évènement (`notifier.send(event)`).
4. **Intégration**
   - Dans `pretrade::evaluate`, consulte `KillSwitch::is_active()`.
   - Dans `engine::scanner`, arrête la boucle si un `KillEvent` est reçu.
5. **CLI**
   - Ajoute un sous-commande `kill-switch` avec options `--action arm|disarm|status`.
   - Lors d'un `arm`, demande une confirmation (`tape "ARM" pour confirmer`).
   - Affiche l'état actuel (`status` lit le fichier JSON et imprime). Ajoute option `--follow` pour rester en attente des évènements.
6. **Tests**
   - `test_arm_disarm`: utilise un dossier temporaire, arme, vérifie `is_active`, désarme.
   - `test_watch_notifies`: modifie manuellement le fichier et vérifie qu'un évènement est reçu.
7. **Runbook**
   - Complète `docs/runbooks/killswitch.md` :
     - Minute 0 : armer via CLI.
     - Minute 1 : vérifier `monitoring` (alerte slack).
     - Minute 5 : notifier l'équipe.
     - Minute 10 : démarrer procédure de reprise (checklist).
8. **Journal**
   - `docs/logs/sprint-004C.md` : sorties tests, capture CLI, photo du runbook mis à jour.

## Critères d'acceptation
- ✅ `cargo test -p risk kill_switch` passe.
- ✅ CLI `kill-switch --action arm` crée le fichier JSON avec la raison.
- ✅ Un composant abonné reçoit le broadcast en < 1s.
- ✅ Runbook mis à jour et journal complété.

## Dépendances
- Impacte tous les modules (pre-trade, scanner, executors).
- Nécessaire avant de passer en paper trading (SPRINT-005A).

## Points d'attention
- Gère les erreurs disque (si `fs::write` échoue, loggue `error` et retourne `Result::Err`).
- Pour éviter les races, mets le `KillSwitch` dans un `Arc` partagé.
- Ajoute une option `--reason-file` pour charger un message long depuis un fichier (permet copier/coller du runbook).
