# SPRINT-001A — Installer la toolchain & les dépendances système `[P0]`

> **Persona visée** : dev junior sans expérience système. Suis chaque étape dans l'ordre sans sauter de case.
> **Priorité** : **P0** — sprint bloquant : sans toolchain validée, impossible de démarrer les autres travaux.

## Résultat attendu à la fin du sprint
- Poste macOS Apple Silicon prêt pour compiler des projets Rust 1.90 optimisés (toolchain + composants + cibles).
- Outils systèmes installés et testés individuellement (`protoc`, `llvm-profdata`, `cmake`).
- Solana CLI v1.18.16+ disponible avec l'autocomplétion.
- Trousseau macOS nommé `OtterSlice` créé pour les secrets.
- Journal de validation rempli dans `docs/logs/sprint-001A.md` (à créer) contenant captures de commandes & commentaires.

## Préparation (à faire avant les commandes)
1. Créer un dossier `~/OtterSlice-notes/` et un fichier `sprint-001A-checklist.md` où tu colleras chaque sortie de commande.
2. Mettre à jour macOS (menu pomme → Réglages Système → Général → Mise à jour de logiciels). Redémarre si demandé.
3. Désactiver les restrictions proxy/VPN qui bloqueraient `curl` ou `brew` le temps du sprint.

## Étapes détaillées (ne pas condenser, tout exécuter)
1. **Installer Homebrew**
   - Ouvre l'app Terminal (Spotlight → "Terminal").
   - Colle la commande suivante :
     ```bash
     /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
     ```
   - À la fin, suis les instructions affichées (copier/coller les lignes `echo` vers `.zprofile` ou `.zshrc`).
   - Vérifie : `brew --version` doit afficher `Homebrew 4.x`. Si ce n'est pas le cas, relance le terminal et recommence la vérification.
2. **Installer Rustup + toolchain 1.90.0**
   - Commande :
     ```bash
     curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.90.0
     ```
   - Quand l'installateur termine, exécute `source "$HOME/.cargo/env"`.
   - Vérifie : `rustc --version` et `cargo --version` doivent afficher `1.90.0`.
3. **Installer les composants Rust nécessaires**
   - Commande unique :
     ```bash
     rustup component add rustfmt clippy llvm-tools-preview --toolchain 1.90.0
     ```
   - Vérifie qu'aucune erreur n'est signalée (sortie "installed successfully").
4. **S'assurer que la cible Apple Silicon est disponible**
   - `rustup target list --installed` doit contenir `aarch64-apple-darwin`.
   - Si absent, exécute `rustup target add aarch64-apple-darwin --toolchain 1.90.0`.
5. **Installer les dépendances système via Homebrew**
   - Commande :
     ```bash
     brew install protobuf cmake llvm coreutils jq pkg-config
     ```
   - Vérifications :
     - `protoc --version` → `libprotoc 3.21+`.
     - `cmake --version` → `cmake version 3.27+`.
     - `$(brew --prefix llvm)/bin/llvm-profdata --version` → note la version.
     - Ajoute `export PATH="$(brew --prefix llvm)/bin:$PATH"` dans `~/.zshrc` si `llvm-profdata` n'est pas trouvé.
6. **Installer Solana CLI v1.18.16 ou supérieur**
   - Commande :
     ```bash
     sh -c "$(curl -sSfL https://release.solana.com/v1.18.16/install)"
     ```
   - Ajoute la ligne suivante dans `~/.zshrc` (remplace `<USERNAME>` si nécessaire) :
     ```bash
     export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
     ```
   - Recharge le terminal (`source ~/.zshrc`) puis vérifie :
     ```bash
     solana --version
     solana config get
     ```
     La première commande doit afficher `solana-cli 1.18.16` (ou +). La seconde doit fonctionner sans erreur.
7. **Installer Node.js (utilitaire)**
   - Commande : `brew install node`.
   - Vérifie : `node --version` ≥ 20.x et `npm --version` ≥ 10.x.
8. **Configurer la complétion shell et rust-analyzer**
   - Installe la complétion cargo :
     ```bash
     rustup component add rust-analyzer --toolchain 1.90.0
     mkdir -p ~/.zfunc
     rustup completions zsh cargo > ~/.zfunc/_cargo
     rustup completions zsh rustup > ~/.zfunc/_rustup
     ```
   - Ajoute dans `~/.zshrc` :
     ```bash
     fpath=($HOME/.zfunc $fpath)
     autoload -Uz compinit && compinit
     ```
9. **Créer le trousseau Keychain dédié**
   - Ouvre `Applications > Utilitaires > Trousseau d'accès`.
   - Menu `Fichier > Nouveau trousseau…` → nomme-le **OtterSlice** → définis un mot de passe et note-le dans `sprint-001A-checklist.md` (ne le stocke pas dans un dépôt git).
   - Crée un élément test : clic droit dans le trousseau → "Nouvel élément de mot de passe" → Nom `otterslice-test` / Compte `demo` / Mot de passe `change_me`. Vérifie qu'il apparaît.
10. **Configurer le fichier de log du sprint**
    - Dans le dépôt, crée le dossier `docs/logs/` s'il n'existe pas : `mkdir -p docs/logs`.
    - Crée `docs/logs/sprint-001A.md` avec :
      ```markdown
      # Sprint 001A — Journal d'installation
      - Date : <JJ/MM/AAAA>
      - Opérateur : <Ton nom>

      ## Preuves
      ```
    - Ajoute dans la section "Preuves" des blocs de code avec les sorties de chaque commande de vérification.

## Tests & critères d'acceptation
- ✅ `rustc --version` et `cargo --version` retournent `1.90.0 (stable)`.
- ✅ `rustup show` liste `Default toolchain 1.90.0 (aarch64-apple-darwin)`.
- ✅ `protoc --version`, `cmake --version`, `llvm-profdata --version`, `node --version`, `solana --version` exécutés sans erreur (captures stockées dans `docs/logs/sprint-001A.md`).
- ✅ Capture d'écran (ou export PDF) du trousseau "OtterSlice" jointe dans le log (sauvegarde le fichier sous `docs/logs/otterslice-keychain.png`).
- ✅ Fichier `docs/logs/sprint-001A.md` commité avec toutes les preuves.

## Dépendances / Chemin critique
- Aucune dépendance en amont : ce sprint ouvre le projet.
- Livrable obligatoire avant SPRINT-001B. Sans les preuves de test, blocage automatique.

## Risques & actions de secours
- **Erreur "command not found"** après l'installation : assure-toi d'avoir relancé le terminal ou exécuté `source ~/.zshrc`.
- **Solana CLI lente** : ajoute `export SOLANA_INSTALL_UPDATE_MANIFEST_SKIP=1` dans `~/.zshrc` pour éviter les mises à jour automatiques.
- **Téléchargements bloqués** : connecte-toi à un hotspot 4G temporaire et relance l'étape concernée.
