# M4 — Dossier de traitement

26 septembre 2026 · lot M4 de [implementation-plan.md](dossier-claude/implementation-plan.md), suite de [m3-contrats-v1.md](m3-contrats-v1.md). Branche `claude/m4-traitement`. P01 et P04 approuvées par l'utilisateur.

## Objectif

Permettre de fermer l'application et de reprendre un traitement plus tard, sans base de données, sans dépendre de l'emplacement du SGX d'origine, et sans qu'une donnée modifiée entre-temps passe inaperçue. C'est le moteur que l'interface Tauri appellera en M5.

## Dossier de traitement

```text
<nom> - traitement/
  traitement.json         manifeste versionné (schéma 1) : état, révision, empreintes SHA-256
  traitement.verrou       verrou système exclusif
  source/<nom>.sgx        copie de la source, contrôlée par empreinte
  inventaire/             inventaire produit à la création
  edition/                classeurs que l'utilisateur ouvre dans Excel
  edition/precedents/     classeurs d'édition modifiés puis remplacés (jamais perdus)
  entrees/                instantanés adoptés : correspondances-rNNN.xlsx, decisions-rNNN.xlsx
  analyse/                analyse des correspondances adoptées
  controle/               dernier contrôle des décisions adoptées
  sorties/tentative-NNN/  rapports, bilan.json et éventuel SGX d'une tentative
```

## API (`src/traitement.rs`)

| Opération | Effet | Étape obtenue |
|---|---|---|
| `Traitement::creer(source, dossier_parent, journal)` | Crée `<nom> - traitement`, ou `… (2)` si le nom existe ; copie la source, inventorie, prépare le classeur de correspondance. En cas d'échec (P03), le dossier créé est supprimé | InventairePret |
| `Traitement::ouvrir(dossier)` | Vérifie la version du manifeste, les chemins internes, la copie source et les instantanés | Étape enregistrée |
| `etat()` | Vue complète pour l'interface : étape, compteurs, classeurs à relire, tentatives et état de leur SGX, tentatives interrompues | — |
| `adopter_correspondances(revision, fichier?, journal)` | Lit le classeur d'édition ou un fichier importé, contrôle P02, crée un instantané ; l'analyse et les décisions deviennent obsolètes | CorrespondancesPretes |
| `preparer_analyse(revision, journal)` | Recalcule l'analyse depuis la copie source et l'instantané, et crée le classeur de décision | AnalysePrete |
| `adopter_decisions(revision, fichier?, journal)` | Lit, contrôle (R07/R08, P02) et recompte en mémoire (R09/R10), puis crée un instantané | DecisionsControlees |
| `produire(revision, journal)` | Recalcule tout depuis les instantanés vérifiés, crée `sorties/tentative-NNN`, écrit, vérifie et publie le SGX (M3), puis enregistre la tentative | ResultatProduit si un SGX est produit |

Chaque opération retourne un `Bilan` (compteurs et statut) ou une `Erreur` à code stable.

## Garanties

| Garantie | Mise en œuvre | Critère |
|---|---|---|
| Reprise sans l'original | La copie interne fait référence ; le chemin d'origine est seulement informatif | A11 |
| Copie ou instantané altéré ou supprimé | Vérifié à l'ouverture et avant chaque opération : erreur `traitement_altere` avec la liste des fichiers | A11, A12 |
| Classeur modifié mais non relu | L'empreinte de référence est comparée à chaque `etat()` ; la production est refusée (`classeur_modifie`) | A12, ui-spec |
| Correspondances changées | Analyse et décisions invalidées ; un classeur de décision modifié et non adopté est déplacé dans `edition/precedents/` | A12 |
| Import invalide ou illisible | Rien n'est adopté, la révision reste inchangée, l'adoption précédente est conservée ; le fichier fourni n'est jamais modifié | A14 |
| Double clic, fenêtre obsolète | Chaque opération reçoit la révision attendue : `revision_obsolete` | A13 |
| Deux fenêtres ou deux processus | Verrou système (`File::try_lock`), libéré automatiquement même en cas d'arrêt brutal : `traitement_deja_ouvert` | A13 |
| Ancienne sortie | Chaque production a son dossier de tentative ; `courante` n'est vraie que pour une tentative portant sur les entrées actuelles | A13, R12 |
| SGX produit supprimé ou modifié | `etat_sgx` : Disponible, Absent ou Modifie, recalculé par empreinte | A13 |
| Arrêt pendant une production | Dossier de tentative non enregistré : listé dans `tentatives_interrompues`, jamais réutilisé, jamais promu | A13 |
| Manifeste enregistré après publication du SGX | En cas d'échec : `etat_non_confirme`, sans faux succès | A13 |
| Manifeste inconnu | `traitement_version_inconnue`, sans réécriture | ui-spec |
| Chemins du manifeste | Relatifs uniquement : un chemin absolu ou contenant `..` est refusé | architecture |
| Écritures internes | Fichier temporaire, synchronisation sur disque, puis renommage | P04 |

Le moteur refait tous les contrôles à la production, même si l'interface ne propose pas l'action : aucune décision admissible, contrôle bloquant, classeur à relire, étape manquante.

## Outillage

- **Rust 1.89.0** (au lieu de 1.88.0), pour `File::try_lock` de la bibliothèque standard : un verrou système fiable, sans dépendance. Version fixée dans `rust-toolchain.toml`.
- **`sha2` 0.10.9** : empreintes SHA-256 de la copie source, des instantanés, des classeurs d'édition et des SGX produits. `Cargo.lock` : ajouts uniquement (sha2 et ses 8 dépendances), aucune version existante modifiée.

## Preuves d'exécution (Linux)

| Contrôle | Résultat |
|---|---|
| `cargo test --locked` | 26 tests unitaires, 2 de bibliothèque, 33 CLI, **18 de traitement** : tous réussis |
| `cargo clippy --locked --all-targets -- -D warnings`, `cargo fmt --check` | Propres |
| Pilote de qualification, binaire M4 | 30 scénarios identiques à M3 (codes, fichiers, SGX, attributs ZIP, console) : le découpage de `workflow.rs` n'a rien changé au CLI |

## Limites

- Tests sous Linux uniquement. Le renommage qui remplace un fichier (manifeste) et le verrou doivent être revérifiés sous Windows en M6.
- Pas de nettoyage automatique des tentatives, conformément à la spécification.
- Les erreurs d'écriture sont couvertes par les chemins d'erreur du code (temporaire supprimé, manifeste inchangé), sans simulation de disque plein.

## Lot suivant — M5 (interface)

Adaptateur Tauri : une commande par opération ci-dessus, avec l'état `Traitement` gardé dans un `Mutex`. Écrans Svelte selon ui-spec.md. **La maquette de l'interface est à valider par l'utilisateur avant le développement.**
