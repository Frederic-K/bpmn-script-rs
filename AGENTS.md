# Consignes pour les agents (Claude, Codex…)

BPMN-Script : application Windows locale qui harmonise les noms de swimlanes des modèles d'un export Signavio (`.sgx`). L'opérateur décide des renommages dans Excel ; l'application inventorie, contrôle et produit un nouveau SGX **sans jamais modifier la source**. Un SGX erroné importé en production aurait des conséquences lourdes : la robustesse passe avant tout.

État courant, décisions et guide de revue : [SESSION.md](SESSION.md). Fichiers d'un traitement et correspondance code / Python : [docs/guide.md](docs/guide.md). Spécification d'origine (figée, ne pas modifier) : [docs/dossier-claude/](docs/dossier-claude/).

## Style imposé par le propriétaire

- **KISS / YAGNI.** Pas d'abstraction anticipée, pas de couche générique, pas de dépendance sans besoin concret.
- **Noms explicites en français**, sans abréviation (sauf les évidences : `e` pour un événement, `i` pour un index).
- **Commentaires `//` simples, en français**, seulement pour expliquer un choix non évident. Pas de `///`, `//!` ni de JSDoc.
- **Interface : Svelte 5 en JavaScript uniquement.** Aucun `.ts`, aucun `lang="ts"`, aucune configuration TypeScript. Pas de store global, pas de mémoïsation, pas d'habitudes React.
- Composants aux responsabilités claires : ni composant monolithique, ni micro-composants.
- **Styles : Tailwind CSS v4**, en classes utilitaires dans les composants. Les couleurs sont définies une seule fois dans `@theme` (`ui/src/style.css`) et le thème sombre redéfinit ces variables : pas de classes `dark:`. Les éléments répétés sont des composants (`Bouton`, `Pastille`, `Chiffre`, `Tableau`), jamais des classes `@apply`. Les classes choisies en JavaScript sont écrites en entier (Tailwind ne détecte pas les noms construits).
- **Textes de l'interface** : tous dans `ui/src/textes.js`, regroupés par écran ; une phrase à paramètres est une fonction. Pas de bibliothèque ni de sélecteur de langue. Les messages d'erreur et motifs de contrôle viennent du moteur Rust et sont affichés tels quels.
- **Thème** : bascule clair/sombre par `data-theme` sur `<html>` (`main.js` à l'ouverture, `basculerTheme` dans `App.svelte`), choix mémorisé dans `localStorage`, thème du système au premier lancement.

## Structure

| Chemin | Rôle |
|---|---|
| `src/regles.rs` | Règles métier en mémoire : inventaire, synthèse, correspondances, dry-run, contrôle des décisions, recomptage, vérification du résultat. Aucun accès fichier |
| `src/sgx.rs` | Lecture de l'archive (anomalies P03), écriture, **relecture et vérification** puis publication |
| `src/fichiers.rs` | Publication sans écrasement : temporaire complet, puis lien physique (refusé si la destination existe) ; copies enregistrées par l'utilisateur |
| `src/excel.rs` | Lecture brute des cellules (formules comprises), contrôle feuille et en-têtes, écriture des classeurs |
| `src/workflow.rs` | Étapes communes, `Bilan`, `Statut`, mode historique `input/`, `work/`, `output/` |
| `src/traitement.rs` | Dossier de traitement : manifeste, empreintes, instantanés, révisions, verrou, tentatives |
| `src/lib.rs` | `Erreur` (code stable, message, détails), exports publics |
| `src/main.rs` | CLI historique (utilisé par le pilote de qualification) |
| `src-tauri/src/lib.rs` | Commandes Tauri : une par opération, dialogues, ouverture de fichiers limitée au traitement |
| `ui/src/App.svelte` | Seul détenteur de l'état de l'interface ; `executer` : une action à la fois |
| `ui/src/moteur.js` | Une fonction par commande Tauri |
| `ui/src/textes.js` | Catalogue des textes français de l'interface, par écran |
| `ui/src/composants/` | Un composant par étape ; `Etapes`, `BilanTraitement`, `Message`, `Fichier` ; éléments répétés `Bouton`, `Pastille`, `Chiffre`, `Tableau` |
| `ui/src/style.css` | Tailwind : couleurs (`@theme`) et leurs valeurs du thème sombre |
| `tests/` | `cli.rs`, `chemins.rs`, `traitement.rs` ; `interface/parcours.py` (bout en bout) ; `qualification/verifier_v1.py` |
| `docs/m0…m6-*.md` | Comptes rendus des lots, avec preuves et limites |

## Commandes

```sh
npm ci                                            # dépendances interface (versions exactes)
cargo test --workspace --locked                   # 90 tests Rust
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
npm run build                                     # interface : doit rester sans avertissement
npm run tauri dev                                 # application en développement
npm run tauri build                               # release + installateur NSIS (Windows)
```

- **Qualification** (30 scénarios comparés au Python d'origine) : voir `tests/qualification/verifier_v1.py` et `.github/workflows/windows.yml`.
- **Bout en bout (Linux)** : `tests/interface/parcours.py`, qui nécessite tauri-driver, WebKitWebDriver, Xvfb, xdotool, selenium et openpyxl.
- **Intégration continue Windows** : `.github/workflows/windows.yml` (tests, installateur, qualification, lancement).

## Outillage verrouillé

- **Rust 1.89.0**, fixé par `rust-toolchain.toml` : minimum requis pour `File::try_lock`. Le composant `rust-analyzer` y figure pour que VS Code utilise la version accordée à cette toolchain (celle embarquée par l'extension est trop récente).
- **Cargo** : `Cargo.lock` unique pour l'espace de travail (moteur et `src-tauri`). Construire avec `--locked` ; ne pas mettre à jour les dépendances par commodité.
- **npm** : versions exactes dans `package.json`, verrouillées par `package-lock.json`. Installer avec `npm ci`.
- **Fonctionnalités de dépendances volontaires** :
  - `serde_json` : `preserve_order` et `arbitrary_precision` (les nombres des modèles sont recopiés à l'identique) ;
  - `zip` : `unreserved` (conservation des champs supplémentaires Java comme `0xCAFE`).

## Invariants métier (product-spec R01–R12)

- Source SGX jamais modifiée ; sortie toujours distincte ; **jamais d'écrasement** d'un fichier existant, y compris par une copie enregistrée (publication par `fichiers::publier_sans_ecraser`, jamais `exists()` puis `rename`, qui écrase sous Windows).
- Identité d'un modèle = chemin interne complet (`…model_1_.json`), jamais le nom affiché.
- Seul un OUI dont les cinq champs (chemin, flux, nom actuel, nouveau nom, occurrences) sont valides et identiques à une proposition est admis. NON, attente et ligne absente n'autorisent rien. **OUI et NON sur la même proposition = contrôle bloquant** (aucune ligne ne l'emporte).
- Renommages appliqués séquentiellement et recomptés ; **toute divergence = aucun SGX**.
- Tout JSON ou Excel mal formé est signalé (P02, P03) ; jamais de conversion silencieuse ni de résultat partiel présenté comme complet.
- Le SGX écrit est relu et comparé à la source avant publication : seules les lanes validées peuvent différer (`regles::verifier_modele`, `sgx::verifier_sgx`). **Ne jamais affaiblir ce contrôle.**
- Rust est l'autorité : l'interface ne calcule aucune décision ; un bouton désactivé ne remplace pas un contrôle du moteur.
- Excel reste l'outil d'édition ; aucune grille éditable dans l'application.

## Pièges connus

- **Indexation mutable de `serde_json`** : `valeur["cle"]` sur un `&mut Value` insère `null` si la clé manque. Utiliser `get_mut` (voir `regles::enfants_modifiables`).
- **Messages du CLI** : ils font partie de la comparaison de qualification ; ne pas en changer l'ordre sans raison.
- **Données obsolètes** : un classeur modifié sans être relu (`*_a_relire`) laisse le dernier bilan en place ; l'interface doit le présenter comme « à actualiser », jamais comme l'état actuel.
- **Un seul classeur de décision** : `edition/validation_modifications.xlsx`. Un retour importé le remplace entièrement (pas de fusion). Le classeur d'analyse a lui aussi une colonne Validation : ne pas le proposer à l'ouverture dans l'interface.
- **Non calculé ≠ vide** : `etat.propositions` est vide tant que l'analyse n'est pas préparée ; tester `analyse_preparee` avant de conclure à « aucun changement ».
- **Chemins Windows** : ne pas transmettre la forme canonique `\\?\` à Excel ou à l'Explorateur.

## Interdits

- Versionner SAPHIR ou toute donnée métier réelle (utiliser des fixtures synthétiques).
- Modifier `docs/dossier-claude/` (empreintes de référence) ou le paquet de qualification qu'il contient.
- Modifier une attente de test pour faire passer un changement sans l'expliquer (cas avant/après, référence P/R/A).
- Pousser directement sur `main`, réécrire l'historique partagé, fusionner sans demande.
- Revendiquer une vérification non faite : distinguer compilé, testé sur fixtures, testé sous Windows, testé sur copie SAPHIR, importé dans Signavio.
