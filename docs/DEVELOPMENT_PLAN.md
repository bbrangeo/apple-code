# Plan de développement – Apple Code Assistant (Rust)

Mis à jour le **17 novembre 2026**, pour la version **0.1.3**, publiée sur crates.io.

CLI de génération de code sur appareil, inspirée de [Apple-AI-CLI](https://github.com/0xatrilla/Apple-AI-CLI). Le projet n'est plus un squelette : génération Apple Intelligence, REPL, fichiers, configuration et sessions sont implémentés. La fiabilité des sorties et certaines options restent à améliorer. **Le streaming n'est pas encore implémenté.**

Les cases cochées correspondent au code livré ; les cases ouvertes sont des travaux restants, pas des fonctionnalités garanties.

## État d'ensemble

| Domaine | État en 0.1.3 |
|---|---|
| Backend Apple Intelligence | Génération complète via `apple_ai` 0.1.1 ; repli vers le mock si l'initialisation échoue |
| Interface terminal | Logo, thèmes CLI, aperçu et coloration syntaxique ; pas de TUI plein écran |
| REPL | Saisie et commandes interactives ; `-l` prioritaire sur la configuration depuis 0.1.3 |
| Langages | 20 noms canoniques et leurs alias ; validation de `-l` en mode direct et interactif |
| Fichiers et presse-papiers | Écriture, édition et copie disponibles ; aucune validation automatique du code généré |
| Conversations | Sessions JSON et reprise ; sélection de la session et taille de l'historique à fiabiliser |
| Entrées et templates | stdin, contexte explicite, globs et templates en mode direct |
| Qualité | 11 tests avec les fonctionnalités par défaut, 10 sans Apple AI ; Clippy strict et formatage validés lors de la préparation de 0.1.3 |
| Distribution | Version 0.1.3 publiée ; exclusions des fichiers locaux corrigées |

## Phase 0 – Socle livré

- [x] Projet Cargo, binaire `apple-code` et bibliothèque `apple_code_assistant`.
- [x] Parsing `clap` et routage des sous-commandes `config`, `models`, `languages`, `test`.
- [x] Modules `api`, `cli`, `config`, `conversation`, `types`, `ui`, `utils`.
- [x] Suppression du code mort et mutualisation des requêtes, de la sérialisation de configuration et du rendu de l'historique.

## Phase 1 – Configuration, logs et erreurs : socle livré

- [x] Chargement des valeurs par défaut, du fichier TOML et des variables d'environnement ; lecture de `.env` via `dotenvy`.
- [x] `--config` pour sélectionner explicitement un fichier ; `config --list/get/set/reset`.
- [x] Templates TOML avec texte système, modèle et température ; erreur si un template explicitement demandé est absent en mode direct.
- [x] Logs via `env_logger` et erreurs typées avec `thiserror`, propagées avec `anyhow` dans le CLI.
- [x] Sérialisation directe de `Config`, exclusion du chemin local et tests de roundtrip/fusion.
- [ ] Unifier les priorités CLI/configuration entre mode direct et REPL : thème, température, maximum de tokens et templates ne sont pas traités uniformément.
- [ ] Valider les bornes des options numériques avant conversion vers les types du backend.
- [ ] Propager les erreurs de lecture stdin et de persistance des sessions au lieu de les ignorer.

Le fichier par défaut dépend de `dirs::config_dir()`. Sur macOS :
`~/Library/Application Support/apple-code-assistant/config.toml`.
Le `config.toml` du checkout n'est pas chargé automatiquement ; utiliser `--config config.toml`.

Source : [configuration](../src/config/mod.rs).

## Phase 2 – Interface terminal : livrée, à fiabiliser pour les pipelines

- [x] Logo statique et couleurs `owo-colors` ; choix CLI `--theme light|dark`.
- [x] Aperçu du code et mode direct `--tool-mode` sans cadre de terminal.
- [x] Nettoyage du premier bloc Markdown complet en mode outil, avec tests de compatibilité.
- [ ] Respecter `--no-color`, les sorties non interactives et la configuration du thème.
- [ ] Définir une séparation cohérente entre résultat sur stdout et messages de statut sur stderr, notamment avec `-o` et `--copy`.

Conserver une interface simple ; n'ajouter une TUI plein écran que pour un besoin identifié.

## Phase 3 – Apple Intelligence : backend livré, limites connues

- [x] Client réel `AppleAiCodeGenClient` et client mock derrière `CodeGenClient`.
- [x] Activation du backend par défaut ; mode mock avec `--no-default-features`.
- [x] Transmission du langage, du contexte et des consignes dans le message utilisateur : les messages système sont ignorés par le chemin de génération simple de `apple_ai` 0.1.1.
- [x] Température et maximum de tokens transmis au backend.
- [x] Message d'erreur JSON enrichi avec une piste de réduction du contexte.
- [ ] Corriger en amont, ou adopter une version corrigée de `apple_ai`, pour éviter le doublonnage du prompt dans le transcript et préserver les erreurs Swift réelles.
- [ ] Définir une politique de budget d'entrée/sortie et signaler explicitement toute troncature.
- [ ] Rendre explicite le choix réel/mock : un succès du mock ne doit pas être confondu avec une génération Apple Intelligence.
- [ ] Implémenter réellement la sélection de modèle, ou clarifier les options `model` actuellement transportées mais non utilisées par le client Apple.

Le modèle peut produire du code invalide ou une réponse non pertinente même avec un code de sortie nul. Réduire `--char-limit` ne permet pas de résumer intégralement un diff plus grand.

Source : [client Apple AI](../src/api/apple_ai_client.rs).

## Phase 4 – REPL livré ; streaming à faire

- [x] REPL `rustyline`, historique de saisie local et sortie par `/exit`, EOF ou interruption.
- [x] Commandes `/help`, `/clear`, `/history`, `/sessions`, `/models`, `/languages`, `/test`.
- [x] `apple-code -l rust` et `apple-code -i -l RS` prioritaires sur la langue configurée, sans modifier la configuration enregistrée.
- [x] Tests des alias, des deux modes interactifs, des valeurs par défaut et des langues invalides.
- [ ] Faire utiliser à `/test` le vrai chemin de vérification ; sa réponse actuelle est statique.
- [ ] Partager les listes de modèles et de langages entre sous-commandes CLI et REPL ; les réponses interactives sont actuellement statiques ou abrégées.
- [ ] Définir quelles options de génération du mode direct sont prises en charge dans le REPL, puis les transmettre ou les refuser explicitement.
- [ ] Ajouter le streaming du backend jusqu'à stdout, avec propagation des erreurs et interruption propre. Aujourd'hui, la réponse complète est attendue avant affichage.

Commencer par l'affichage des chunks dans le terminal ; aucune nouvelle dépendance de TUI n'est requise par principe.

Source : [REPL](../src/ui/repl.rs).

## Phase 5 – Conversations : persistance livrée

- [x] Types `Message`, `Role`, `Session` ; identifiants UUID et fichiers JSON dans le répertoire de configuration.
- [x] Création, chargement et sauvegarde des messages ; `/sessions <id>` et `/history`.
- [x] Reprise en mode direct avec `--extend-conversation` et formatage partagé de l'historique.
- [ ] Choisir la session la plus récente selon sa date : le code prend actuellement le dernier UUID trié lexicalement.
- [ ] Borner l'historique envoyé au modèle et éviter de réinclure inutilement le prompt courant.
- [ ] Définir la persistance des données de contexte : stdin et fichiers ne sont pas sauvegardés avec le prompt utilisateur actuel.
- [ ] Tester la reprise sur plusieurs sessions et les échecs de lecture/écriture.

`/clear` efface l'écran, pas la conversation. Aucune analyse automatique du dépôt n'est déclenchée par `--extend-conversation`.

Source : [gestionnaire de sessions](../src/conversation/manager.rs).

## Phase 6 – Fichiers et presse-papiers : priorité à la sécurité des écritures

- [x] Écriture via `-o`, édition via `-e/--edit`, création des répertoires parents et copie via `arboard`.
- [x] Exemples du README avec langage explicite, mode brut et édition d'une copie plutôt que du fichier source original.
- [x] Vérifications manuelles de fichiers Rust générés et tests fonctionnels de deux recherches binaires Python en VM ; voir le [rapport des exemples](README_EXAMPLES_TEST_REPORT.md).
- [ ] Ne pas traiter le nettoyage Markdown comme une validation de syntaxe ou de comportement.
- [ ] Prévoir une écriture temporaire et une revue du diff avant remplacement du fichier original ; ajouter une validation explicite adaptée au langage lorsque disponible.
- [ ] Clarifier `--save` : actuellement, `-o` déclenche l'écriture et le booléen `--save` n'est pas consulté.
- [ ] Couvrir les sorties tronquées, les erreurs d'écriture et la conservation de l'original en cas d'échec.

## Phase 7 – Coloration syntaxique : livrée

- [x] `syntect`, correspondance langage/extension et thèmes clair/sombre.
- [x] Coloration dans les aperçus et le REPL ; repli vers du texte brut si aucune syntaxe n'est trouvée.
- [ ] Tester les langages effectivement disponibles dans les syntaxes embarquées, l'absence de couleurs et la préservation des fins de ligne.

La liste des langages acceptés par le CLI ne garantit pas qu'une grammaire de coloration correspondante est embarquée.

## Phase 8 – Langages, pipelines et qualité : partiellement livrée

- [x] Liste centralisée de 20 langages/formats et alias, normalisation et rejet des valeurs `-l` inconnues.
- [x] stdin séparé de la consigne, `--context`, `--context-glob` et limite d'entrée en mode direct.
- [x] Gestion de Ctrl+C avec message d'annulation.
- [ ] Réparer `--repeat-input` : ignoré en mode outil ; hors mode outil, le code répète le prompt plutôt que le stdin.
- [ ] Rendre `--char-limit` sûr pour UTF-8 et cohérent avec son nom : le code utilise actuellement une longueur et une coupure en octets, susceptibles de couper un caractère.
- [ ] Décider de la détection de langage depuis un fichier ou un prompt : des fonctions existent, mais ne sont pas branchées au routage actuel.
- [ ] Faire retourner un code non nul à `apple-code test` lorsqu'une génération échoue.
- [ ] Aligner les derniers exemples et promesses du README avec les comportements réels, puis automatiser leurs contrôles déterministes.
- [ ] Ajouter une CI couvrant formatage, Clippy, tests et empaquetage ; fixer une version minimale de Rust après vérification des dépendances.

Source : [handler CLI](../src/cli/handler.rs), [langages](../src/utils/languages.rs).

## Ordre de travail recommandé

1. **Protéger les fichiers** : revue avant remplacement, erreurs d'écriture et code généré invalide.
2. **Fiabiliser le contexte et les erreurs du backend** : doublonnage, UTF-8, budget et troncature explicite.
3. **Aligner CLI et REPL** : priorités de configuration, templates, options inactives et codes de sortie.
4. **Réparer les pipelines et la reprise** : `--repeat-input`, canaux stdout/stderr, choix de session et historique borné.
5. **Automatiser les régressions**, puis ajouter le streaming sans nouvelle couche d'interface inutile.

## Validation et distribution

Contrôles validés lors de la préparation de la version 0.1.3 :

```bash
cargo fmt --check
cargo test --locked
cargo test --locked --no-default-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo doc --locked --no-deps
cargo publish --dry-run --allow-dirty --locked
```

Les tests unitaires ne nécessitent pas de génération réelle ; les essais Apple Intelligence et du presse-papiers restent manuels. Le [rapport des exemples](README_EXAMPLES_TEST_REPORT.md) conserve les observations historiques et leurs limites ; ses anciens échecs Clippy précèdent les corrections désormais livrées.

Pour le backend réel : macOS 26+, Apple Silicon, Apple Intelligence activée et modèle téléchargé, avec Xcode/Command Line Tools compatibles pour compiler le pont Swift. Installation de la version publiée :

```bash
MACOSX_DEPLOYMENT_TARGET=26.0 cargo install apple-code-assistant --version 0.1.3 --force
```

Les fichiers locaux de configuration, l'historique et les fichiers d'éditeur sont exclus du paquet. Lors d'une prochaine publication, vérifier le contenu avec `cargo package --list` ; n'utiliser `--allow-dirty` qu'après contrôle des changements présents.

Dépendances déjà en place : `clap`, `anyhow`, `thiserror`, `serde`, `toml`, `dotenvy`, `env_logger`, `log`, `dirs`, `apple_ai`, `tokio`, `rustyline`, `serde_json`, `uuid`, `arboard`, `syntect`, `owo-colors`, `ctrlc`, `glob`. Aucune nouvelle dépendance n'est prévue sans besoin démontré.
