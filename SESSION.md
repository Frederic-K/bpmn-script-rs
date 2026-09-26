# Session de développement — état au 26 septembre 2026

Développement réalisé par Claude (Claude Code, session cloud Linux), à partir du dossier [docs/dossier-claude/](docs/dossier-claude/). Consignes permanentes : [AGENTS.md](AGENTS.md).

## Où en est le projet

`main` (commit `737867a`) contient les lots M0 à M6, un commit par lot, historique linéaire.

| Lot | Commit | Livré | Compte rendu |
|---|---|---|---|
| M0 | `d07db58` | État des lieux, reproduction de la compilation et des 30 scénarios | [docs/m0-etat-des-lieux.md](docs/m0-etat-des-lieux.md) |
| M1 | `44f7335` | Tests de caractérisation du comportement d'origine | [docs/m1-tests-caracterisation.md](docs/m1-tests-caracterisation.md) |
| M2 | `be68f2d` | Bibliothèque extraite, CLI inchangé (229 fichiers de sortie identiques) | [docs/m2-extraction.md](docs/m2-extraction.md) |
| M3 | `a675f99` | Contrats V1 P02–P06, vérification du SGX produit | [docs/m3-contrats-v1.md](docs/m3-contrats-v1.md) |
| M4 | `6fb7f24` | Dossier de traitement P01/P04 | [docs/m4-traitement.md](docs/m4-traitement.md) |
| M5 | `99f64f4` | Interface Svelte 5 (JavaScript) et application Tauri 2 | [docs/m5-interface.md](docs/m5-interface.md) |
| M6 | `bd6d434`, `737867a` | Qualification Windows automatique, procédure opérateur | [docs/m6-qualification.md](docs/m6-qualification.md) |

**Reste à faire, par l'opérateur :**
- A16 : installation et parcours complet sur le poste cible ;
- essai sur une copie de SAPHIR ;
- A17 : import dans un emplacement Signavio de test.

La procédure et le tableau à remplir sont dans [docs/m6-qualification.md](docs/m6-qualification.md).

## Décisions prises

| Sujet | Décision | Origine |
|---|---|---|
| P01 à P07 | Toutes adoptées | Propriétaire : « tu peux tout approuver sur la partie dev » |
| Interface | Svelte 5 en JavaScript, sans TypeScript, sans store ni mémoïsation | Consigne du propriétaire |
| Styles | Tailwind CSS v4 (4.3.3, plugin Vite), à la place d'un fichier CSS classique ; rendu identique à la maquette validée ; éléments répétés en composants (`Bouton`, `Pastille`, `Chiffre`, `Tableau`) plutôt qu'en `@apply` | Demande du propriétaire, après M6 |
| Thème | Bascule clair/sombre dans la barre du haut, mémorisée ; thème du système au premier lancement | Demande du propriétaire, après M6 |
| Maquette | Validée avant le développement de M5 | Propriétaire |
| Rust | 1.88 → 1.89, pour `File::try_lock` (verrou système libéré même en cas d'arrêt brutal) | M4 |
| Dépendances ajoutées | `sha2` (empreintes) ; Tauri 2 et ses deux plugins ; Svelte, Vite, @tauri-apps/api | M4, M5 |
| Fonctionnalités activées | `serde_json/arbitrary_precision`, `zip/unreserved` | M3 (P06) |
| Code de sortie du CLI | 0 = étape terminée, 1 = erreur, **2 = décisions fournies sans SGX produit** (auparavant 0) | M3 (P04) |
| Anomalie de modèle | Bloque tout le traitement (pas d'inventaire partiel) | M3 (P03) |
| Dialogues | Côté Rust, ouverts dans Documents ; l'interface n'a aucune permission de fichier ni de shell | M5 |
| Installateur | NSIS, installation par utilisateur (sans droits admin), WebView2 téléchargé seulement s'il manque | M6 |

## Défauts trouvés et corrigés

1. **Défaut du portage d'origine.** Chaque forme sans `childShapes` recevait `"childShapes": null` dans les modèles réécrits, à cause de l'indexation mutable de `serde_json`. Ce défaut a été détecté par la vérification du SGX produit (M3). Correction : `regles::enfants_modifiables` ; test `renommage_n_ajoute_aucune_cle`.
2. **Conversions silencieuses.** Nombres, booléens et formules étaient lus comme du texte (P02), et les lanes mal typées étaient ignorées (P03). Ces entrées sont désormais refusées avec un diagnostic.
3. **Code 0 sans SGX.** Le CLI renvoyait 0 alors qu'aucun SGX n'était produit, et une ancienne sortie pouvait être écrasée (P04). Il renvoie maintenant 2, et une sortie existante n'est jamais écrasée.
4. **Attributs ZIP.** Les commentaires et les champs supplémentaires de l'entrée réécrite étaient perdus, ainsi que le commentaire d'archive (P06). Ils sont maintenant conservés.

## Preuves

| Contrôle | Résultat |
|---|---|
| `cargo test --workspace --locked` | 81 tests : 26 unitaires, 2 de bibliothèque, 33 CLI, 19 de traitement, 1 de l'adaptateur, **sous Linux et sous Windows** |
| Clippy (`-D warnings`), rustfmt, `npm run build` | Propres, sans avertissement |
| Qualification V1 (30 scénarios comparés au Python d'origine) | Conforme sous Linux et avec le binaire Windows (`tests/qualification/verifier_v1.py`) |
| Test de bout en bout de la vraie application (Linux, WebKitGTK) | 3 exécutions sur 3, 14 contrôles chacune, dont le double clic sur « Générer » (une seule tentative) et le classeur modifié qui suspend la génération |
| Windows (GitHub Actions) | Tests, installateur, qualification et lancement réussis : https://github.com/Frederic-K/bpmn-script-rs/actions/runs/36253157042 |
| Mutations volontaires du code (M1) | 5 sur 5 détectées par les tests |

**Non vérifié :**
- le poste cible (A16) ;
- SAPHIR, qui n'a pas été rejoué ;
- l'import Signavio (A17) ;
- l'ouverture réelle d'Excel ;
- le lecteur d'écran et l'agrandissement du texte ;
- l'usage depuis une clé USB.

## Guide de revue (pour Codex ou un relecteur)

À lire dans cet ordre, du plus critique au moins critique :

1. **`src/sgx.rs`**
   - `produire_sgx` : écriture dans un temporaire, vérification, contrôle d'existence, renommage.
   - `verifier_sgx` : même liste d'entrées, entrées non modifiées identiques octet pour octet, modèles modifiés vérifiés un par un.
   - `ecrire_sgx` : conservation des attributs.

   Question clé : **un SGX qui diffère de la source au-delà des renommages validés peut-il être publié ?**
2. **`src/regles.rs`**
   - `controler_decisions` : types stricts, égalité sur cinq champs, catégories de résultat.
   - `tester_renommages` et `verifier_modele`, avec `appliquer_renommages` : deux calculs distincts, l'un séquentiel, l'autre en une passe.
   - `trouver_lanes` : anomalies.
3. **`src/excel.rs`** : `lire_feuille` (détection des formules, contrôle des en-têtes, lignes vides).
4. **`src/traitement.rs`**
   - `verifier_integrite` : empreintes contrôlées.
   - `chemin_interne` : aucun chemin ne sort du dossier du traitement.
   - `produire` : recalcule tout à partir des instantanés.
   - `remplacer_edition` et `archiver_edition` : aucune perte d'une saisie de l'utilisateur.
   - `verrouiller` et l'enregistrement du manifeste.
5. **`src-tauri/src/lib.rs`**
   - `verifier_appartenance` : aucun fichier hors du traitement ne peut être ouvert.
   - `try_lock` : une seule opération à la fois.
6. **`ui/src/App.svelte`** (`executer`) et **`EtapeDecisions.svelte`** (`generationPossible`) : aucune décision n'est prise côté interface.

Points volontairement discutables à challenger :
- **Comparaison des JSON sans l'ordre des clés** (`verifier_modele`) : les clés ne sont jamais réordonnées (`preserve_order`), mais un réordonnancement ne serait pas détecté.
- **Attributs non conservés** sur les entrées ZIP non modifiées (`external_attr`, champs supplémentaires) : c'est une limite de la bibliothèque `zip`, documentée, dont l'effet dépend du résultat de l'import Signavio.
- **Anomalie bloquante :** une seule anomalie dans un modèle bloque tout le traitement. C'est plus sûr, mais contraignant si un export réel contient une anomalie tolérable.
- **Code de sortie 2** du CLI : changement de contrat par rapport à l'ancien programme.

## Environnement de cette session (pour reproduire)

- Sous Linux, Rust et Python étaient isolés dans un dossier temporaire ; `webkit2gtk-4.1`, `webkit2gtk-driver`, `xvfb` et `xdotool` ont été installés dans le conteneur.
- Le paquet de qualification a toujours été décompressé hors du dépôt.
- Aucune donnée réelle n'a été manipulée.
