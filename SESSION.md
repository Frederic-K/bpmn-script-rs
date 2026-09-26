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
| Revue Codex de `623fb56` | D01 à D08 corrigés sur la branche `claude/corrections-revue` (voir ci-dessous) ; architecture conservée | Revue indépendante |
| Installateur | NSIS, installation par utilisateur (sans droits admin), WebView2 téléchargé seulement s'il manque | M6 |

## Défauts trouvés et corrigés

1. **Défaut du portage d'origine.** Chaque forme sans `childShapes` recevait `"childShapes": null` dans les modèles réécrits, à cause de l'indexation mutable de `serde_json`. Ce défaut a été détecté par la vérification du SGX produit (M3). Correction : `regles::enfants_modifiables` ; test `renommage_n_ajoute_aucune_cle`.
2. **Conversions silencieuses.** Nombres, booléens et formules étaient lus comme du texte (P02), et les lanes mal typées étaient ignorées (P03). Ces entrées sont désormais refusées avec un diagnostic.
3. **Code 0 sans SGX.** Le CLI renvoyait 0 alors qu'aucun SGX n'était produit, et une ancienne sortie pouvait être écrasée (P04). Il renvoie maintenant 2, et une sortie existante n'est jamais écrasée.
4. **Attributs ZIP.** Les commentaires et les champs supplémentaires de l'entrée réécrite étaient perdus, ainsi que le commentaire d'archive (P06). Ils sont maintenant conservés.

### Corrections issues de la revue Codex (commit `623fb56`)

| Constat | Correction | Preuve |
|---|---|---|
| D01 (P1) : « Enregistrer une copie » pouvait remplacer un fichier existant, y compris le SGX original | `copier_sans_ecraser` (`src-tauri/src/lib.rs`) : destination créée exclusivement (`create_new`), copie interrompue supprimée | Test `copie_sans_ecrasement` : fichier existant, source par trois alias, dossier absent |
| D02 (P1) : relire des correspondances identiques (ou redemander l'analyse) remplaçait le bilan des décisions par un bilan partiel | `adopter_correspondances` et `preparer_analyse` conservent le dernier bilan quand rien ne change | Test `operations_repetees_conservent_le_dernier_bilan` (échoue sans la correction : `DecisionsAttendues, 0` au lieu de `ProductionPossible, 2`) ; bout en bout |
| D03 : compteurs d'un contrôle obsolète présentés comme actuels | « Dernier contrôle — à actualiser » (Décisions), « à actualiser » (bilan latéral), avertissement sur l'écran Résultat | Bout en bout |
| D04 : Analyse accessible avant sa préparation, affichée comme « aucun changement » | Étape accessible seulement si `analyse_preparee` ; l'écran distingue « non préparée » et « vide » | Bout en bout |
| D05 : textes dispersés dans les composants | Catalogue `ui/src/textes.js` par écran | `npm run build` ; bout en bout (libellés inchangés) |
| D06 : « occurrences renommées », « modèles modifiés » | « occurrences traitées », « modèles concernés par la production » | Relecture |
| D07 : cause trop précise pour une analyse vide | « Aucun changement proposé. Vérifiez les nouveaux noms renseignés. » | Relecture |
| D08 : Résultat inaccessible avec une seule tentative interrompue | Étape accessible dès qu'une tentative, même interrompue, existe | Bout en bout |

Règles de renommage, contrôles et attentes V1 inchangés.

## Preuves

| Contrôle | Résultat |
|---|---|
| `cargo test --workspace --locked` | 83 tests : 26 unitaires, 2 de bibliothèque, 33 CLI, 20 de traitement, 2 de l'adaptateur. Sous Linux ; sous Windows pour les 81 d'avant la revue (les 83 le seront par l'intégration continue Windows) |
| Clippy (`-D warnings`), rustfmt, `npm run build` | Propres, sans avertissement |
| Qualification V1 (30 scénarios comparés au Python d'origine) | Conforme sous Linux et avec le binaire Windows (`tests/qualification/verifier_v1.py`) |
| Test de bout en bout de la vraie application (Linux, WebKitGTK) | 23 contrôles réussis après les corrections de la revue, dont le double clic sur « Générer » (une seule tentative), le classeur modifié qui suspend la génération, et les transitions D02, D03, D04, D08 |
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
