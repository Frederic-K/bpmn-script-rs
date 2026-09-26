# bpmn-script-rs

Portage Rust de [bpmn-script](https://github.com/Frederic-K/bpmn-script), version V0.1 (tag `v0.1-sgx-import-tested`).

Objectif : comparer une réécriture Rust simple (KISS, YAGNI) avec le script Python d'origine. Le workflow, les dossiers et les fichiers produits sont identiques.

## Contenu

| Fichier | Rôle |
| --- | --- |
| `src/main.rs` | Tout le programme (un seul fichier, comme `main.py`) |
| `Cargo.toml` | Dépendances |

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
| Erreur bloquante | Trace Python | Message `[ERREUR] ...` et code retour 1 |
| Entrées SGX non modifiées | Recompressées | Données compressées d'origine recopiées sans recompression |
| En-têtes ZIP | Système « DOS » conservé | Système « Unix », droits de lecture standard (0644) |
| JSON des modèles modifiés | Séparateurs `, ` et `: ` | Format compact (contenu identique) |
| Cellule numérique dans une colonne texte | Lue comme nombre | Lue comme texte |

Ces écarts ne modifient ni les décisions humaines, ni les contrôles, ni le contenu des modèles.

## Hors périmètre

Volontairement non ajouté : arguments en ligne de commande, configuration, journalisation, tests automatisés, découpage en modules. Le programme reste un script linéaire, comme l'original.
