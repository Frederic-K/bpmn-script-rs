# M1 — Tests Rust de caractérisation

26 septembre 2026 · lot M1 de [implementation-plan.md](dossier-claude/implementation-plan.md), suite de [m0-etat-des-lieux.md](m0-etat-des-lieux.md). Branche `claude/m1-tests-caracterisation`.

## Objectif

Figer le comportement **actuel** du CLI par des tests Rust, défauts compris, **sans modifier le métier ni réorganiser le code**. Un test marqué `caractérisation : défaut connu, voir Pxx` décrit un comportement à revoir en M3, pas une attente V1.

## Fichiers modifiés

| Fichier | Changement |
|---|---|
| `rust-toolchain.toml` | Nouveau : Rust 1.88.0, profil minimal, composants `clippy` et `rustfmt` |
| `src/main.rs` | Ajout d'un module `#[cfg(test)] mod tests` en fin de fichier (265 lignes ajoutées, 0 ligne existante modifiée) |
| `tests/cli.rs` | Nouveau : tests d'intégration exécutant le binaire |

Aucune nouvelle dépendance : les fixtures utilisent `zip`, `rust_xlsxwriter`, `calamine` et `serde_json`, déjà présentes. `Cargo.lock` inchangé.

## Tests ajoutés

**Unitaires (16)**, sur les fonctions pures existantes : `trouver_lanes`, `renommer_lanes`, `synthetiser`, `dry_run`, `Ligne::proposition`, `texte`, `entier`.

**Intégration (23)**, chacun dans un dossier temporaire avec `input/`, `work/`, un SGX et des Excel synthétiques. Chaque exécution vérifie que la source est inchangée octet pour octet.

| Comportement | Tests | Réf. |
|---|---|---|
| Lanes imbriquées, noms vides/espaces, tâche homonyme ignorée, doublons conservés, espaces internes conservés | unitaires `inventaire_*`, `inventaire_seul_sans_correspondance` | R03, A01 |
| Modèles homonymes distingués par chemin, « Nombre de flux » = noms distincts | `synthese_*`, `inventaire_seul_*`, `dry_run_*` | R02, R05, A02 |
| Nouveau nom vide, ancien nom inconnu, dernière correspondance retenue | `analyse_sans_validation` | R04 |
| Analyse vide normale, création de `output/` | `analyse_vide_si_aucun_nom_connu`, `inventaire_seul_*` | P05, A14 |
| OUI/NON/attente, OUI normalisé, ligne absente non autorisée | `oui_non_et_attente_melanges`, `oui_normalise`, `arbitrage_partiel_*` | R07, R08, A04 |
| Altération de chacun des cinq champs → ligne ignorée, autres OUI appliqués ; occurrences en texte | `alteration_de_chaque_champ_*`, `occurrences_non_numeriques_*` | R07, R08, A05 |
| Chaîne, doublon de validation, permutation → aucun SGX ; ancien = nouveau → SGX | `renommages_en_chaine_*`, `doublon_de_validation_*`, `permutation_*`, `ancien_nom_identique_*` | R09, R10, A06 |
| Entrées non ciblées identiques, ordre des entrées, propriétés inconnues conservées | `verifier_preservation` dans les tests produisant un SGX | R11, A07 |
| Source absente, multiple, invalide ; metadata manquante | `aucune_source`, `plusieurs_sources`, `source_invalide`, `modele_sans_metadata_*` | A08 |

**Défauts figés** (à convertir en attentes V1 en M3) :

| Défaut | Tests | Réf. |
|---|---|---|
| Nom de lane non textuel ou `childShapes` mal typé ignorés sans diagnostic | `inventaire_nom_non_textuel_*`, `inventaire_child_shapes_*`, `nom_de_lane_nul_*`, `child_shapes_mal_type_*` | P03 |
| Nombres, booléens convertis en texte ; `2.0` accepté comme occurrence | `texte_convertit_*`, `ancien_nom_numerique_*`, `nouveau_nom_numerique_*` | P02 |
| Code de sortie 0 sans SGX (refus, divergence) ; ancienne sortie non distinguée | `tous_refuses_*`, `renommages_en_chaine_*`, `ancienne_sortie_*` | P04, R12 |

## Preuves d'exécution (Linux, session cloud)

| Contrôle | Résultat |
|---|---|
| `cargo test --locked` (Rust 1.88.0) | 16 + 23 tests réussis |
| `cargo clippy --locked --all-targets -- -D warnings` | Aucun avertissement |
| `cargo fmt --check` | Conforme |
| Mutations volontaires du code, sur une copie | 5 sur 5 détectées : nouveau nom non contrôlé, divergence non bloquante, renommage sans `trim`, entrée non ciblée altérée, NON appliqué. Une mutation témoin sans effet reste verte |
| Pilote de qualification avec le binaire M1 (`--binary`) | 30 scénarios identiques à M0 (codes, contenus, SGX, attributs ZIP, sorties console) ; assertions de caractérisation réussies |

## Limites

- Non testés ici, toujours couverts par le pilote Python : formules Excel (avec ou sans cache), attributs ZIP (dates, commentaires, champs supplémentaires), présentation Excel (largeurs, couleurs, liste OUI/NON).
- Tests exécutés sous Linux uniquement. Pas encore exécutés sous Windows.
- SAPHIR non rejoué (fichier non fourni). Aucun import Signavio.
- Le message trompeur « + SAPHIR » de `check_qualification.py` n'a pas été modifié : l'archive de référence reste intacte.

## Lot suivant — M2 (extraction)

Extraire une bibliothèque (`lib.rs`) avec calcul, SGX et Excel séparés, et des chemins explicites à la place de `input/`, `work/`, `output/` codés en dur. Garder un CLI compatible avec le pilote. Les tests M1 doivent rester verts sans modification de leurs attentes, et la qualification doit rester identique à M0.
