# M3 — Contrats V1

26 septembre 2026 · lot M3 de [implementation-plan.md](dossier-claude/implementation-plan.md), suite de [m2-extraction.md](m2-extraction.md). Branche `claude/m3-contrats-v1`.

Propositions P02 à P06 **approuvées par l'utilisateur** le 26 septembre 2026 (« tu peux tout approuver sur la partie dev »), avec deux consignes : KISS/YAGNI et robustesse avant tout. Le risque principal est l'import en production d'un SGX contenant une erreur non détectée.

## Changements par proposition

### P05 — Analyse vide et dossier de sortie

Comportement Rust conservé et désormais explicite : `[RÉSULTAT] aucun changement proposé…`. Sans proposition, le classeur de décisions n'est plus lu (rien à décider).

### P03 — Modèles mal formés

Toute anomalie bloque l'inventaire, et donc tout le traitement. L'erreur liste chaque anomalie avec son emplacement (`dossier/a/model_1_.json : racine.childShapes[1].properties.name : texte attendu`).

| Situation | Avant | V1 |
|---|---|---|
| JSON de modèle ou de métadonnées illisible | Erreur générique | Anomalie localisée |
| Métadonnées absentes | Erreur générique | Anomalie localisée |
| `name` du flux non textuel | Traité comme vide | Anomalie |
| Nom de lane nul, nombre, objet | Lane ignorée silencieusement | Anomalie |
| `properties` non objet | Lane ignorée | Anomalie |
| `childShapes` non tableau, enfant non objet | Enfants ignorés | Anomalie |
| Entrée ZIP en double | Non détecté | Anomalie |
| Archive sans modèle `model_1_.json` | Inventaire vide « réussi » | Erreur « Aucun modèle compatible trouvé » |
| Champs absents (`properties`, `name`, `childShapes`) | Vides | Vides (convention historique conservée) |

### P02 — Import Excel strict

Commun aux deux classeurs :
- la feuille attendue doit exister ; sinon, la liste des feuilles trouvées est donnée ;
- les en-têtes des colonnes utilisées doivent être exacts, espaces périphériques ignorés ;
- les colonnes supplémentaires et les lignes entièrement vides sont ignorées ;
- les formules sont détectées, avec ou sans valeur calculée, et ne sont jamais évaluées.

**Correspondances.** Une ligne dont le nouveau nom est vide ou ne contient que des espaces ne porte pas de demande. Sur une ligne avec demande, les deux noms doivent être du texte. Toute erreur (nombre, booléen, erreur Excel, formule, ancien nom vide) **refuse le classeur entier**, avec un diagnostic par cellule (`Correspondance, ligne 3, colonne B (Nouveau nom) : texte attendu (valeur lue : nombre 123)`). Deux informations sont signalées sans bloquer : un ancien nom saisi plusieurs fois (dernière ligne retenue) et un ancien nom absent de l'inventaire (souvent une faute de frappe).

**Décisions.** Les six champs sont lus sans conversion :
- une ligne OUI dont un champ est invalide est IGNORÉE avec le motif de la cellule, et les autres lignes restent traitées ;
- les occurrences doivent être un nombre entier positif ou nul : `1.0` est accepté ; le texte « 1 », un booléen, un nombre négatif ou fractionnaire ne le sont pas ;
- une validation non textuelle (formule, booléen) ne donne aucun droit : EN ATTENTE avec motif ;
- une valeur autre que OUI ou NON reste EN ATTENTE, avec un motif désormais explicite ;
- le rapport de contrôle conserve le numéro de ligne Excel d'origine, et les valeurs brutes sont recopiées comme texte (une formule ne redevient jamais exécutable) ;
- le bilan compte les propositions de l'analyse restées sans réponse (lignes absentes).

### P04 — Résultat explicite et écriture sûre

- `executer` retourne un `Bilan` : un `Statut` (InventaireTermine, AnalyseSansImpact, DecisionsAttendues, AucuneDecisionAdmissible, ControleBloquant, SgxProduit) et des compteurs nommés.
- Les erreurs ont un code stable (`modeles_invalides`, `classeur_cellules`, `verification_echouee`…), un message et des détails, pour l'interface.
- CLI : dernière ligne `[RÉSULTAT] …`. Code de sortie : 0 si l'étape s'est terminée normalement, 1 en cas d'erreur, **2 si des décisions ont été fournies sans qu'un SGX soit produit** (auparavant 0).
- Écriture : fichier `…sgx.en-cours`, synchronisé sur disque, **relu puis vérifié**, et publié sous le premier nom libre (`export_modifie.sgx`, puis `export_modifie_2.sgx`…). Une sortie existante n'est jamais écrasée. En cas d'échec, le temporaire est supprimé et aucun fichier final n'est créé.

**Vérification du SGX produit**, indépendante du test en mémoire :
- mêmes entrées, dans le même ordre ;
- chaque entrée non modifiée est identique octet pour octet après décompression, sommes de contrôle vérifiées ;
- chaque modèle modifié est comparé à sa source, à laquelle on applique en une passe les seuls renommages validés : toute autre différence (autre lane, autre propriété, coquille, nombre d'occurrences) bloque la publication.

### P06 — Préservation ZIP

| Élément | Avant | V1 |
|---|---|---|
| Commentaire d'archive | Perdu | Conservé |
| Commentaire de l'entrée modifiée | Perdu | Conservé |
| Champs supplémentaires de l'entrée modifiée | Perdus | Conservés, sauf ZIP64 (recalculé). Fonctionnalité `unreserved` de la crate `zip` activée pour conserver les champs Java comme `0xCAFE` |
| Date, méthode de compression, permissions Unix de l'entrée modifiée | Date et méthode conservées | Conservées |
| Nombres JSON des modèles modifiés | Réécrits en flottants | Texte conservé (`serde_json` `arbitrary_precision`) ; seule la notation à exposant est normalisée (`1e2` devient `1e+2`, même valeur) |

**Limites de la bibliothèque `zip`**, documentées et vérifiées par un test :
- `external_attr` et le système d'origine sont recalculés, sur toutes les entrées ;
- les champs supplémentaires des entrées **non modifiées** (recopiées sans recompression) sont perdus ; leur contenu, lui, est identique octet pour octet.

Une transformation refusée par la bibliothèque est listée dans `Bilan.transformations`. La qualification Signavio (A17) reste nécessaire.

## Évolution après la revue du parcours — décisions contradictoires

Ajoutée après M6, à la suite de la revue Codex du commit `abe5311`.

**Avant :** une même proposition présente deux fois dans le classeur de décision, avec OUI sur une ligne et NON sur l'autre, donnait une admission et un refus. Le OUI était appliqué. Le Python d'origine se comporte de la même façon : c'est une limite héritée.

**Maintenant :** une proposition qui reçoit OUI et NON, dans n'importe quel ordre, est une **contradiction**. Aucune des deux lignes ne l'emporte. Le contrôle est bloquant (`Statut::ControleBloquant`, `Bilan.contradictions`), et aucun SGX n'est produit. Le CLI écrit une ligne `[ERREUR] Ligne n (OUI) et ligne m (NON) : décisions contraires pour « A » → « Z » (fichier modèle)` et sort avec le code 2.

- La contradiction porte sur la proposition exacte, c'est-à-dire ses cinq champs. Un NON sur une autre proposition n'est pas une contradiction.
- Après la contre-revue de `d3c57af`, le rapport `controle_validation.xlsx` reste lisible seul. Les lignes concernées ont le résultat `CONTRADICTOIRE`, et leur motif indique la ligne opposée et l'absence de SGX. La proposition contradictoire n'est ni admise (`modifications_validees.json`) ni comptée comme refusée. Le nom de feuille et les colonnes du rapport sont inchangés.
- Un OUI répété n'est pas une contradiction. R09 conserve ce doublon, et le recomptage le bloque, comme avant (scénario `duplicate_validation`).
- Aucun des 30 scénarios de qualification ne contient de contradiction : les attentes V1 sont inchangées (`tests/qualification/verifier_v1.py`, vérifié après la modification).

Tests : `regles::decisions_contradictoires_signalees` (les deux ordres, OUI répété, autre proposition), `cli::decisions_contradictoires_bloquent_la_production`, `traitement::decisions_contradictoires_bloquent_la_production`.

## Évolution — colonne F de l'inventaire

Décision du propriétaire, après analyse de l'export SAPHIR. L'application harmonise les noms de swimlanes ; elle ne recherche pas les modèles Signavio homonymes.

**Avant :** la colonne F « Flux avec occurrences multiples » listait `titre (n)` pour chaque titre de modèle où la swimlane apparaissait plus d'une fois. Le décompte se faisait par titre (`occurrences_par_flux`) : deux cas différents étaient confondus.

| Cas | Exemple | Ancienne colonne F |
|---|---|---|
| Deux swimlanes de même nom dans un modèle | « A » deux fois dans le modèle `a/model_1_.json` de titre « Flux » | `Flux (2)` |
| Une swimlane dans deux modèles distincts de même titre | « A » une fois dans `a/…` et une fois dans `b/…`, tous deux de titre « Flux » | `Flux (2)` |

Sur SAPHIR, 43 mentions en F : 6 du premier cas, 37 du second.

**Maintenant :** la colonne F s'appelle « Répétitions dans un même modèle ».

- Une entrée par modèle (chemin interne) où le nom apparaît au moins deux fois : `Titre du modèle (n occurrences) — chemin/interne/model_1_.json`.
- Une entrée par ligne dans la cellule (retour à la ligne actif, comme avant).
- Ordre de première apparition du modèle ; cellule vide si aucun modèle n'est concerné.
- Des modèles homonymes sans répétition n'apparaissent plus. Deux homonymes qui répètent chacun le nom donnent deux entrées, distinguées par leur chemin.
- Information seulement : aucun contrôle, aucune décision et aucun message CLI n'en dépendent.
- Calcul : `regles::repetitions_dans_un_modele`, à partir de `occurrences_par_modele`.

Sur SAPHIR : 4 lignes renseignées, 6 entrées.

**Calcul supprimé :** `occurrences_par_flux` ne servait qu'à l'ancienne colonne F. Le champ est retiré de `Synthese`, de `synthetiser` et donc des nouveaux `synthese.json`. Usages vérifiés : `src/excel.rs` (seul consommateur), un test unitaire et un test CLI (attentes mises à jour). Aucun code, ni l'interface, ne relit `synthese.json`.

**Conservé :** colonnes A à E ; « Nombre de flux » (D) compte toujours les titres distincts, pas les modèles (R05) ; `occurrences`, `flux` et `occurrences_par_modele` dans `synthese.json`.

**Compatibilité :**

- L'import des correspondances ne lit et ne contrôle que les colonnes A et B : les classeurs à l'ancienne colonne F restent acceptés.
- Un traitement créé avant cette évolution garde son inventaire, son `synthese.json` et son classeur d'édition tels quels : rien n'est migré ni régénéré. La reprise recalcule l'inventaire depuis la copie de la source et ne relit pas `synthese.json`.
- La spécification figée (`docs/dossier-claude/product-spec.md`, « Formats d'échange ») mentionne encore l'ancien libellé de F et le JSON historique : elle n'est pas modifiée ; cette section fait foi pour la V1.

**Qualification :** `tests/qualification/verifier_v1.py` relit les sorties de chaque scénario (`qualification-runs/`) et n'accepte que deux écarts avec Python :

- `synthese.json` : identique au Python après retrait de `occurrences_par_flux`, ordre des clés compris ;
- `inventaire_swimlanes.xlsx` : mêmes lignes, six colonnes, A à E identiques, ancien en-tête F côté Python, nouvel en-tête côté Rust, et colonne F égale à la valeur recalculée depuis le `synthese.json` Python.

Tout autre écart reste une régression, et les exceptions V1 précédentes sont inchangées. Le script vérifie aussi que l'évolution est présente (scénario `inventory`) et, si une copie de SAPHIR a été fournie au pilote, que seules ces deux sorties diffèrent à chaque étape. Contrôles de ce script : 5 altérations volontaires des sorties détectées (colonne F, colonne C, en-tête F, `synthese.json`, écart sur un autre fichier) ; le binaire antérieur est refusé.

Tests : `regles::repetitions_*` (six cas), `cli::inventaire_seul_sans_correspondance`, `cli::repetitions_de_modeles_homonymes_distinguees_par_chemin`, `cli::ancien_classeur_a_six_colonnes_accepte`, `traitement::traitement_cree_avant_l_evolution_de_la_colonne_f_reste_utilisable`.

### Note pour le refactor Python

Même correction à reporter, comme **évolution fonctionnelle distincte** : ne pas la mêler à un refactor à comportement inchangé, dont la comparaison au tag `v0.1-sgx-import-tested` doit rester stricte.

- Remplacer F par « Répétitions dans un même modèle », même format et même ordre qu'ici (depuis `occurrences_par_modele`).
- Supprimer `occurrences_par_flux` de la synthèse et de `synthese.json`.
- Dans le test de référence, n'autoriser que ces deux écarts, contrôlés exactement comme dans `verifier_v1.py`.

## Défaut hérité corrigé

La vérification du SGX produit a révélé un défaut du portage d'origine. `forme["childShapes"].as_array_mut()` utilise l'indexation mutable de `serde_json`, qui **ajoute `"childShapes": null` à chaque forme qui n'en a pas**, dans tous les modèles réécrits. Aucun test ne le voyait, car toutes les formes des fixtures avaient un `childShapes`. Le parcours utilise désormais `get_mut`, et un test couvre ce cas (`renommage_n_ajoute_aucune_cle`).

## Autres changements

- Noms explicites dans tout le code (`ws`, `pm`, `Res`… remplacés) et commentaires `//` simples.
- Dépendances : aucune nouvelle crate. Fonctionnalités `zip/unreserved` et `serde_json/arbitrary_precision` ajoutées ; `Cargo.lock` inchangé.

## Preuves d'exécution (Linux, Rust 1.88.0)

| Contrôle | Résultat |
|---|---|
| `cargo test --locked` | 26 tests unitaires, 2 de bibliothèque, 33 CLI : tous réussis |
| `cargo clippy --locked --all-targets -- -D warnings`, `cargo fmt --check` | Propres |
| Pilote de qualification, binaire M3 | 30 scénarios exécutés ; divergences toutes expliquées ci-dessous. `check_qualification.py` échoue comme attendu : ses assertions décrivent le comportement historique |

### Divergences de qualification par rapport à M0

| Scénarios | Changement | Réf. |
|---|---|---|
| inventory, dry_run, accepted, mixed, normalized_yes, empty_mapping, unknown_mapping, duplicate_mapping, missing_output, partial, same_name | Mêmes fichiers et mêmes codes ; une ligne `[RÉSULTAT]` s'ajoute | P04 |
| refused, stale_name, stale_count, stale_path, stale_flux, duplicate_different, old_output | Code 0 → 2 (aucune décision admissible) ; fichiers identiques | P04 |
| duplicate_validation, chain_conflict | Code 0 → 2 (contrôle bloquant) ; fichiers identiques | P04 |
| pending | Code 0 → 2 ; motif « Validation « PEUT-ÊTRE » non reconnue : OUI ou NON attendu » au lieu de « Aucune validation renseignée » | P02 |
| numeric_old, numeric_new, formula_new | Classeur de correspondance refusé (code 1), diagnostics par cellule ; plus d'analyse ni de SGX | P02 |
| null_name, bad_children | Anomalie bloquante (code 1) ; plus d'inventaire partiel ni de SGX | P03 |
| zip_attributes | Commentaire et champ supplémentaire de l'entrée modifiée conservés, commentaire d'archive conservé ; restent `external_attr` et `create_system` | P06 |
| invalid_zip, no_source, multiple_sources | Même code 1 ; message plus explicite pour l'archive illisible | P04 |

## Limites

- Tests exécutés sous Linux uniquement.
- La vérification compare les objets JSON sans tenir compte de l'ordre des clés. Le programme ne réordonne jamais les clés : `preserve_order` est actif.
- SAPHIR non rejoué, aucun import Signavio.

## Lot suivant — M4

Dossier de traitement (P01) : copie source contrôlée, instantanés des classeurs adoptés, manifeste versionné, reprise, invalidations, tentatives numérotées et verrou contre un double lancement.
