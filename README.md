# bpmn-script-rs

Portage Rust de [bpmn-script](https://github.com/Frederic-K/bpmn-script), version V0.1 (tag `v0.1-sgx-import-tested`).

Objectif : comparer une réécriture Rust simple (KISS, YAGNI) avec le script Python d'origine. Le workflow, les dossiers et les fichiers produits sont identiques.

## Contenu

| Fichier | Rôle |
| --- | --- |
| `src/main.rs` | CLI historique : lance le workflow sur `input/`, `work/`, `output/` du dossier courant |
| `src/lib.rs` | Bibliothèque : point d'entrée `executer(&Chemins, journal)` avec dossiers explicites |
| `src/workflow.rs` | Enchaînement des étapes, écriture des JSON |
| `src/regles.rs` | Règles métier en mémoire (inventaire, synthèse, dry-run, contrôle, test des renommages) |
| `src/sgx.rs` | Lecture et écriture de l'archive SGX |
| `src/excel.rs` | Lecture des Excel de saisie, écriture des Excel produits |
| `tests/` | Tests d'intégration (`cargo test`) |
| `Cargo.toml` | Dépendances |
| `src/traitement.rs` | Dossier de traitement : copie source, instantanés, manifeste, reprise, tentatives |
| `rust-toolchain.toml` | Version de Rust (1.89.0) |

Dépendances, une par besoin :

| Crate | Besoin |
| --- | --- |
| `zip` | Lire et écrire l'archive SGX |
| `serde`, `serde_json` | Lire et écrire les JSON (ordre des clés conservé) |
| `indexmap` | Dictionnaires ordonnés, comme les `dict` Python |
| `calamine` | Lire les Excel de `work/` |
| `rust_xlsxwriter` | Écrire les Excel de `output/` |

## Compilation

Prérequis : Rust installé (`rustup`).

```powershell
cargo build --release
cargo test
```

Binaire produit : `target\release\bpmn-script-rs.exe`.

## Utilisation

Même mode opératoire que le README de bpmn-script. Seule la commande change.

Depuis le dossier de travail qui contient `input/`, `output/` et `work/` :

```powershell
.\bpmn-script-rs.exe
```

Workflow inchangé :

```text
Export SGX
  -> inventaire              (output/inventaire_swimlanes.xlsx)
  -> correspondance humaine  (work/correspondance_swimlanes.xlsx)
  -> dry-run                 (output/analyse_modifications.xlsx)
  -> validation humaine      (work/validation_modifications.xlsx)
  -> contrôles et SGX modifié (output/<nom>_modifie.sgx)
```

## Comparaison avec la version Python

Contrôle réalisé sur un SGX de test synthétique, puis sur un export Signavio réel (SAPHIR, 241 modèles, 848 swimlanes, 289 noms uniques), avec les mêmes saisies humaines pour les deux versions :

- messages console identiques aux 3 lancements ;
- fichiers JSON identiques octet pour octet ;
- Excel identiques (valeurs, largeurs, volet figé, filtre, liste OUI/NON, surlignage) ;
- SGX modifié identique après lecture (mêmes entrées, même compression, mêmes JSON) ;
- cas testés : renommage validé, refus (`NON`), ligne falsifiée (`IGNORÉE`), renommages en chaîne (test en mémoire non conforme, pas de SGX produit) ;
- export réel : renommage des 13 swimlanes dont le nom contient un retour à la ligne ou un double espace, sans écart ;
- SGX produit : archive valide, 630 entrées dans le même ordre que la source.

| Critère | Python | Rust |
| --- | --- | --- |
| Lignes de code | 716 | environ 600 (après `cargo fmt`) |
| Installation poste | Python + venv + `pip install` | Aucune : un `.exe` autonome |
| Temps, export réel (241 modèles, 3e lancement complet) | 0,7 s | 0,17 s |
| Temps, SGX synthétique (2 000 modèles, inventaire) | 1,5 s | 0,9 s |

Sur un export réel, l'écart vient surtout du démarrage de Python et du chargement d'openpyxl. Sur un gros volume, il se réduit : le parsing JSON Python est déjà écrit en C. L'intérêt principal reste le binaire unique à distribuer.

Point restant à vérifier : l'import dans Signavio d'un SGX produit par la version Rust (dossier de test). Les en-têtes ZIP diffèrent légèrement de la version Python (voir ci-dessous) ; le contenu des modèles est identique.

## Écarts assumés

| Point | Python V0.1 | Rust |
| --- | --- | --- |
| Dossier `output/` absent | Erreur | Créé automatiquement |
| Analyse sans impact | Erreur (plage Excel vide) | Analyse vide, résultat explicite |
| Erreur bloquante | Trace Python | Message `[ERREUR] ...` et code retour 1 |
| Entrées SGX non modifiées | Recompressées | Données compressées d'origine recopiées sans recompression |
| En-têtes ZIP | Système « DOS » conservé | Système de la plateforme de compilation ; commentaires, champs supplémentaires et date conservés sur les entrées réécrites, commentaire d'archive conservé |
| JSON des modèles modifiés | Séparateurs `, ` et `: ` | Format compact, nombres recopiés à l'identique (contenu identique) |

## Contrat V1 (lot M3)

Détail et justification : [docs/m3-contrats-v1.md](docs/m3-contrats-v1.md).

- **Modèles** : un JSON illisible, des métadonnées absentes, un nom de lane ou un `childShapes` mal typé bloquent le traitement avec la liste des anomalies. Aucun inventaire partiel.
- **Excel** : feuille et en-têtes contrôlés ; noms et chemins en texte, occurrences en nombre entier ; formules refusées. Une erreur dans les correspondances refuse le classeur ; une erreur sur une ligne OUI ignore cette ligne seulement.
- **Production** : le SGX est écrit dans un fichier temporaire, relu et comparé à la source (seules les lanes validées peuvent différer), puis publié sans jamais écraser un fichier existant (`_modifie_2.sgx`...).
- **Résultat** : dernière ligne `[RÉSULTAT] ...` ; code de sortie 0 (étape terminée), 1 (erreur bloquante), 2 (décisions fournies mais aucun SGX produit).

## Hors périmètre

Volontairement non ajouté : arguments en ligne de commande, configuration, journalisation. Le CLI garde le comportement de l'original ; le découpage en modules (lot M2) ne change ni les fichiers produits ni les messages. Voir `docs/`.
