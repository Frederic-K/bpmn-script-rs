# M0 — État des lieux du dépôt `bpmn-script-rs`

26 septembre 2026 · lot M0 de [implementation-plan.md](dossier-claude/implementation-plan.md). Aucun code, dépendance, verrou ni donnée de référence modifié.

Légende : **OBSERVÉ** = constaté par exécution dans cette session ; **REPRIS** = mesure consignée dans le dossier, non rejouée ici ; **PROPOSÉ** = recommandation, non arbitrée.

## 1. État du dépôt

| Point | Constat (OBSERVÉ) |
|---|---|
| Branche de travail | `claude/modest-goldberg-ds5id5`, arbre propre, aucun changement local à préserver |
| Commit de départ | `0c57915` (ajout de `docs/dossier-claude/`) ; `main` au même commit |
| Écart avec `3c846fc` | Uniquement `docs/dossier-claude/` (9 fichiers ajoutés). `src/main.rs`, `Cargo.toml`, `Cargo.lock`, `README.md`, `.gitignore` inchangés |
| Écart avec la référence du paquet | `bpmn-script-rs-main/` du ZIP de qualification identique octet pour octet au dépôt (5 fichiers). Seul ajout du paquet : `rust-toolchain.toml` (1.88.0), absent du dépôt |
| Intégrité du dossier | 8 empreintes de `docs/dossier-claude/SHA256.json` conformes ; 33 empreintes internes du ZIP conformes |
| Consignes applicables | Pas de `CLAUDE.md` ni `AGENTS.md` à la racine. Consignes : `docs/dossier-claude/CLAUDE.md` et la demande utilisateur (M0 seul) |
| Contenu du code | Un CLI unique `src/main.rs` (604 lignes), chemins relatifs fixes `input/`, `work/`, `output/`, aucun test Rust, aucun module |
| Chantier Python pédagogique | Hors dépôt ; présent seulement dans le paquet de qualification, utilisé en lecture pour la comparaison |

## 2. Environnement et versions (OBSERVÉ)

Session cloud Linux x86_64 (le dossier et la qualification d'origine sont sous Windows).

| Outil | Version | Usage |
|---|---|---|
| Rust | 1.88.0 (`rustc 6b00bc388`, cargo 1.88.0), profil minimal, `RUSTUP_HOME`/`CARGO_HOME` isolés dans un dossier temporaire | Compilation de référence |
| Rust | 1.94.1 (toolchain par défaut de l'environnement, non modifiée) | `clippy` et `rustfmt` uniquement, absents du profil minimal 1.88.0 |
| Python | 3.11.15, venv isolé, openpyxl 3.1.5, et_xmlfile 2.0.0 | Pilote de qualification et programme Python de référence |
| Dépendances Cargo | Résolues par `--locked` sur le `Cargo.lock` existant (calamine 0.36.1, rust_xlsxwriter 0.99.1, zip 8.6.0, serde_json 1.0.151…) | Aucune mise à jour |

Remarque : le code utilise les *let chains* (`if … && let …`) en édition 2024, stabilisées en Rust 1.88. **1.88.0 est donc la version minimale**, pas seulement la version testée.

## 3. Compilation et contrôles statiques (OBSERVÉ)

Cible de compilation placée hors du dépôt (`CARGO_TARGET_DIR`) ; `git status` vide après toutes les commandes.

| Commande | Résultat |
|---|---|
| `cargo +1.88.0 build --locked` (référence du paquet) | Succès |
| `cargo +1.88.0 build --locked` (dépôt courant) | Succès |
| `cargo +1.88.0 build --release --locked` (dépôt courant) | Succès — compilation seulement, pas de packaging |
| `cargo +1.88.0 test --locked` | Succès, **0 test** (aucun test existant) |
| `cargo +1.94.1 build --locked` | Succès |
| `cargo +1.94.1 clippy --locked` | Aucun avertissement |
| `cargo fmt --check` | Conforme |

## 4. Qualification sur copies (OBSERVÉ)

Le paquet a été décompressé deux fois dans des dossiers temporaires distincts ; les sources Python/Rust du paquet et les données du dépôt n'ont pas été modifiées.

| Exécution | Binaire Rust testé | `qualify_rust.py` | `check_qualification.py` |
|---|---|---|---|
| A — référence incluse | `bpmn-script-rs-main/…/target/debug/bpmn-script-rs` compilé depuis le paquet | 30 scénarios exécutés | Succès |
| B — **dépôt courant** | binaire compilé depuis `/home/user/bpmn-script-rs` (1.88.0), passé par `--binary` | 30 scénarios exécutés | Succès |

Le binaire par défaut du pilote est un `.exe` Windows : `--binary` a été fourni dans les deux cas. Les sorties console et d'erreur de A et B sont identiques pour les 30 scénarios (sources identiques).

**Résultats B (code courant)** :

- 22 scénarios concordent avec Python sur les contenus métier et codes de sortie ;
- 8 écarts, exactement la liste attendue : `empty_mapping`, `unknown_mapping`, `missing_output`, `numeric_old`, `numeric_new`, `formula_new`, `null_name`, `bad_children` ;
- SGX produit dans les 9 scénarios attendus, ancienne sortie inchangée dans `old_output` ;
- source intacte dans les 60 exécutions (Python et Rust) ;
- entrées non concernées préservées dans chaque SGX produit ; commentaire global ZIP non conservé (défaut connu) ;
- différences de présentation Excel sur tous les classeurs, comme dans la mesure consignée ; relevé `excel-presentation-details.json` identique à celui du paquet.

**SAPHIR : non rejoué.** Le SGX n'est pas fourni (et ne doit pas être versionné). Attention : `check_qualification.py` affiche « 30 synthetic scenarios + SAPHIR in 3 stages » même sans scénario SAPHIR, car il ne vérifie pas sa présence. Les chiffres SAPHIR (241 modèles, 12 noms, 13 propositions) restent **REPRIS** du paquet.

**Inspection visuelle des classeurs** : non réalisée (pas d'Excel dans cet environnement). Les comparaisons de cellules ne la remplacent pas.

## 5. Divergences avec les mesures consignées

Comparaison champ par champ entre `rust-qualification-results.json` du paquet (Windows) et l'exécution B :

| Divergence | Explication |
|---|---|
| Messages console : `output\export_modifie.sgx` → `output/export_modifie.sgx` | Séparateur de chemin de la plateforme (`PathBuf::display`). Aucune autre différence de sortie console une fois le séparateur normalisé |
| `zip_attributes` : sur l'entrée modifiée, `create_system` change en plus de `external_attr`, `extra`, `comment` | La fixture fixe `create_system = 0` (DOS). La crate `zip` écrit le système de la plateforme de compilation pour une entrée réécrite : DOS sous Windows (donc inchangé dans la mesure consignée), Unix (3) sous Linux avec `external_attr = 0o100644 << 16`. Les assertions restent vérifiées (`{extra, comment}` inclus). **Conséquence :** les attributs ZIP produits dépendent de la plateforme de build ; à couvrir par un test dédié (P06) et à qualifier sur le binaire Windows final |

Aucune autre divergence : codes, présences de fichiers, contenus, SGX produits, contrôle d'archive et présentation identiques à la mesure consignée.

## 6. Observations sur le code utiles pour la suite (OBSERVÉ, non corrigées)

- `generer_sgx` retourne un succès (code 0) sans créer de SGX en cas de décompte divergent ou sans décision admise : résultat ambigu (P04, R12).
- Les JSON de modèle illisibles interrompent tout le traitement, alors qu'un `childShapes` non tableau ou un nom non textuel sont ignorés silencieusement (`null_name`, `bad_children`) (P03).
- `texte()` convertit toute cellule non vide en texte (nombre, booléen, erreur) ; `entier()` accepte les flottants entiers (P02).
- Le SGX final est écrit directement sous son nom définitif (`File::create`), sans fichier temporaire (P04).
- L'ordre des colonnes Excel lues est codé en dur, sans contrôle des en-têtes ni de la présence de la feuille autre que l'erreur calamine (P02).

## 7. Contenu proposé pour M1 — tests Rust de caractérisation (PROPOSÉ)

Objectif : figer le comportement **actuel**, défauts compris, sans modifier le métier ni réorganiser les fichiers.

1. **Épingler l'outillage** : ajouter `rust-toolchain.toml` (channel 1.88.0, composants `clippy`, `rustfmt`), identique à celui du paquet hormis les composants. Seul changement hors tests.
2. **Tests unitaires** dans un module `#[cfg(test)]` de `src/main.rs`, sur les fonctions pures déjà existantes, sans les déplacer : `trouver_lanes` (lanes imbriquées, noms vides/espaces, tâche homonyme non-Lane, doublons conservés, nom `null`, `childShapes` non tableau), `renommer_lanes` (retour du compte, comparaison après `trim`), `synthetiser` (modèles homonymes distincts par chemin, « Nombre de flux » = noms distincts), `dry_run` (ancien nom inconnu, dernière correspondance retenue), `Ligne::proposition`, `texte`/`entier` (valeurs actuelles, y compris coercitions).
3. **Tests d'intégration** `tests/cli.rs` exécutant le binaire (`CARGO_BIN_EXE_bpmn-script-rs`) dans des dossiers temporaires uniques, avec fixtures synthétiques générées par les crates déjà présentes (`zip`, `rust_xlsxwriter` ; aucune nouvelle dépendance) :
   - OUI/NON/attente mélangés, arbitrage partiel (ligne supprimée), OUI normalisé ;
   - altération de chacun des cinq champs (chemin, flux, ancien nom, nouveau nom, occurrences) → ligne ignorée, autres OUI appliqués ;
   - chaîne A→B/B→C et doublon de validation → aucun SGX, code 0 (défaut caractérisé) ;
   - intégrité de la source (empreinte avant/après), entrées non ciblées identiques, ordre des entrées ;
   - ancienne sortie présente et refus total → ancienne sortie inchangée.
4. Chaque test caractérisant un défaut porte un commentaire `// caractérisation : défaut connu, voir Pxx` pour être converti en attente V1 en M3.
5. Hors M1 : pas d'extraction `lib.rs` (M2), pas de correction P02–P06 (M3), pas de modification du pilote Python ni de ses attentes. Un correctif du message SAPHIR trompeur de `check_qualification.py` ne se ferait qu'avec accord, dans une copie de travail, jamais dans l'archive de référence.

Preuve de sortie attendue : `cargo test --locked` vert en 1.88.0, clippy/fmt propres, qualification B relancée sans changement.
