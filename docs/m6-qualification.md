# M6 — Qualification

26 septembre 2026 · lot M6 de [implementation-plan.md](dossier-claude/implementation-plan.md), suite de [m5-interface.md](m5-interface.md). Branche `claude/m6-qualification`.

Deux parties : ce qui est vérifié automatiquement sur Windows (intégration continue), et ce que seul l'opérateur peut qualifier (A16 sur son poste, A17 dans Signavio).

## 1. Qualification automatique sous Windows

Le workflow [`.github/workflows/windows.yml`](../.github/workflows/windows.yml) s'exécute à chaque envoi sur une machine Windows (GitHub Actions, `windows-latest`) :

| Étape | Vérifie |
|---|---|
| Tests Rust | Les 81 tests du moteur, du CLI, du dossier de traitement et de l'adaptateur, sous Windows (chemins, verrou système, renommage du manifeste) |
| Application et installateur | `npm run tauri build` : compilation release et installateur NSIS |
| Qualification V1 | Les 30 scénarios du paquet de qualification rejoués avec le **binaire Windows**, comparés au Python d'origine, puis vérifiés par [`tests/qualification/verifier_v1.py`](../tests/qualification/verifier_v1.py) (attentes V1, séparées de la caractérisation historique) |
| Lancement | L'application démarre et reste ouverte 15 s (WebView2 présent sur la machine) |
| Artefacts | `bpmn-script-windows` : installateur, application, CLI (conservés 30 jours) |

Résultat de l'exécution : voir la section 4.

### Choix d'installation

- **Installation pour l'utilisateur courant**, sans droits administrateur (`installMode: currentUser`).
- **WebView2 :** le programme d'installation embarque un petit amorceur qui ne télécharge WebView2 que s'il manque. Windows 10 et 11 à jour l'ont déjà ; un poste sans WebView2 et sans accès à Internet ne pourra pas l'installer (option « installateur hors ligne » possible, environ 130 Mo de plus).
- **Installateur non signé :** Windows SmartScreen peut afficher « Windows a protégé votre ordinateur ». Choisir « Informations complémentaires » puis « Exécuter quand même ». Une signature de code nécessite un certificat de l'organisation : point ouvert.

## 2. A16 — Poste cible (opérateur)

1. Télécharger l'artefact `bpmn-script-windows` depuis la page de l'exécution GitHub Actions (section « Artifacts »), puis le décompresser.
2. Lancer `BPMN-Script_0.1.0_x64-setup.exe` ; noter tout message (SmartScreen, WebView2, antivirus).
3. Démarrer BPMN-Script depuis le menu Démarrer.
4. Faire un traitement complet sur un **petit export de test** : inventaire, correspondances, analyse, décisions, génération.
5. Vérifier ensuite :
   - **Ouverture des classeurs :** « Ouvrir le classeur » ouvre bien Excel.
   - **Retour d'Excel :** modifier le classeur, l'enregistrer puis revenir dans l'application : l'étape doit afficher « à relire ».
   - **Reprise :** fermer l'application, la rouvrir, et reprendre le traitement avec « Ouvrir un dossier de traitement ».
   - **Dossier :** « Afficher le SGX dans le dossier » ouvre l'Explorateur sur le fichier.
   - **Double ouverture :** une seconde instance ouverte sur le même traitement est refusée.
6. **Parcours des classeurs avec Excel réel** (essais recommandés par la revue du parcours) :
   - **Validation locale :** « Ouvrir le classeur de décision », saisir OUI ou NON, enregistrer, fermer, « Lire le classeur de décision ». Le message « Décisions lues le … depuis le classeur de décision du traitement » apparaît.
   - **Validation externe :** « Enregistrer une copie à transmettre » vers un nouveau nom, la remplir comme un valideur, puis « Importer le retour du valideur ». Avec les mêmes réponses, le résultat doit être identique à la validation locale. Le message indique le nom du fichier importé.
   - **Copie sans écrasement :** « Enregistrer une copie » vers un fichier existant, en confirmant « Remplacer » dans le dialogue de Windows. La copie doit être refusée et le fichier existant intact.
   - **Retour remplacé :** importer un premier retour partiel, puis un second. Seul le second est utilisé ; les propositions absentes apparaissent en « Sans décision ».
   - **Import refusé :** importer un classeur au mauvais format (le classeur de correspondance, par exemple). Le message « Classeur non adopté. Les décisions lues le … restent utilisées » apparaît.
   - **Correspondances modifiées :** modifier le classeur de correspondance après l'analyse. L'analyse, les décisions et le bilan affichent « à actualiser », et la génération est suspendue.
   - **Contradiction :** dupliquer une ligne du classeur de décision, avec OUI sur l'une et NON sur l'autre. Le contrôle est bloquant, les deux lignes sont citées, et aucun SGX n'est possible.
7. **Emplacement sans liens physiques :** « Enregistrer une copie » vers une clé USB en FAT ou exFAT. La copie doit être refusée avec la consigne de choisir un disque local, sans laisser de fichier sous le nom choisi.
8. **Clé USB (facultatif) :** copier `bpmn-script-app.exe` sur une clé et le lancer sur un autre poste disposant de WebView2. La portabilité n'est pas garantie tant que cet essai n'est pas consigné.

## 3. Essai sur une copie de SAPHIR puis A17 — Signavio (opérateur)

Le fichier SAPHIR ne doit pas être ajouté au dépôt. Travailler sur une **copie**.

### Comparaison automatique Python / Rust sur SAPHIR (Windows)

```powershell
Expand-Archive docs\dossier-claude\references\qualification-rust-reproductible.zip -DestinationPath C:\qualification
Copy-Item "<copie de SAPHIR>.sgx" "C:\qualification\SAPHIR - Modélisation des processus cibles.sgx"
python -m pip install openpyxl==3.1.5
python C:\qualification\qualify_rust.py --root C:\qualification --binary <chemin>\bpmn-script-rs.exe
```

Résultat attendu pour l'entrée `saphir` de `rust-qualification-results.json` :
- 12 noms normalisés et 13 propositions ;
- aucune différence avec Python aux trois étapes ;
- un SGX produit.

(Mesures de référence du dossier : 241 modèles, 848 occurrences, 289 noms distincts.) Les décisions OUI de cet essai sont fabriquées pour le test : ce n'est pas un arbitrage métier.

### Import Signavio (A17)

1. Produire avec l'application un SGX à partir de la copie de SAPHIR, avec quelques renommages réels validés.
2. L'importer dans un **emplacement Signavio de test**, jamais en production.
3. Vérifier :
   - que l'import se termine sans erreur ;
   - que le nombre de modèles importés est le même que dans la source ;
   - pour 3 modèles renommés : le nouveau nom des lanes, et que le reste du diagramme (formes, liens, propriétés) est intact ;
   - pour 3 modèles non renommés : aucune différence visible ;
   - que les pièces jointes et les métadonnées (nom du modèle, dossier) sont intactes.
4. Consigner le résultat ci-dessous.

## 4. Résultats

### Automatique (GitHub Actions, Windows) — réussie

Exécution n° 1, commit `bd6d434`, 26 septembre 2026, `windows-latest` : https://github.com/Frederic-K/bpmn-script-rs/actions/runs/36253157042

| Étape | Résultat |
|---|---|
| Tests Rust (81 tests) | Réussis sous Windows, en 7 min |
| Application et installateur NSIS (release) | Réussis, en 11 min |
| CLI historique (release) | Réussi |
| Qualification V1 : 30 scénarios, binaire Windows comparé au Python d'origine | « Attentes V1 respectées : 30 scénarios. » Codes, écarts et SGX identiques à la mesure sous Linux |
| Lancement de l'application | « Application lancée depuis 15 s » : WebView2 opérationnel, aucun arrêt |
| Artefact `bpmn-script-windows` | Installateur, application et CLI ; 10,2 Mo ; SHA-256 du zip `a2eaf9c17175c9a49b2c1532b1735f45ec9657bf133540b64d60b9576575781b` ; disponible jusqu'au 26 octobre 2026 |

Critère A16 : compilation, installateur et lancement **qualifiés sur une machine Windows d'intégration continue**, pas encore sur le poste cible (section 2).

### Opérateur (à remplir)

| Critère | Date | Poste / environnement | Résultat | Observations |
|---|---|---|---|---|
| A16 — installation et lancement | | | | |
| A16 — parcours complet, Excel, reprise | | | | |
| A16 — parcours des classeurs (étape 6) | | | | |
| A16 — clé USB en FAT/exFAT refusée (étape 7) | | | | |
| Clé USB (facultatif) | | | | |
| SAPHIR — comparaison Python / Rust | | | | |
| A17 — import Signavio de test | | | | |

## Limites connues

- **Installateur non signé** : l'avertissement SmartScreen est possible.
- **Tests d'interface de bout en bout** exécutés sous Linux uniquement (M5). Sous Windows, les boîtes de dialogue natives ne peuvent pas être pilotées ainsi ; le parcours est à vérifier manuellement (section 2).
- **Attributs ZIP :** `external_attr` et le système d'origine sont recalculés par la bibliothèque `zip` (voir M3). Seul l'import Signavio (A17) permet de confirmer que c'est sans effet.
