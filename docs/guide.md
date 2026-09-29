# Guide — fichiers du traitement et lecture du code

Deux usages : savoir quel fichier sert à quoi pendant un traitement, puis retrouver dans le code la partie qui s'occupe d'une chose donnée, sans connaître Rust.

## 1. Les fichiers d'un traitement

L'opérateur ne modifie que **deux classeurs**, toujours depuis les boutons de l'application. Tous les autres fichiers sont produits par l'application et servent seulement à la consultation.

| Fichier (dans le dossier du traitement) | Qui l'écrit | Rôle |
|---|---|---|
| `edition/correspondance_swimlanes.xlsx` | **Vous**, colonne Nouveau nom | Demandes de renommage. Une cellule vide = pas de changement. Enregistrer, fermer, puis « Lire les correspondances » |
| `edition/validation_modifications.xlsx` | **Vous**, colonne Validation (OUI ou NON) | Le classeur de décision, **le seul utilisé** pour les décisions. Enregistrer, fermer, puis « Lire le classeur de décision » |
| Copie enregistrée (emplacement libre) | Vous, ou la personne à qui vous la confiez (valideur, réunion d'arbitrage) | « Enregistrer une copie » : aux Correspondances, le classeur tel qu'il est ; aux Décisions, les propositions **sans réponse**. Une fois remplie, « Importer un classeur de correspondances » ou « Importer un classeur de décision » la copie dans le classeur du traitement, qu'elle **remplace entièrement** (aucune fusion). Le fichier d'origine n'est plus relu ensuite |
| `source/`, `inventaire/`, `analyse/`, `controle/` | L'application | Copie de la source, inventaire, analyse, rapports de contrôle (un par lecture de décisions ; « Ouvrir le rapport de contrôle » ouvre celui des décisions utilisées) : à consulter, jamais à remplir |
| `entrees/` | L'application | Instantanés des classeurs lus. **C'est cette copie qui est utilisée**, pas le fichier d'origine |
| `edition/precedents/` | L'application | Classeur modifié sans être lu puis remplacé (par un import ou une nouvelle analyse) : gardé, jamais perdu |
| `sorties/tentative-NNN/` | L'application | Le SGX produit et vérifié, avec ses rapports. Une tentative par génération |

### Lire l'inventaire (classeur de correspondance)

Seules les colonnes A et B sont lues à l'import. Les autres sont des informations pour décider.

| Colonne | Contenu |
|---|---|
| A — Nom actuel | Nom de swimlane, tel qu'il est dans les modèles (espaces périphériques retirés) |
| B — Nouveau nom | À remplir pour demander un renommage ; vide = aucun changement |
| C — Occurrences | Nombre total de swimlanes portant ce nom, tous modèles confondus |
| D — Nombre de flux | Nombre de **titres de modèles distincts**. Deux modèles différents qui portent le même titre comptent pour un. Ce n'est donc pas un nombre de modèles |
| E — Flux concernés | Ces titres, dans l'ordre de première apparition |
| F — Répétitions dans un même modèle | Les modèles où ce nom apparaît **au moins deux fois**, un par ligne : `Titre (n occurrences) — chemin interne`. Le chemin distingue deux modèles de même titre. Vide si le nom n'est jamais répété dans un modèle |

La colonne F est une information : une répétition ne bloque rien. Le renommage s'applique à toutes les occurrences du modèle, et l'analyse propose une ligne par modèle.

Un traitement créé avant cette évolution garde son ancienne colonne F, « Flux avec occurrences multiples », qui additionnait les occurrences par titre et mêlait les répétitions et les modèles homonymes. Ce classeur reste accepté ; il n'est ni converti ni régénéré.

Ce que l'application garantit :

- La source et les fichiers que vous fournissez ne sont jamais modifiés.
- Aucun fichier existant n'est jamais remplacé, y compris par « Enregistrer une copie », même si vous confirmez « Remplacer » dans le dialogue de Windows.
- Une copie ou un SGX n'apparaît sous son nom final qu'une fois complet. Un arrêt brutal peut laisser un fichier `….copie-en-cours` ou `….en-cours`, qui n'est jamais un résultat. Un emplacement qui ne permet pas cette garantie (clé USB en FAT ou exFAT, certains partages réseau) est refusé avec un message : choisir un disque local.
- La copie d'un SGX produit est comparée à l'empreinte enregistrée lors de sa production : un SGX modifié depuis n'est jamais copié.
- Un classeur modifié sans être relu est signalé « à relire », et ce qui en dépend « à actualiser ». La génération est alors suspendue.
- Seul un OUI identique à une proposition de l'analyse est appliqué. NON, une réponse absente ou une ligne absente n'autorisent rien. OUI et NON sur la même proposition bloquent tout ; le rapport de contrôle marque ces lignes « CONTRADICTOIRE ».
- Le SGX écrit est relu et comparé à la source avant d'être publié : seules les lanes validées peuvent différer.

## 2. Lire le code : quelle partie s'occupe de quoi

Le métier est entièrement dans le moteur Rust (`src/`). L'interface (`ui/`) affiche l'état et envoie les actions : elle ne décide rien.

### Moteur (`src/`)

| Fichier | S'occupe de | Fonctions à lire en premier |
|---|---|---|
| `regles.rs` | Les **règles métier**, en mémoire, sans fichier : trouver les lanes, compter, calculer les propositions, contrôler les décisions, recompter, vérifier le résultat | `trouver_lanes`, `synthetiser`, `lire_correspondances`, `dry_run`, `controler_decisions`, `tester_renommages`, `verifier_modele` |
| `sgx.rs` | L'**archive SGX** : lire les modèles, écrire la nouvelle archive, la relire et la vérifier avant publication | `extraire_lanes`, `produire_sgx`, `verifier_sgx` |
| `excel.rs` | Les **classeurs Excel** : lire les cellules telles quelles (formules comprises), vérifier la feuille et les en-têtes, écrire les rapports | `lire_feuille` |
| `fichiers.rs` | **Empreintes, et publication d'un fichier sans jamais en écraser un** (SGX produit, copies enregistrées) | `publier_sans_ecraser`, `copier_sans_ecraser` |
| `workflow.rs` | L'**enchaînement des étapes**, commun au CLI et à l'application ; le `Bilan` et le `Statut` retournés | `executer` (équivalent du script Python) |
| `traitement.rs` | Le **dossier de traitement** : création, reprise, instantanés, empreintes, révisions, verrou, tentatives | `adopter_correspondances`, `preparer_analyse`, `adopter_decisions`, `produire`, `verifier_integrite` |
| `lib.rs` | Le type `Erreur` (code, message, détails) et ce que le moteur expose | — |
| `main.rs` | Le **CLI historique** (`input/`, `work/`, `output/`), utilisé par la qualification | — |

### Application et interface

| Fichier | S'occupe de |
|---|---|
| `src-tauri/src/lib.rs` | Une commande par action de l'interface ; dialogues de fichiers ; ouverture limitée aux fichiers du traitement |
| `ui/src/moteur.js` | Une fonction JavaScript par commande |
| `ui/src/App.svelte` | L'état affiché (celui retourné par le moteur) et `executer` : une seule action à la fois |
| `ui/src/composants/Etape*.svelte` | Un écran par étape : Source, Correspondances, Analyse, Décisions, Résultat |
| `ui/src/textes.js` | Tous les textes affichés, écran par écran |

### Retrouver le script Python

| Python `main.py` | Rust |
|---|---|
| Recherche et renommage des lanes | `regles.rs` : `trouver_lanes`, `renommer_lanes` |
| Extraction des modèles du SGX | `sgx.rs` : `extraire_lanes` |
| Synthèse et inventaire Excel | `regles.rs` : `synthetiser` ; `workflow.rs` : `inventorier` |
| Colonne F de l'inventaire | `regles.rs` : `repetitions_dans_un_modele`. Évolution V1 : le Python calcule l'ancienne colonne, par titre de modèle (`docs/m3-contrats-v1.md`) |
| Correspondances et analyse | `regles.rs` : `lire_correspondances`, `dry_run` |
| Comparaison des validations | `regles.rs` : `controler_decisions` |
| Renommage en mémoire et écriture | `regles.rs` : `tester_renommages`, `appliquer_renommages` ; `sgx.rs` : `produire_sgx` |

Ordre de lecture conseillé :
1. `workflow::executer`, pour l'enchaînement général ;
2. les fonctions de `regles.rs` ;
3. `sgx.rs` ;
4. `traitement.rs`, qui ajoute la reprise d'un traitement : le script la laissait au mode opératoire.

Pour apprendre, suivre un seul cas (A → Z) de bout en bout est plus utile que tout lire d'un coup.
