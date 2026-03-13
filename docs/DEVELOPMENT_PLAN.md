# Plan de développement – Apple Code Assistant (Rust)

CLI inspirée de [Apple-AI-CLI](https://github.com/0xatrilla/Apple-AI-CLI) : génération de code via Apple Intelligence, TUI, streaming, multi-langages, gestion des conversations.

**État actuel** : squelette compilable (parser clap, handler stub, modules `api`, `config`, `ui`, `utils`, `types`).

---

## Vue d’ensemble des objectifs

| Objectif | Description |
|----------|-------------|
| **Apple Intelligence** | Intégration on-device des modèles Apple Foundation |
| **TUI** | Interface terminal (ASCII art, panneaux, thème) |
| **Conversations** | Historique et sessions persistantes |
| **Multi-langages** | Génération pour 25+ langages (typescript, python, rust, etc.) |
| **Fichiers** | Sauvegarde, édition, sortie vers fichier |
| **Syntax highlighting** | Affichage du code généré (style highlight.js) |
| **Presse-papiers** | Copie du code dans le clipboard macOS |
| **CLI** | Parsing robuste (déjà en place avec clap) |
| **Gestion d’erreurs** | Erreurs structurées et fallbacks propres |

---

## Phase 0 – Déjà fait (skeleton)

- [x] Projet Cargo, `clap` + `anyhow`
- [x] Structure `src/{cli,api,config,ui,utils,types}`
- [x] Parser CLI (options + sous-commandes `config`, `models`, `languages`, `test`)
- [x] Handler stub (délégation sous-commandes / mode interactif / mode direct)
- [x] `.gitignore`, `env.example`, README

---

## Phase 1 – Config, logging et erreurs

**But** : configuration chargée (env + fichier), logging selon `--verbose`/`--debug`, erreurs explicites et sortie propre.

1. **Configuration**
   - Créer `config::Config` avec champs : `model`, `default_language`, `theme`, `max_tokens`, `temperature`, `config_file`.
   - Charger depuis `.env` (crate `dotenvy`) et/ou fichier config (ex. `~/.config/apple-code-assistant/config.toml` ou env uniquement).
   - Utiliser `config::load()` au démarrage et passer `Config` au handler.

2. **Logging**
   - Dans `utils`, initialiser un logger (ex. `env_logger` ou `tracing`) selon `args.verbose` / `args.debug` et `config`.
   - Appeler `utils::setup_logging(verbose, debug)` dans `main` après parse.

3. **Gestion d’erreurs**
   - Introduire des types d’erreur dédiés (crate `thiserror`) : `ConfigError`, `ApiError`, `IoError`, etc.
   - Utiliser `anyhow` en bordure (main, handler) et `Result<T, E>` typé en interne.
   - Messages clairs et sortie propre (pas de panic en prod).

4. **Sous-commandes `config`**
   - Implémenter `config --list`, `--get <key>`, `--set key=value`, `--reset` (lecture/écriture du fichier config ou env).

**Livrables** : config chargée, logs contrôlables, sous-commande `config` fonctionnelle, erreurs structurées.

---

## Phase 2 – Terminal UI (branding et affichage)

**But** : interface terminal soignée (ASCII art, couleurs, thème) et affichage du code en mode direct.

1. **ASCII art et branding**
   - Afficher le logo “APPLE CODE” (ASCII art) au démarrage du mode interactif (et optionnellement en mode direct).
   - Utiliser `figlet`-like en Rust (crate `figlet_rs` ou fichier art statique) et couleurs (crate `owo_colors` ou `colored`).

2. **Thème**
   - Respecter `--theme` / `config.theme` (light/dark) pour couleurs du terminal (titres, code, bordures).

3. **Affichage du code (preview)**
   - En mode direct, si `--preview` ou pas de `--output`, afficher le code généré dans le terminal avec bordures et titres (sans TUI complète pour l’instant).

4. **Dépendances**
   - Ajouter `owo_colors` (ou `colored`), éventuellement `figlet_rs` ou art statique.

**Livrables** : logo au lancement, thème appliqué, preview du code en terminal.

---

## Phase 3 – Intégration Apple Intelligence (client API)

**But** : brancher le flux de génération de code à un “client” Apple Foundation (ou mock pour dev).

1. **Modèle d’API**
   - Définir dans `api` une interface (trait) du type :  
     `generate(&self, prompt, language, options) -> Result<StreamOrResponse>`.
   - Types dans `types` : `GenerateRequest`, `GenerateResponse` ou stream de chunks.

2. **Implémentation réelle ou mock**
   - **Option A (macOS)** : appels vers l’API on-device Apple (Foundation Models) si disponible (FFI, CLI système, ou doc Apple).
   - **Option B (mock)** : client mock qui retourne du code statique ou un délai + snippet, pour développer le reste (TUI, streaming, fichiers) sans dépendance Apple.

3. **Options de génération**
   - Utiliser `args` + `config` : `temperature`, `max_tokens`, `model`, `context`.

**Livrables** : trait + impl (réelle ou mock), génération déclenchée depuis le handler avec options.

---

## Phase 4 – Streaming et mode interactif (TUI)

**But** : réponses en streaming et mode interactif type REPL avec historique.

1. **Streaming**
   - Si l’API Apple (ou le mock) renvoie un stream, afficher les chunks au fur et à mesure dans le terminal (sans effacer la ligne de prompt).
   - Utiliser `crossterm` ou `ratatui` pour mise à jour du texte (zone dédiée “réponse”).

2. **Mode interactif (REPL)**
   - Boucle readline : prompt `> `, lecture de la ligne.
   - Commandes internes : `/help`, `/exit`, `/clear`, `/history`, `/sessions`, `/models`, `/languages`, `/test`.
   - Pour chaque entrée utilisateur (hors commande) : appeler le client de génération et afficher la réponse (streaming si dispo).

3. **Dépendances**
   - `crossterm` et/ou `ratatui`, `rustyline` ou `linefeed` pour la ligne de commande interactive.

**Livrables** : streaming visible en terminal, REPL avec commandes `/help`, `/exit`, etc.

---

## Phase 5 – Gestion des conversations (sessions et historique)

**But** : persistance de l’historique et des sessions (comme dans Apple-AI-CLI).

1. **Modèle de données**
   - Types `Conversation`, `Message` (role: user/assistant, content), `Session` (id, messages, metadata).
   - Stockage : répertoire config (ex. `~/.config/apple-code-assistant/sessions/`) avec un fichier par session (JSON ou SQLite léger).

2. **ConversationManager**
   - Créer/charger session, ajouter message, sauvegarder après chaque échange.
   - En mode interactif : session “courante” avec contexte des N derniers messages pour les appels API (si l’API le supporte).

3. **Commandes**
   - `/history` : afficher les messages de la session courante.
   - `/sessions` : lister les sessions sauvegardées ; option pour charger une session.

**Livrables** : sessions persistantes, `/history` et `/sessions` opérationnels.

---

## Phase 6 – Opérations fichiers et presse-papiers

**But** : sauvegarde, édition et copie du code généré.

1. **Fichiers**
   - **Save** : avec `--save` et `-o/--output`, écrire le code généré dans le fichier (création ou écrasement après confirmation si besoin).
   - **Edit** : avec `-e/--edit <file>`, envoyer le contenu du fichier + prompt comme contexte, remplacer la sortie dans le fichier (ou afficher un diff avant d’écrire).
   - Utiliser `utils::file_operations` (ou module dédié) et `Config` pour répertoire par défaut si pertinent.

2. **Presse-papiers (macOS)**
   - Avec `--copy`, copier le code généré dans le clipboard (crate `clipboard` ou `arboard`).
   - Gérer l’erreur si non-macOS ou outil absent (message clair).

**Livrables** : `--save -o file`, `--edit file`, `--copy` fonctionnels.

---

## Phase 7 – Syntax highlighting

**But** : affichage du code généré avec couleurs par langage (équivalent highlight.js).

1. **Choix de la crate**
   - Utiliser `syntect` (basé sur Sublime) ou `tree-sitter` + thèmes pour coloriser le code.

2. **Intégration**
   - Détecter le langage depuis `--language` ou le prompt (utils `language_detector`).
   - En preview / mode interactif : afficher le code passé par le highlighter avant d’écrire dans le terminal (avec support ANSI).

**Livrables** : code affiché avec couleurs selon le langage (terminal et preview).

---

## Phase 8 – Multi-langages et polish

**But** : support explicite de 25+ langages et finition.

1. **Liste des langages**
   - Fichier ou constante : liste des langages supportés (typescript, javascript, python, rust, go, java, cpp, swift, etc.) avec alias (ts → typescript).
   - Sous-commande `languages` : afficher cette liste.

2. **Détection et validation**
   - `utils::language_detector` : proposer un langage par défaut à partir du prompt ou de l’extension du fichier (pour `--edit`).
   - Valider `--language` par rapport à la liste et afficher une erreur claire si invalide.

3. **Sous-commandes `models` et `test`**
   - `models` : lister les modèles Apple Foundation disponibles (ou “mock” / config).
   - `test` : ping ou appel minimal vers l’API pour vérifier la connexion / disponibilité.

4. **Polish**
   - Gestion SIGINT/SIGTERM (message “Operation cancelled”, sortie propre).
   - Vérifier tous les chemins d’erreur (fichier illisible, API indisponible, etc.).
   - README et `--help` à jour avec exemples.

**Livrables** : `languages` et `models`/`test` utiles, détection de langage, gestion des signaux et erreurs, doc à jour.

---

## Ordre recommandé et dépendances

```text
Phase 0 (fait) → Phase 1 (config, logs, erreurs)
       → Phase 2 (TUI branding, preview)
       → Phase 3 (API / mock)
       → Phase 4 (streaming, REPL)
       → Phase 5 (conversations)
       → Phase 6 (fichiers, clipboard)
       → Phase 7 (syntax highlighting)
       → Phase 8 (multi-langages, polish)
```

- Les phases 1 et 2 peuvent être en grande partie parallélisables une fois les interfaces définies.
- La phase 3 (API/mock) est le prérequis pour 4, 5, 6.
- Les phases 6 et 7 améliorent l’UX une fois le flux de génération en place (3+4).

---

## Dépendances Cargo à ajouter (au fil des phases)

| Phase | Crates suggérées |
|-------|-------------------|
| 1 | `dotenvy`, `serde`, `toml` ou équivalent, `thiserror`, `env_logger` ou `tracing` |
| 2 | `owo_colors` ou `colored`, optionnel `figlet_rs` |
| 3 | (selon API Apple) ou rien pour mock |
| 4 | `crossterm`, `ratatui`, `rustyline` ou `linefeed` |
| 5 | `serde_json`, `uuid` ou ids simples |
| 6 | `arboard` ou `clipboard` |
| 7 | `syntect` (ou `bat` pour thèmes) |

---

## Références

- [Apple-AI-CLI](https://github.com/0xatrilla/Apple-AI-CLI) – référence fonctionnelle et README.
- [README Apple-AI-CLI](https://raw.githubusercontent.com/0xatrilla/Apple-AI-CLI/refs/heads/main/README.md) – fonctionnalités et commandes.

Ce document peut servir de base pour des issues ou des PRs (une par phase ou par sous-étape).
