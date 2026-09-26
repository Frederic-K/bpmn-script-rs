# M2 — Extraction de la bibliothèque

26 septembre 2026 · lot M2 de [implementation-plan.md](dossier-claude/implementation-plan.md), suite de [m1-tests-caracterisation.md](m1-tests-caracterisation.md). Branche `claude/m2-extraction`.

## Objectif

Séparer calcul, SGX et Excel dans une bibliothèque, avec des dossiers passés explicitement, et garder un CLI léger compatible avec le pilote de qualification. **Aucun changement de comportement.**

## Organisation obtenue

Un seul paquet Cargo, conformément à architecture.md (« commencer par `lib.rs` et une entrée CLI »). Pas de workspace ni de crate supplémentaire à ce stade.

| Fichier | Responsabilité | Dépend de |
|---|---|---|
| `src/regles.rs` | Types métier, inventaire des lanes, synthèse, correspondances retenues (R04), dry-run, contrôle des décisions (R07/R08), test séquentiel des renommages (R09/R10) | Aucun fichier, aucun Excel ; `serde_json::Value` pour les modèles |
| `src/sgx.rs` | Recherche de la source, lecture des modèles, écriture de la nouvelle archive | `regles` (types, `trouver_lanes`) |
| `src/excel.rs` | Lecture des cellules (`texte`, `entier`) et écriture des trois classeurs | `regles` (types) ; aucune décision métier |
| `src/workflow.rs` | `Chemins { input, work, output }`, `executer(&Chemins, journal)`, écriture des JSON | Les trois modules ci-dessus |
| `src/lib.rs` | Point d'entrée public : `Chemins`, `executer`, `Res`, `Journal` | — |
| `src/main.rs` | CLI historique : `Chemins::historiques("")` et affichage du journal sur la console | Bibliothèque |

Les messages de progression passent par un `journal` (fonction reçue en paramètre) au lieu de `println!`. La bibliothèque n'écrit plus sur la console, ce qui la rend utilisable depuis Tauri en M5.

Seuls `Chemins`, `executer`, `Res` et `Journal` sont publics. Les autres éléments sont internes au crate (`pub(crate)`), faute de consommateur à ce stade.

### Déplacements notables (sans changement de règle)

- La normalisation de « Validation » (`trim` puis majuscules) passe de la lecture Excel à `regles::controler_decisions` : c'est une règle (R07), pas un format de fichier.
- La sélection des correspondances (nouveau nom nettoyé, vide ignoré, dernière retenue) passe de la lecture Excel à `regles::retenir_correspondances`.
- `generer_sgx` est découpé en `sgx::charger_modeles`, `regles::tester_renommages` et `sgx::ecrire_sgx`. Les modèles sont chargés avant le test au lieu de l'être à la demande. C'est sans effet observable : un modèle validé provient toujours de l'inventaire de la même archive (`validees ⊆ analyse`).
- Les chemins affichés dans les messages restent écrits avec `/` (`{output}/inventaire_swimlanes.xlsx`), comme les chaînes historiques. Le message du SGX produit utilise toujours l'affichage natif du chemin, comme avant.

## Preuves d'exécution (Linux, Rust 1.88.0)

| Contrôle | Résultat |
|---|---|
| `cargo test --locked` | 16 tests unitaires + 23 tests CLI (M1) + 2 nouveaux tests de bibliothèque : tous réussis |
| Tests M1 | `tests/cli.rs` identique à M1 ; les 16 tests unitaires sont déplacés dans `regles.rs` et `excel.rs` avec un corps identique (comparaison automatique des 20 fonctions) |
| `cargo clippy --locked --all-targets -- -D warnings`, `cargo fmt --check` | Propres |
| `cargo build --release --locked` | Succès |
| Pilote de qualification, binaire M2 (`--binary`) | 30 scénarios identiques à M0 : codes, contenus, présentation Excel, SGX produits, attributs ZIP, stdout, stderr. Assertions de caractérisation réussies |
| Binaires M1 et M2 lancés sur des copies identiques des 30 dossiers de scénario | 229 fichiers de sortie identiques : JSON et SGX octet pour octet, Excel identiques en valeurs et en styles. Mêmes codes de sortie et mêmes messages console |

Remarque : la comparaison directe des sorties de deux lancements du pilote montre des dates d'entrée ZIP différentes. Le pilote régénère le SGX source à chaque lancement, et le programme recopie ses dates. D'où la comparaison ci-dessus sur des entrées identiques.

## Nouveaux tests (`tests/chemins.rs`)

- `dossiers_explicites_et_separes` : source, saisies et résultats dans trois dossiers distincts, sortie imbriquée créée automatiquement, aucune écriture dans les dossiers source et saisie, messages reçus par le journal.
- `erreur_nomme_le_dossier_source` : l'erreur « Aucun fichier SGX trouvé » cite le dossier fourni ; aucun dossier de sortie créé.

## Limites

- Tests exécutés sous Linux uniquement.
- Les erreurs restent des messages texte (`Box<dyn Error>`). Les résultats structurés (statut, compteurs, diagnostics) relèvent de P04, en M3/M4.
- Le CLI ne prend toujours aucun argument : le mode historique suffit au pilote. Des arguments de chemins pourront être ajoutés quand un besoin le demandera.

## Lot suivant — M3 (contrats V1)

Traiter P05 (déjà conforme, à confirmer par des attentes V1), puis P02, P03, P06 et les résultats explicites de P04, **une proposition à la fois**. Pour chacune : test avant/après, conversion des tests « caractérisation : défaut connu » concernés en attentes V1, et divergence explicite dans la qualification. Ces propositions ne sont pas encore arbitrées : un accord est nécessaire avant de changer le comportement.
