# BPMN-Script — qualification du portage Rust

26 septembre 2026 — complément à la revue initiale.

**Le portage fourni compile et les parcours usuels concordent avec Python. La parité générale n’est pas acquise : huit des trente scénarios synthétiques montrent un écart de contenu ou de résultat.** Les différences de présentation Excel et de métadonnées ZIP sont suivies séparément.

## Environnement et périmètre

Rust 1.88.0 a été installé dans un dossier isolé de cette tâche. La compilation de développement a réussi avec le `Cargo.lock` fourni, hors ligne. Le Rust par défaut de l’utilisateur et les dépôts sur E: n’ont pas été modifiés. Aucun changement du code métier ni des dépendances.

Les tests exécutent les deux programmes originaux sur des copies indépendantes, avec les mêmes SGX et Excel. Le pilote de comparaison est en Python avec openpyxl 3.1.5 ; il s’agit d’une suite externe testant réellement le binaire Rust, pas encore de tests unitaires internes en Rust.

Comparaisons : code de sortie, présence des fichiers, valeurs JSON, cellules Excel, ordre et contenu des entrées SGX. Les JSON internes sont comparés après lecture. Les styles Excel et attributs ZIP font l’objet de relevés distincts. Les assertions vérifient également l’intégrité des sources, la validité des ZIP produits et la préservation du contenu des entrées non concernées.

## Résultats

| Ensemble | Résultat observé |
|---|---|
| Vingt scénarios historiques | Dix-sept concordent sur les contenus et codes de sortie ; trois écarts : analyse vide dans deux cas, dossier de sortie absent. |
| Dix scénarios supplémentaires | Cinq concordent sur ces mêmes critères ; cinq révèlent des écarts sur Excel ou JSON atypiques. |
| Arbitrage partiel | Seules les lignes présentes et conformes sont appliquées ; concordance. |
| Doublon avec deux nouveaux noms différents | Dernière correspondance retenue ; concordance. |
| Ancien nom identique au nouveau | Renommage compté et SGX produit ; concordance. |
| Ancienne sortie puis refus de toutes les décisions | Ancienne sortie conservée dans les deux programmes ; aucune nouvelle production. |
| Source | Empreinte inchangée pour chaque exécution vérifiée. |

Les refus humains, décisions en attente, OUI normalisés, altérations de proposition, doublons de validation et conflits en chaîne sont couverts. Sur les cas testés, les deux derniers empêchent la génération lorsque le recomptage diverge.

**SAPHIR : trois étapes complètes réussies en Python et Rust**, sur des copies de test. Inventaire : 241 modèles, 848 occurrences, 289 noms distincts. Normalisation expérimentale de 12 noms distincts contenant un saut de ligne ou un double espace : 13 propositions et 13 occurrences renommées. Les JSON, valeurs Excel et contenus des archives concordent à chaque étape. Les décisions OUI de ce test ont été fabriquées pour l’essai ; elles ne constituent pas un arbitrage métier. Aucun résultat n’a été importé dans Signavio.

## Écarts reproduits et recommandations

Les recommandations ci-dessous sont proposées pour la V1 ; elles ne sont pas déjà implémentées.

| Cas | Observation | Recommandation |
|---|---|---|
| Correspondance vide ou ancien nom inconnu | Python échoue en ajoutant la validation Excel sur la plage inversée E2:E1 ; Rust produit une analyse vide. | Conserver le comportement Rust et afficher « Aucun changement proposé ». Ne pas reproduire ce défaut Python. |
| Dossier de sortie absent | Python échoue ; Rust crée le dossier et termine l’inventaire. | Conserver la création automatique. |
| Ancien nom numérique 123, lane texte « 123 » | Python ne trouve pas la clé numérique dans sa synthèse et échoue ensuite sur l’analyse vide ; Rust prépare un impact après conversion en texte. | Définir un contrat texte explicite et signaler les cellules de type incorrect, sans conversion silencieuse. |
| Nouveau nom numérique 123 | Python écrit un nombre JSON ; Rust écrit la chaîne « 123 ». Les deux admettent leurs décisions correspondantes. | Exiger un nouveau nom textuel. Pour « 123 », demander une cellule texte. |
| Formule Excel sans valeur calculée en cache | Python conserve la formule comme nouveau nom et produit un SGX ; Rust lit une valeur vide, ignore la correspondance et ne produit pas de SGX. | Refuser explicitement les formules dans les colonnes métier importées. Le cas avec résultat en cache reste à tester. |
| Nom de lane nul | Python échoue ; Rust ignore la lane mal formée et peut modifier un autre modèle. | Signaler l’anomalie et bloquer la production tant qu’elle n’est pas résolue ; ne pas afficher un inventaire partiel comme complet. |
| `childShapes` numérique | Python échoue ; Rust ignore les enfants et peut modifier un autre modèle. | Même politique explicite pour une structure mal formée. |
| Métadonnées ZIP | Sur la fixture enrichie, Rust perd les champs supplémentaires et le commentaire de l’entrée modifiée. Des attributs externes changent aussi sur des entrées non modifiées. | Corriger ou documenter chaque différence, puis qualifier dans Signavio. Une copie directe du contenu compressé ne prouve pas la conservation de tous les attributs. |
| Commentaire global ZIP | Non conservé par les deux versions sur la fixture. | Décider sa conservation comme amélioration explicite. |
| Présentation Excel | Valeurs identiques sur les cas usuels, mais largeurs sérialisées différentes (ex. 35 contre 35,7109375), regroupement des colonnes et encodage des couleurs différents. | Définir un rendu utile à préserver, puis le vérifier visuellement. Ne pas annoncer une identité exacte des styles. Les choix OUI/NON et cellules vides autorisées concordent dans le cas inspecté. |

Le nombre « huit écarts sur trente » concerne uniquement les contenus métier et codes de sortie. Il ne signifie donc pas que les vingt-deux autres scénarios sont identiques sur tous les attributs ZIP et styles Excel.

## Conséquences pour l’application

Le moteur doit retourner un résultat explicite : analyse sans impact, décisions attendues, aucune décision admissible, contrôle bloquant ou SGX effectivement produit. Un retour sans erreur ne prouve pas une génération : c’est le comportement actuel lors d’un conflit de décompte.

Chaque traitement doit identifier ses sorties courantes. Un ancien SGX laissé dans le dossier après un refus ne doit jamais être présenté comme le résultat de la nouvelle tentative. L’écriture dans un fichier temporaire suivie d’une finalisation réussie reste recommandée ; les pannes d’écriture n’ont pas été simulées dans cette passe.

La séparation Svelte / Tauri / moteur Rust proposée dans la revue reste pertinente. L’interface peut être spécifiée sur cette base, mais le contrat d’import Excel et le traitement des anomalies doivent apparaître explicitement dans les critères d’acceptation.

## Livrable de reproduction

L’archive de qualification jointe contient les sources de référence fournies, le pilote des trente scénarios, les assertions de caractérisation, les résultats JSON, un script de compilation/exécution et le verrouillage Rust 1.88.0. Les assertions distinguent les écarts connus d’une nouvelle régression ; leur réussite ne valide pas les défauts décrits ci-dessus.

Le SGX SAPHIR et les archives modifiées de test ne sont pas inclus dans ce paquet. Pour rejouer SAPHIR, placer une copie du SGX original à la racine du paquet sous son nom fourni. Le fichier de résultats inclus contient les mesures déjà obtenues, sans contenu des modèles SAPHIR.

Limites restantes : compilation release et packaging non vérifiés, import Signavio non effectué, interface non construite, tests non exhaustifs sur Unicode, précision numérique, formules avec cache et erreurs d’écriture. Les essais établissent une base de travail reproductible, pas une qualification finale de livraison.
