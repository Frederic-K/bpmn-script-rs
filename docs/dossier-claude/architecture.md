# BPMN-Script — architecture minimale proposée

Version 1.0 · 26 septembre 2026. Ce document développe P01, P04, P06 et P07 de [product-spec.md](product-spec.md). Les noms de modules et d’opérations sont indicatifs ; leurs responsabilités et les invariants produit sont la référence.

## Direction

Svelte 5 présente le parcours ; Tauri assure la liaison avec Windows ; Rust possède les règles métier, les données de traitement et la production des fichiers. Proposition : TypeScript et Vite pour le frontend, Tauri 2 pour l’application. L’application distribuée ne doit pas dépendre de Python : Python reste un outil de comparaison pendant le développement.

Tauri fournit des commandes pouvant recevoir des arguments et retourner résultats ou erreurs ; utiliser cette frontière pour les opérations métier, avec des objets structurés. Le frontend ne calcule pas lui-même les décisions admissibles. [Documentation des commandes](https://v2.tauri.app/develop/calling-rust/).

Les assets frontend sont construits pour être embarqués dans l’application ; le serveur de développement Vite n’est pas un serveur produit à distribuer. Vérifier la configuration sur la [documentation Tauri/Vite](https://v2.tauri.app/start/frontend/vite/).

## Extraction progressive

Partir du `src/main.rs` existant. Extraire une bibliothèque réutilisable, conserver un CLI léger utilisant cette bibliothèque, puis ajouter l’adaptateur Tauri. Une disposition possible à l’issue du travail :

```text
Cargo.toml                  # workspace Rust après intégration
crates/bpmn-core/           # modèle, règles, workflow, SGX, Excel, traitement
crates/bpmn-cli/            # entrée historique légère pour comparaison
src-tauri/                 # application desktop et adaptation
src/                       # interface Svelte
tests/fixtures/             # données synthétiques versionnées
tests/qualification/        # comparaisons et attentes V1
docs/                      # présent dossier et décisions actualisées
```

Cette organisation est une proposition de destination, pas une obligation de déplacer tous les fichiers dès le premier lot. Commencer par `lib.rs` et une entrée CLI si cela réduit le risque. Ne pas créer une crate par fonction ou introduire une abstraction générique de moteur.

| Responsabilité | Contenu | Ne doit pas dépendre de |
|---|---|---|
| Modèles et règles | Occurrences, synthèse, propositions, décisions, décomptes | Svelte, Tauri, fichiers Excel ouverts |
| SGX | Lecture des JSON, préparation en mémoire, écriture contrôlée | État visuel de l’interface |
| Excel | Parsing typé, diagnostics, rapports et modèles de saisie | Décision métier reconstruite à partir d’un tableau affiché |
| Workflow | Ordre des opérations, conditions de production, bilan | Messages console analysés comme protocole |
| Traitement | Chemins, instantanés, empreintes, état persistant | Répertoire courant global du processus |
| Tauri | Commandes, ouverture de fichiers, résultat sérialisable | Duplications des règles métier |

Préserver les structures JSON inconnues dans un arbre générique. Les structures typées de l’application décrivent les propositions et résultats ; elles ne doivent pas reconstruire un modèle Signavio en perdant les champs non reconnus.

## Contrats du moteur

Les opérations logiques sont : créer/reprendre un traitement, inventorier, adopter les correspondances, préparer l’analyse, adopter/contrôler les décisions, produire et ouvrir un artefact. Les signatures exactes seront définies pendant M2/M4.

Les commandes transportent un identifiant de traitement et une révision attendue, plus les seules données nécessaires. Rust vérifie qu’il s’agit du traitement actif et refuse une requête fondée sur une révision obsolète. Une seule mutation par traitement à la fois, y compris deux clics rapprochés ou deux fenêtres/processus visant le même dossier.

Résultat proposé : `état du traitement`, `révision`, `statut de l’opération`, `compteurs`, `diagnostics`, `artefacts produits`. Statuts d’opération distincts : terminé, sans impact, décisions attendues, aucune décision admissible, contrôle bloquant, produit. Les erreurs techniques sont structurées avec code stable, message compréhensible, contexte et action de reprise ; les détails bruts restent consultables séparément.

Compteurs explicitement nommés : modèles reconnus, noms de flux distincts, occurrences inventoriées, noms distincts, propositions, lignes admises/refusées/ignorées/en attente, propositions sans réponse, modèles modifiés, occurrences effectivement traitées. Ne pas additionner des grandeurs différentes pour fabriquer un total rassurant.

Un diagnostic porte selon le cas le chemin interne, la feuille, la ligne et la colonne. Conserver les valeurs non conformes sous une forme exploitable dans le rapport, sans transformer une formule en formule exécutable lors de sa réécriture.

Les tâches longues de lecture, comparaison et écriture ne bloquent pas l’interface. Publier des phases connues (lecture, analyse, contrôle, écriture) ; ajouter des décomptes seulement s’ils sont mesurés. Aucun besoin de bus d’événements généraliste.

## Persistance minimale — P01

Un traitement possède un dossier autonome avec chemins internes relatifs :

```text
traitement.json
source/source.sgx
edition/                    # copies qu’on ouvre et modifie dans Excel
entrees/<revision>/          # instantanés adoptés, non édités en place
sorties/<tentative>/         # rapports et éventuel SGX de cette tentative
```

Manifeste : version de schéma, identifiant, nom affiché, référence de la source et empreinte, versions/empreintes des Excel adoptés, état, révision, tentative courante, artefacts finalisés. Le chemin original est informatif ; la copie source contrôlée est la référence. Pas de base de données, de fichier système global obligatoire ni de dépendance à une lettre de lecteur fixe.

Les empreintes servent à détecter les modifications ; elles ne prouvent pas que l’arbitre a validé une donnée. Le contrôle métier R07 reste obligatoire. Les instantanés adoptés sont immuables par convention applicative ; vérifier leur intégrité, ne pas supposer que les droits du filesystem empêchent toute modification externe.

Import : lire une copie stable dans une zone temporaire, vérifier les règles, puis adopter une nouvelle révision. Si lecture ou validation globale échoue, ne pas remplacer l’instantané précédent. Les diagnostics par ligne de décisions restent compatibles avec l’adoption d’un classeur contenant des OUI ignorés selon R08/P02.

À la reprise, vérifier schéma, présence et empreintes des artefacts requis. Un original externe absent est acceptable ; un instantané interne altéré ne l’est pas. Une version de manifeste inconnue produit un message explicite, sans réécriture opportuniste. Les fichiers source et les classeurs fournis par l’utilisateur ne sont jamais déplacés ni modifiés.

## Préconditions et invalidations

| Changement | Effet |
|---|---|
| Autre source choisie | Nouveau traitement ; aucune récupération automatique des autorisations anciennes |
| Correspondances adoptées différentes | Invalider analyse et contrôle des décisions ; conserver les anciens résultats comme historiques |
| Classeur d’édition modifié mais non relu | Suspendre la génération et demander l’action « Lire » |
| Décisions réimportées | Invalider le contrôle précédent, conserver l’analyse si ses entrées n’ont pas changé |
| Fichier de sortie supprimé/modifié | Marquer l’artefact indisponible/altéré ; ne pas conserver un succès ouvrable fictif |

Chaque génération utilise un ensemble cohérent de données chargées depuis les instantanés vérifiés. Recalculer l’analyse et les décisions, puis transformer ces mêmes modèles en mémoire. Ne pas relire un autre fichier externe entre contrôle et écriture. Détecter toute révision concurrente et refuser sa publication sur l’ancien contexte.

## Écriture et reprise après erreur — P04/P06

Créer un fichier temporaire dans la destination de la tentative. Écrire et finaliser le ZIP, vérifier sa lisibilité et les invariants de contenu, puis publier sous un nom final unique. La source et une sortie précédente ne sont jamais la destination. Tester les règles de finalisation Windows ; ne pas supposer qu’un renommage remplace un fichier ouvert.

Enregistrer ensuite le résultat dans le manifeste par remplacement contrôlé. Fichier final et manifeste ne constituent pas une transaction unique : si l’enregistrement du manifeste échoue après publication, rendre un état non confirmé. À la reprise, vérifier le fichier orphelin avant association, ou conserver la tentative comme interrompue. Ne jamais déduire le succès du seul nom de fichier.

Conserver la liste et l’ordre des entrées, le contenu non concerné et les attributs pertinents. Les champs ZIP dépendant des offsets, tailles ou CRC doivent être recalculés correctement ; conserver aveuglément tous les octets supplémentaires peut être incorrect. Versionner une fixture avec dates, permissions, commentaires, champs supplémentaires et pièces jointes. Documenter toute limite réelle de la bibliothèque choisie ; ne pas masquer une perte derrière une comparaison des seuls JSON.

## Dépendances et distribution

Conserver initialement les dépendances Rust observées : `zip`, `serde`, `serde_json`, `indexmap`, `calamine`, `rust_xlsxwriter`, avec le verrou fourni. Rust 1.88.0 est la version testée pour le CLI actuel ; ce n’est pas une garantie sur toutes les dépendances futures Tauri. Si une version plus récente devient nécessaire, expliciter et verrouiller le changement, puis relancer les contrôles.

Lors du scaffold : verrouiller une version précise de Rust et les résolutions frontend ; documenter Node et le gestionnaire de paquets choisis. Ajouter seulement les capacités nécessaires pour sélectionner et ouvrir les fichiers gérés. Ne pas donner à l’interface un accès général au shell pour ouvrir Excel. Les chemins contenus dans un manifeste sont validés à la reprise et résolus dans le dossier du traitement.

Windows nécessite un choix explicite de fourniture de WebView2 ; les modes d’installation dépendent du poste et de l’accès réseau. Vérifier l’installateur et le lancement sur un poste cible. Ne pas assimiler un `.exe` compilé à une distribution portable autonome. [Documentation Windows Tauri](https://v2.tauri.app/distribute/windows-installer/).

## Stratégie de tests

Garder la référence d’origine intacte. Ajouter des tests unitaires Rust sur les règles et des tests d’intégration sur fichiers. Le pilote externe fourni reste utile pour exécuter le binaire reconstruit. Sa liste d’écarts attendus est une caractérisation de l’ancien code, pas le contrat final : ne pas la modifier simplement pour faire disparaître un échec.

Après une correction P02/P03/P06, ajouter l’attente V1 avec un exemple avant/après et expliquer pourquoi la comparaison historique diverge. Après M2, tous les écarts nouveaux inexpliqués sont des régressions. Les tests d’interface vérifient les transitions et les vrais résultats du moteur ; l’import Signavio reste un contrôle manuel séparé.
