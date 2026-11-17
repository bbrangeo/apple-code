# Tests des exemples du README

Date : 2026-11-17. Version du checkout : `0.1.2`.

## Protocole

- Compilation et exécution du vrai backend Apple Intelligence, pas du mock.
- Essais des commandes CLI sur le binaire compilé, avec les arguments des exemples du README. Pour les variantes `cargo run -- …`, l'appel direct au binaire a évité de recompiler à chaque cas ; `cargo run -- --help` a aussi été exécuté.
- Copie de `src/` et `config.toml` dans un espace temporaire, avec `HOME` isolé. Les écritures, l'édition, les configurations et les sessions n'affectent pas les données utilisateur.
- Le vrai diff du checkout a été capturé avant le lancement des essais. Sa taille et son contenu influencent les résultats des résumés.
- Les sorties redirigées ont été capturées dans des fichiers de logs.
- REPL testé avec un pseudo-terminal et commandes envoyées successivement : `/help`, `/history`, `/sessions`, `/models`, `/languages`, `/test`, `/clear`, `/exit`.
- Installations réalisées avec `--root` temporaire pour ne pas remplacer le binaire utilisateur. Installation crates.io puis réinstallation avec `--force` et `MACOSX_DEPLOYMENT_TARGET=26.0` ; installation du checkout avec `--path . --force`.
- Presse-papiers sauvegardé comme enregistrement AppleScript, test `--copy`, puis restauration. Texte généré présent : 117 caractères lors de cet essai.
- Dossier des résultats bruts : `/tmp/apple-code-readme.dNvfDn/` (temporaire, non versionné).

Un code de sortie nul ne valide ni la syntaxe du code généré, ni sa pertinence.

## Résultats des exemples

| Exemple du README | Résultat observé |
|---|---|
| `cargo install apple-code-assistant` et variante avec cible macOS/`--force` | Installation et réinstallation réussies dans une racine temporaire. La version disponible sur crates.io est **0.1.1**, pas celle du checkout. La première installation a également utilisé la cible macOS explicite. |
| `apple-code --help` | OK : aide et options affichées. |
| `cargo build` | OK, avec trois avertissements préexistants. |
| `cargo run -- --help` | OK. |
| `cargo install --path . --force` puis `apple-code --version` | OK dans une racine temporaire : **0.1.2**. |
| Hello world Python en tool mode | OK : `print("hello world")`. |
| Résumé du diff avec `--tool-mode --char-limit 2000 --max-tokens 512` | Exécution réussie, mais résultat partiel : ne décrit que les limites modifiées dans le début du diff. Un essai supplémentaire sur un petit diff a produit du code au lieu d'un résumé. **Pertinence non garantie.** |
| `-T tests --tool-mode` sans configuration explicite | **Échec** en environnement vierge : `Unknown prompt template: 'tests'`. |
| Recherche binaire Python, `--save -o … --tool-mode` | OK : fichier Python sans balises ; syntaxe et cinq cas fonctionnels vérifiés en VM. |
| Template `strict_code` avec `--config config.toml` | OK : deuxième fichier Python ; syntaxe et cinq cas fonctionnels vérifiés en VM. Le bloc TOML du README est présent dans la configuration d'exemple et a été chargé. |
| `cargo run -- -i` | Binaire interactif démarré en pseudo-terminal. Commandes ci-dessus exécutées et sortie propre. `/models`, `/languages` et `/test` ont des réponses statiques ; `/test` ne vérifie pas l'API. |
| Hello world Python avec `--preview` | OK : code Python et rendu de prévisualisation. |
| Quicksort Rust avec `--save -o sort.rs` sans tool mode | **Échec de contenu** : fichier écrit mais entouré de balises ` ```rust ` ; rejeté par `rustfmt`. Variante avec `--tool-mode` testée : syntaxe Rust acceptée par `rustfmt` ; comportement du tri non exécuté. |
| `--copy` | OK : texte généré copié ; presse-papiers initial restauré. La qualité de la fonction générée n'a pas été validée. |
| `git diff \| apple-code -p "summarize the changes"` | **Échec** sur le diff actuel : réponse non JSON du pont, compatible avec un dépassement du contexte. Sans tool mode, la consigne de génération de code reste en outre active. |
| Même résumé redirigé vers `summary.txt` | **Échec** identique ; stdout capturé vide. |
| `cat src/main.rs \| apple-code -p "add doc comments to this code"` | **Échec de contenu** : sortie TypeScript pour une entrée Rust, en raison du langage configuré par défaut en environnement vierge. |
| `-T tests` sans tool mode et sans configuration explicite | **Échec** : template inconnu. |
| `-T refactor -l rust` sans configuration explicite | **Échec** : template inconnu. |
| Reprise de conversation : deux prompts avec `--extend-conversation` | Les deux appels réussissent ; une session contient quatre messages, dans le bon ordre. Réponses non pertinentes : objet TypeScript inventé, identique aux deux appels ; aucun contexte du projet n'est fourni automatiquement. |
| `--repeat-input --tool-mode` sur le diff réel | **Échec** de génération sur le diff actuel. Un essai distinct avec un marqueur court confirme que le stdin n'est **pas répété** : sortie `DONE`, sans marqueur. |
| `--char-limit 8000 --context-glob "src/**/*.rs"` | **Échec** sur les données du checkout : erreur JSON du pont. La présence d'une limite de caractères ne garantit pas de tenir dans le contexte. |
| `-p "add error handling" -e src/main.rs` | **Échec de contenu** : la copie temporaire du fichier Rust est écrasée par du TypeScript entouré de Markdown. Le fichier du checkout n'a pas été touché. |
| `config --list` | OK : valeurs affichées. |
| `config --get theme` | OK : `dark`. |
| `config --set default_language=python` | OK : relecture confirmée à `python`. |
| `config --reset` | OK : relecture confirmée à `typescript`. Configuration utilisateur réelle inchangée. |
| `models` | OK : liste affichée, statique ; ne prouve pas la disponibilité de chaque modèle. |
| `languages` | OK : liste et alias affichés. |
| `test` | OK lors de cet essai : `API test: OK`. Attention : l'implémentation peut aussi retourner un code de sortie nul après une erreur de génération. |

## Essais complémentaires

- `--config config.toml -T tests --tool-mode` avec une petite fonction Python fournie sur stdin : assertions Python générées, exit 0. Leur sémantique n'a pas été exécutée.
- `--config config.toml -T refactor -l rust` avec une petite fonction Rust : sortie générée, exit 0. Pertinence et compilation du résultat non garanties.
- Les deux fichiers de recherche binaire ont été exécutés en microVM smolvm (Python 3.12), avec tableau vide, cible présente, cible absente et plusieurs éléments : **10 vérifications fonctionnelles réussies**, plus analyse syntaxique des deux fichiers.
- Les sessions de test ont été inspectées : persistance effective de deux tours dans une même session.

## Validation du projet

| Commande | Résultat |
|---|---|
| `cargo fmt --check` | Réussi. |
| `cargo test` | Réussi : 2 tests unitaires, aucun test d'intégration ou doctest. |
| `cargo doc --no-deps` | Réussi. |
| `git diff --check` | Réussi. |
| `cargo clippy --all-targets --all-features -- -D warnings` | **Échec** : imports inutilisés/redondants, fonction morte, module de tests avant des items, conversions superflues et autres lints. 16 diagnostics pour la bibliothèque, 17 pour sa variante test. Aucun correctif appliqué pendant cette passe. |

## Priorités révélées par les tests

1. Ne pas écraser des fichiers de code avec une réponse Markdown ou un autre langage ; préciser au minimum langage et tool mode dans les exemples concernés.
2. Charger explicitement `config.toml` dans tous les exemples de templates ; fournir aussi le code auquel « this function/module » fait référence.
3. Réparer `--repeat-input`, ignoré en tool mode.
4. Corriger en amont le doublonnage du prompt et le format des erreurs dans `apple_ai`, puis prévoir une stratégie pour les gros contextes. Réduire le diff ne constitue pas un résumé complet.
5. Distinguer le succès d'exécution de la qualité des générations, particulièrement pour les résumés et la reprise de conversation.
6. Corriger les lints Clippy dans une passe séparée.

Ce rapport décrit les observations de cette exécution ; les générations du modèle sont non déterministes. Aucune correction de l'implémentation ni aucun commit n'a été effectué pendant cette passe.

## Suivi : correction des exemples de fichiers source

Le README a ensuite été corrigé, sans modification de l'implémentation :

- Les exemples Rust destinés à des fichiers ou à une redirection précisent `-l rust --tool-mode`.
- L'exemple de sauvegarde demande un petit adaptateur autour de `sort_unstable`, plutôt qu'une implémentation de quicksort inventée par le modèle. Il est suivi d'une commande `rustc` de vérification.
- L'édition se fait sur `main.generated.rs`, une copie de `src/main.rs`, avec une consigne précise d'ajout de contexte d'erreur via `anyhow::Context`. Un diff est demandé avant toute intégration.
- Le README précise que le mode brut ne valide pas le programme et qu'une réponse doit être revue et testée.

Vérifications des nouvelles variantes, avec sorties dans `/tmp/apple-code-save-check.eApNEN/` :

| Résultat généré | Contrôle |
|---|---|
| `sort.rs` (`sort_numbers` utilisant `sort_unstable`) | Compilation Rust en bibliothèque, émission de métadonnées : réussie. |
| `main.generated.rs` après édition | Compilation Rust avec les dépendances du projet, émission de métadonnées : réussie. |
| `main.documented.rs` après ajout de commentaires | Analyse par rustfmt et compilation Rust avec les dépendances du projet, émission de métadonnées : réussies. |

Les deux premiers prompts de tri et d'édition, plus vagues, ont encore généré des erreurs Rust malgré le mode brut ; ces échecs ont motivé les exemples plus simples et précis. Les vérifications ci-dessus ne garantissent pas les générations futures ni l'équivalence fonctionnelle de tout code réécrit. `src/main.rs` du checkout n'a pas été modifié. Les deux tests unitaires et le contrôle du formatage du projet passent.
