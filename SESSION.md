# Session de développement — état au 2 octobre 2026

Développement réalisé par Claude (Claude Code : session cloud Linux jusqu'au lot M6 et à la colonne F, puis session locale Windows), à partir du dossier [docs/dossier-claude/](docs/dossier-claude/). Consignes permanentes : [AGENTS.md](AGENTS.md).

## Où en est le projet

`main` contient les lots M0 à M6 (un commit par lot, historique linéaire), puis les corrections des revues Codex, l'évolution de la colonne F et la révision de l'interface d'octobre (voir ci-dessous). Dernière fusion : `9ea8b9d`.

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
| Décisions contradictoires | OUI et NON sur la même proposition bloquent la production (évolution V1, `docs/m3-contrats-v1.md`) | Revue du parcours ; propriétaire : « pas de faille liée à une mauvaise manipulation » |
| Parcours des décisions | Un seul classeur de décision ; deux voies sur l'écran Décisions (valider soi-même, faire valider) ; le classeur d'analyse n'est plus proposé à l'ouverture | Revue du parcours |
| Installateur | NSIS, installation par utilisateur (sans droits admin), WebView2 téléchargé seulement s'il manque | M6 |
| Colonne F de l'inventaire | « Répétitions dans un même modèle », par chemin interne, à la place de « Flux avec occurrences multiples » ; `occurrences_par_flux` supprimé ; aucune colonne G ; information non bloquante (évolution V1, `docs/m3-contrats-v1.md`) | Propriétaire : l'application harmonise les noms, elle ne recherche pas les modèles homonymes |
| Écran Décisions | Tableau des lignes non admises retiré : compteurs seuls, message « n OUI ignoré(s) … Détail dans le rapport de contrôle », libellé du contrôle conforme détaillé (« les décisions admises correspondent à l'analyse », pour ne pas contredire le message des OUI ignorés) ; `Bilan.lignes_a_examiner` et `LigneExaminee` supprimés (un manifeste qui les contient reste lisible). Écart assumé avec `ui-spec.md` (« une décision invalide montre son motif ») : le motif est dans le rapport de contrôle | Propriétaire, retour d'un traitement réel : tableau non filtrable, masquant les lignes VALIDÉE, redondant avec le rapport Excel |
| Révision de l'interface (document « Révision_UI ») | 1. Bouton principal = action suivante : « Ouvrir le classeur » tant que le classeur n'est ni modifié ni lu, puis « Lire… » une fois modifié. 2. Classeurs écrits : retour à la ligne et centrage vertical pour toutes les cellules (écart de présentation avec le Python, non contrôlé par `verifier_v1.py`). 3. Écran Analyse : compteurs seuls, détail dans le classeur de décision ; composant `Tableau` supprimé. Écart assumé avec `ui-spec.md` (aperçu flux, noms, occurrences, chemin). 4. Alerte « À relire » conservée, reformulée comme étape suivante | Propriétaire, après un traitement réel |
| Disposition | Étapes en barre horizontale sous l'en-tête ; bilan en barre d'état fixe en bas de fenêtre ; une seule colonne de contenu à toutes les largeurs, limitée à 64 rem (plus de point de rupture à 980 px) ; fenêtre ouverte à 900 × 800 (auparavant 1200 × 800) ; bouton de thème en icône, lune ou soleil selon le thème (libellé « Thème sombre » conservé pour l'accessibilité et en info-bulle). Écart assumé avec `ui-spec.md` (« navigation d'étapes à gauche ») ; le « bilan contextuel dans le même écran » est conservé | Propriétaire, après le retrait des tableaux et l'exercice de maquettes (canevas de design, hors dépôt) |

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

### Revue du parcours et comparaison avec Python (commit `abe5311`)

| Constat | Correction | Preuve |
|---|---|---|
| P1 : OUI et NON sur la même proposition autorisaient la modification (hérité du Python) | Contradiction détectée dans `regles::controler_decisions` ; statut bloquant, lignes citées ; OUI répété toujours bloqué par le recomptage (R09) | Tests unitaire, CLI et traitement ; bout en bout ; 30 scénarios V1 inchangés |
| P2 : OUI saisis dans le classeur d'analyse au lieu du classeur de décision | Le classeur d'analyse n'est plus proposé à l'ouverture ; écran Décisions en deux voies : « Je valide moi-même », « Je fais valider par une autre personne » (copie à transmettre, import du retour) | Bout en bout |
| P2 : provenance des décisions ambiguë (retour modifié après import, retours successifs, import refusé) | `Etat.decisions_lues` et `correspondances_lues` : fichier lu, date, classeur du traitement ou import ; messages « Décisions lues/importées le … », « Classeur non adopté … restent utilisées », « remplace … aucune fusion » | Tests de contrat (traitement) ; bout en bout |
| R1 : copie enregistrée écrite directement sous son nom final | `fichiers::copier_sans_ecraser` : temporaire `.copie-en-cours` complet, synchronisé, relu, puis publication par lien physique | Tests : destination existante ou apparue entre-temps, alias de la source, erreur après création, temporaire resté d'une interruption |
| R2 : analyse obsolète non signalée | « Dernière analyse — à actualiser » (écran, étape, bilan latéral) ; passage aux décisions suspendu | Bout en bout |
| R3 : cache Python versionné | Retiré, ignoré | — |

Constats de ma propre relecture, corrigés au passage :
- `sgx::produire_sgx` publiait le SGX par `exists()` puis `rename`. Sous Windows, `rename` remplace un fichier existant. La publication passe maintenant par le même `publier_sans_ecraser`.
- L'étape Décisions était cochée « terminée » même sur un contrôle bloquant. Elle ne l'est plus que si la génération est possible.
- Composant `Fichier` : dans une colonne étroite, le nom devenait illisible (une lettre par ligne). Les actions passent maintenant à la ligne.

### Contre-revue de `d3c57af` et revue approfondie de `f7cbcae` (F01–F04)

| Constat | Correction | Preuve |
|---|---|---|
| F01 : un retour valide dont l'adoption échoue tard (verrou, écriture) laissait son rapport comme rapport courant, alors que les décisions précédentes restaient utilisées | Un rapport par adoption (`controle/rNNN/`), désigné par le manifeste seulement une fois l'adoption enregistrée | `adoption_echouee_ne_change_pas_le_rapport_courant` (échoue sur l'ancien code) |
| F02 : la copie d'un SGX ne vérifiait pas son empreinte de production | `Traitement::empreinte_sgx` ; `copier_sans_ecraser` compare la copie temporaire à cette empreinte avant publication | `copie_d_un_sgx_modifie_refusee`, `copie_refusee_si_l_empreinte_differe` |
| F03 : repli par copie exclusive, sans garantie de fichier final complet, déclenché par toute erreur | Repli supprimé : emplacement sans liens physiques refusé avec une consigne (disque local). Commentaires et guide alignés | Tests de publication ; essai clé USB ajouté à m6 (étape 7) |
| F04 : provenance déduite de la fin du chemin | `Adoption.importe` enregistré à l'adoption | `import_du_classeur_d_un_autre_traitement_reste_un_import` |
| Point 3 : contradiction absente du rapport Excel | Lignes `CONTRADICTOIRE`, motif avec la ligne opposée ; proposition ni admise ni refusée | Tests unitaire et CLI ; bout en bout |
| Point 4 : nom de la copie à transmettre | Nom proposé `validation_<traitement>.xlsx` (seul le nom suggéré change) | — |

Ensuite, à la demande du propriétaire : bouton « Enregistrer une copie » aussi à l'étape Correspondances (classeur tel qu'il est, nom proposé `correspondances_<traitement>.xlsx`). Libellés harmonisés, car la copie ne sert pas qu'à transmettre (réunion, arbitrage) : « Enregistrer une copie » aux trois étapes, « Importer un classeur de correspondances / de décision », voie « Je fais valider ailleurs (autre personne, réunion…) ».

Non retenu, comme recommandé : blocage des doublons de correspondances à cibles différentes (point 7, décision métier à prendre, attentes V1 à faire évoluer).

Documenté pour l'opérateur et le propriétaire : [docs/guide.md](docs/guide.md) (fichiers du traitement, garanties, correspondance code / Python) ; essais Excel réels à faire sous Windows : `docs/m6-qualification.md`, étape 6.

### Évolution de la colonne F de l'inventaire (branche `claude/colonne-f-repetitions`)

Détail, avant / après et note pour le refactor Python : [docs/m3-contrats-v1.md](docs/m3-contrats-v1.md), section « Évolution — colonne F de l'inventaire ». Lecture de l'inventaire pour l'opérateur : [docs/guide.md](docs/guide.md).

| Élément | Changement |
|---|---|
| `src/regles.rs` | `occurrences_par_flux` retiré de `Synthese` et de `synthetiser` ; `repetitions_dans_un_modele` ajouté ; 6 tests unitaires |
| `src/excel.rs` | `ecrire_inventaire` : nouvel en-tête F, une entrée par ligne |
| `tests/cli.rs` | Attentes de l'inventaire et de `synthese.json` mises à jour ; modèles homonymes répétés ; ancien classeur à six colonnes accepté |
| `tests/traitement.rs` | Traitement créé avant l'évolution (ancien `synthese.json`, ancienne colonne F) repris jusqu'au SGX, sans réécriture de ces fichiers |
| `tests/qualification/verifier_v1.py` | Seuls deux écarts acceptés, contrôlés exactement ; entrée `saphir` contrôlée si présente |

Preuves (Linux, conteneur cloud) :

| Contrôle | Résultat |
|---|---|
| `cargo test --workspace --locked` | 103 tests réussis : 36 unitaires du moteur, 2 de chemins, 36 CLI, 28 de traitement, 1 de l'adaptateur |
| `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo fmt --all --check` | Propres |
| `npm ci`, `npm run build` | Réussis, sans avertissement |
| Qualification (pilote + `verifier_v1.py`), binaire Linux | 30 scénarios conformes ; seuls écarts nouveaux : inventaire (F) et `synthese.json`, vérifiés exactement |
| `verifier_v1.py` face à des altérations | 5 altérations détectées (colonne F, colonne C, en-tête F, `synthese.json`, autre fichier) ; binaire de `main` refusé (« évolution absente ») |
| Copie de SAPHIR (non versionnée) | 12 noms, 13 propositions, SGX produit ; seules les deux sorties attendues diffèrent de Python, aux trois étapes ; colonne F : 4 lignes, 6 entrées |

Preuves (Windows, intégration continue sur `47422f3`, fusion de la branche dans `main`) : https://github.com/Frederic-K/bpmn-script-rs/actions/runs/36629309187

| Contrôle | Résultat |
|---|---|
| Rust | 1.89.0 (`1.89.0-x86_64-pc-windows-msvc`), installé depuis `rust-toolchain.toml` |
| `cargo test --workspace --locked` | 103 tests réussis : 36 unitaires du moteur, 2 de chemins, 36 CLI, 28 de traitement, 1 de l'adaptateur |
| `npm ci`, `npm run tauri build` | Application et installateur NSIS construits |
| Qualification (pilote + `verifier_v1.py`), binaire Windows | « Attentes V1 respectées : 30 scénarios » ; écarts limités à l'inventaire et à `synthese.json` (entrée `saphir` absente, non contrôlée) |
| Lancement de l'application (WebView2) | Application lancée, toujours active après 15 s |

Limites :
- Sous Linux, Rust 1.95.0 utilisé : la version 1.89.0 fixée par `rust-toolchain.toml` n'était pas téléchargeable depuis le conteneur. Elle a été vérifiée par l'intégration continue Windows (ci-dessus). Clippy et rustfmt n'ont été exécutés qu'avec 1.95.0.
- Non exécutés : test de bout en bout `tests/interface/parcours.py`, ouverture réelle dans Excel, import Signavio.

### Écran Décisions, révision de l'interface et nettoyage (30 septembre – 1er octobre, session locale Windows)

Décisions correspondantes : lignes « Écran Décisions », « Révision de l'interface » et « Disposition » du tableau ci-dessus. Chaque commit contient son avant / après.

| Branche (fusion) | Commits | Contenu |
|---|---|---|
| `claude/ecran-decisions` (`c7e0518`) | `a747aa1`, `610e619` | Tableau des lignes non admises retiré ; message des OUI ignorés ; libellé du contrôle conforme (« les décisions admises… ») ; `Bilan.lignes_a_examiner`, `LigneExaminee`, `valeur_affichee` supprimés ; test : un manifeste antérieur qui les contient reste lisible |
| `claude/revision-ui` (`9ea8b9d`) | `75f4af0`, `eecea58` | Bouton principal = action suivante (« Ouvrir le classeur », puis « Lire… » une fois modifié), dérivé de `*_adoptees` et `*_a_relire` |
| | `342a1b6` | Textes « À relire » reformulés comme étape suivante (alerte conservée) |
| | `740571f` | Écran Analyse : compteurs seuls ; composant `Tableau` supprimé |
| | `aaf3055` | `excel.rs` : un seul format (retour à la ligne, centrage vertical) pour toutes les cellules des trois classeurs ; paramètre `colonnes_retour_ligne` supprimé ; test `toutes_les_cellules_a_la_ligne_et_centrees_verticalement` |
| | `fdedb9b` | Étapes en barre horizontale sous l'en-tête |
| | `54d4184` | Nettoyage : titre d'écran rendu par `App.svelte` (focus direct), composant `Carte` (5 blocs), statuts de tentative inaccessibles retirés |

**Incident d'historique (sans effet sur le code).** Le 30 septembre, un état intermédiaire de l'écran Décisions a été poussé directement sur `main` par erreur (`f823ae1`, `ba726ba`, `0349f4e` « Auto stash before rebase »). Il a été annulé par un revert unique (`f0b3b13`, arbre identique à `4a9956d`), sans réécriture d'historique, puis refait sur `claude/ecran-decisions`.

Preuves :

| Contrôle | Résultat |
|---|---|
| `cargo test --workspace --locked` (Windows local, Rust 1.89.0) | 104 tests réussis : 37 unitaires du moteur, 2 de chemins, 36 CLI, 28 de traitement, 1 de l'adaptateur |
| Clippy (`-D warnings`), rustfmt, `npm run build` | Propres, sans avertissement (Windows local) |
| Intégration continue Windows | Tests, installateur, qualification (« Attentes V1 respectées : 30 scénarios ») et lancement réussis sur `610e619` (https://github.com/Frederic-K/bpmn-script-rs/actions/runs/36744135310) et `54d4184` (https://github.com/Frederic-K/bpmn-script-rs/actions/runs/36896014095) ; arbre de `main` identique après chaque fusion |
| Rendu de l'interface | Vrais composants dans un navigateur, moteur simulé : écran Décisions (conforme, OUI ignorés, à relire), Correspondances, Analyse, accueil, barre d'étapes à 1200 et 700 px, focus sur le titre. Pas dans la WebView2 de l'application |
| Essai par le propriétaire | Exécutable construit depuis la branche (`54d4184`) pour un essai avant fusion ; aucun retour d'essai consigné |

Limites :
- **`tests/interface/parcours.py` n'a pas été exécuté depuis ces changements** (Linux uniquement). Ses attentes ont été mises à jour (absence de tableau, compteurs de l'Analyse 2 / 3 / 2, OUI ignoré, motif vérifié dans le rapport Excel, nouveaux libellés) et seule leur syntaxe a été vérifiée.
- Qualification locale non rejouée (`openpyxl` absent du poste) : seulement par l'intégration continue.
- Mise en forme des classeurs non vérifiée dans Excel réel (hauteur des lignes). Écart de présentation avec le Python relevé par le pilote, non contrôlé par `verifier_v1.py`.
- Trois écarts assumés avec `ui-spec.md` (motif des décisions à l'écran, aperçu ligne par ligne de l'analyse, navigation à gauche) : voir le tableau des décisions.

## Preuves

| Contrôle | Résultat |
|---|---|
| `cargo test --workspace --locked` | 104 tests (état au 2 octobre) : 37 unitaires du moteur, 2 de chemins, 36 CLI, 28 de traitement, 1 de l'adaptateur. Sous Windows, en local et par l'intégration continue |
| Clippy (`-D warnings`), rustfmt, `npm run build` | Propres, sans avertissement |
| Qualification V1 (30 scénarios comparés au Python d'origine) | Conforme sous Linux (rejouée après la règle des contradictions) et avec le binaire Windows (`tests/qualification/verifier_v1.py`) |
| Test de bout en bout de la vraie application (Linux, WebKitGTK) | **Dernière exécution avant les changements d'interface d'octobre ; non rejoué depuis.** 33 contrôles réussis alors, dont : double clic sur « Générer » (une seule tentative), classeur modifié qui suspend la génération, D02, D03, D04, D08, contradiction bloquante, import refusé, provenance des décisions, analyse à actualiser |
| Windows (GitHub Actions) | Dernier passage complet sur `54d4184` (104 tests, installateur, qualification, lancement) : https://github.com/Frederic-K/bpmn-script-rs/actions/runs/36896014095 |
| Mutations volontaires du code (M1) | 5 sur 5 détectées par les tests |

**Non vérifié :**
- le poste cible (A16) ;
- SAPHIR, qui n'a pas été rejoué ;
- l'import Signavio (A17) ;
- l'ouverture réelle d'Excel, et la nouvelle mise en forme des classeurs ;
- le test de bout en bout depuis les changements d'interface d'octobre ;
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
7. **Changements d'octobre** (`git diff 47422f3 9ea8b9d`) :
   - `src/workflow.rs` : `Bilan` sans `lignes_a_examiner` ; un manifeste antérieur reste lisible (serde ignore le champ), mais une version antérieure de l'application ne relit pas un manifeste écrit par celle-ci.
   - `src/excel.rs` : la mise en forme ne touche ni les valeurs, ni les en-têtes, ni la lecture ; le test inspecte le XML produit par `rust_xlsxwriter`.
   - Interface : boutons principaux dérivés de `*_adoptees` / `*_a_relire`, titre d'écran et focus dans `App.svelte`, `Carte` avec `$props.id()`.

   Question clé : **un retrait d'information à l'écran (motifs, aperçu de l'analyse) laisse-t-il l'opérateur sans moyen de comprendre un refus ?** (renvois au rapport de contrôle et au classeur de décision).

Points volontairement discutables à challenger :
- **Comparaison des JSON sans l'ordre des clés** (`verifier_modele`) : les clés ne sont jamais réordonnées (`preserve_order`), mais un réordonnancement ne serait pas détecté.
- **Attributs non conservés** sur les entrées ZIP non modifiées (`external_attr`, champs supplémentaires) : c'est une limite de la bibliothèque `zip`, documentée, dont l'effet dépend du résultat de l'import Signavio.
- **Anomalie bloquante :** une seule anomalie dans un modèle bloque tout le traitement. C'est plus sûr, mais contraignant si un export réel contient une anomalie tolérable.
- **Code de sortie 2** du CLI : changement de contrat par rapport à l'ancien programme.
- **Écarts avec `ui-spec.md`** décidés par le propriétaire (tableaux retirés, étapes en haut) : la spécification figée n'est pas modifiée, les écarts sont dans le tableau des décisions.

## Environnement de cette session (pour reproduire)

- Sous Linux, Rust et Python étaient isolés dans un dossier temporaire ; `webkit2gtk-4.1`, `webkit2gtk-driver`, `xvfb` et `xdotool` ont été installés dans le conteneur.
- Le paquet de qualification a toujours été décompressé hors du dépôt.
- Aucune donnée réelle n'a été manipulée.
- Depuis le 30 septembre : session locale Windows 11, Rust 1.89.0 (`rust-toolchain.toml`). Aucune donnée réelle versionnée ; les retours sur un traitement réel (document de révision de l'interface, avec captures) viennent du propriétaire, hors dépôt.
