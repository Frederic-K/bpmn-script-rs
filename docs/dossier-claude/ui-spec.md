# BPMN-Script — parcours et interface V1

Version 1.0 · 26 septembre 2026 · **proposition**, dérivée de [product-spec.md](product-spec.md). Les principes validés sont le traitement local, l’arbitrage Excel et les contrôles humains. Cette spécification ne prétend pas reproduire une ancienne maquette, qui n’est pas disponible dans ce dossier.

## Organisation générale

Application en français, une fenêtre et un traitement actif. En-tête : BPMN-Script, nom du traitement et accès au dossier. Navigation par cinq étapes : **Source → Correspondances → Analyse → Décisions → Résultat**. Une action principale par étape, avec les actions secondaires proches du fichier auquel elles se rapportent.

Proposition visuelle sobre : fond clair, texte sombre, une couleur principale pour les actions, couleurs de statut accompagnées de libellés. Typographie système lisible, espacements réguliers. Aucun graphique décoratif. Les chemins longs sont repliables et copiables. Afficher les détails techniques à la demande, pas dans les instructions métier.

Disposition cible : navigation d’étapes à gauche, contenu au centre, bilan contextuel dans le même écran. À petite largeur, étapes compactes en haut et contenu sur une colonne. Pas de défilement horizontal de l’écran ; les tableaux de détail peuvent défiler dans leur zone.

## Accueil et source

Accueil : deux actions, « Nouveau traitement » et « Reprendre un traitement ». La reprise ouvre un dossier choisi ; aucun compte ni historique global n’est nécessaire.

Nouveau traitement : sélectionner un fichier `.sgx` et le dossier qui accueillera un nouveau sous-dossier de traitement. Le sélecteur accepte un seul fichier. Afficher le nom et l’emplacement choisis, puis « Inventorier les lanes ». Une extension correcte ne suffit pas : Rust valide réellement l’archive.

Pendant le travail : état occupé, phase en cours, boutons de mutation désactivés. Pas de pourcentage fictif. Après succès : nombres de modèles reconnus, occurrences de lanes et noms distincts ; date d’inventaire ; bouton « Préparer les correspondances ».

Une archive sans modèle reconnu n’est pas un inventaire réussi avec des zéros inexpliqués : montrer « Aucun modèle compatible trouvé » et proposer de sélectionner un autre export. Une archive contenant des modèles valides sans lane non vide peut se terminer normalement avec cette explication.

## Correspondances

Présenter : « Renseignez les nouveaux noms dans Excel. Une cellule Nouveau nom vide laisse le nom inchangé. » Boutons « Ouvrir le classeur » et « Importer un classeur de correspondances ». L’application prépare et gère le bon fichier ; l’utilisateur n’a pas à le renommer.

L’ouverture se fait avec l’application associée aux `.xlsx`. Si elle échoue : expliquer et proposer « Afficher dans le dossier ». Ne pas annoncer qu’Excel est installé sans vérification.

Au retour : « Enregistrez puis fermez le classeur avant de continuer. » Action « Lire les correspondances ». Lire une copie stable ; si le fichier change pendant la lecture ou est inaccessible, conserver les dernières données valides et expliquer comment réessayer.

Résumé après adoption : noms avec une demande, noms inconnus, doublons informatifs et erreurs éventuelles. Les cellules invalides sont listées par feuille, ligne, colonne et motif. Aucun tableau éditable dans l’application. Le nouveau nom vide est normal, pas une erreur.

## Analyse

Action « Préparer l’analyse ». Afficher le nombre de propositions, les occurrences visées et les modèles concernés (comptés par chemin). Dans un aperçu en lecture seule : flux, ancien nom, nouveau nom, occurrences ; chemin interne disponible pour distinguer les homonymes.

Résultat vide : « Aucun changement proposé. Vérifiez les nouveaux noms renseignés. » Rester capable de revenir aux correspondances ; aucune génération possible.

Actions : « Ouvrir l’analyse » et « Enregistrer une copie pour arbitrage ». La seconde utilise un sélecteur de destination ; l’utilisateur transmet lui-même le fichier. Aucun envoi par email ou autre service n’est implicite.

Le classeur de décision est une copie de l’analyse, avec colonne OUI/NON. L’analyse calculée reste un résultat distinct et n’est pas modifiée par les décisions humaines. On peut modifier la copie locale ou importer un retour d’arbitre à l’étape suivante.

## Décisions

Texte : « Renseignez OUI ou NON dans la colonne Validation, puis importez le classeur enregistré. Les lignes absentes ne seront pas appliquées. » Actions « Ouvrir le classeur de décision », « Importer les décisions », puis « Contrôler les décisions ».

Après contrôle : afficher séparément les lignes OUI admises, OUI ignorées, refus, attentes et propositions sans décision reçue. Ces catégories sont des comptes explicites ; les doublons de lignes ne doivent pas donner l’illusion d’un nombre de propositions distinctes. Indiquer aussi les occurrences prévues par les décisions admises.

Une décision invalide montre son motif et la valeur attendue si disponible. Une cellule de type incorrect respecte P02 ; les autres OUI conformes continuent à être examinés. Un OUI obsolète est ignoré. L’interface ne transforme jamais automatiquement toutes les propositions en OUI.

Le recomptage en mémoire fait partie du contrôle avant production. En cas de divergence : « Les modifications ne correspondent pas aux occurrences attendues. Aucun nouveau SGX n’a été créé. » Détails par modèle et proposition, accès au rapport et retour aux fichiers à corriger.

Si contrôle conforme et au moins une décision admise : bouton « Générer le SGX modifié ». Son libellé suffit comme action explicite ; aucune seconde boîte de confirmation générique n’est nécessaire. Rust refait les vérifications sur les entrées effectivement adoptées avant d’écrire.

## Résultat

Succès uniquement après finalisation du fichier et enregistrement cohérent de la tentative. Afficher nom du SGX, date, nombre de modèles modifiés et occurrences traitées. Actions « Afficher le SGX dans le dossier » et « Ouvrir le rapport de contrôle ».

Mention : « Importez ce fichier dans un emplacement Signavio de test et vérifiez les modèles concernés. » Ne pas afficher « Compatible Signavio » avant la qualification externe.

Si aucune production : expliquer la cause (absence de OUI admissible, décompte divergent ou erreur). Une ancienne sortie reste dans une section distincte « Résultat précédent », avec date et mention « Non produit par cette tentative ». Elle n’est jamais le bouton principal du résultat courant.

## États et transitions

Ces états appartiennent au traitement. Une erreur récupérable est un diagnostic associé au dernier état valide, pas une transition automatique vers un faux succès.

| État | Entrée | Action suivante |
|---|---|---|
| SOURCE_A_CHOISIR | Aucun traitement | Sélectionner une source et un dossier |
| INVENTAIRE_PRET | Copie source contrôlée, inventaire terminé | Préparer/lire les correspondances |
| CORRESPONDANCES_PRETES | Instantané de correspondances adopté | Préparer l’analyse |
| ANALYSE_PRETE | Analyse liée aux entrées courantes, éventuellement vide | Arbitrer si impacts ; sinon modifier les correspondances |
| DECISIONS_CONTROLEES | Décisions comparées, recomptage terminé | Générer si admissibles et conformes ; sinon corriger |
| RESULTAT_PRODUIT | Sortie finalisée et enregistrée | Consulter/exporter le résultat |

Changement de correspondances adopté : retour à CORRESPONDANCES_PRETES ; analyse, décisions contrôlées et autorisation de production deviennent obsolètes. Réimport de décisions : invalider le contrôle précédent jusqu’à nouveau contrôle. Choisir une autre source crée un nouveau traitement. Consulter une étape antérieure n’invalide rien en soi.

## Reprise et changements de fichiers — P01/P04

L’application travaille sur sa copie source, dont l’empreinte est contrôlée. Déplacer ou supprimer l’original externe n’empêche pas de reprendre si cette copie est intacte. Une copie source interne altérée bloque le traitement : proposer un nouveau traitement depuis une source choisie. Ne jamais réassocier silencieusement un autre fichier.

Ouvrir un Excel de travail autorise sa modification externe. Au retour dans l’application et avant l’action suivante, vérifier s’il a changé. Si oui, afficher « Ce classeur a changé. Relisez-le pour actualiser le traitement » et suspendre la génération fondée sur l’ancienne lecture. Pas de recalcul automatique caché ; l’action explicite adopte un nouvel instantané.

Un classeur reçu d’un arbitre n’a pas besoin d’un identifiant propriétaire ajouté par l’application. Il est contrôlé contre les propositions recalculées du traitement choisi. La compatibilité des cinq champs ne prouve pas à elle seule la provenance du classeur ; ne pas afficher une garantie plus forte.

En cas de fermeture pendant une opération : sauvegarder seulement les états finalisés. À la reprise, expliquer qu’une tentative a été interrompue ; revalider les fichiers et ne pas promouvoir un temporaire en résultat sans vérification.

## Messages et accessibilité

| Situation | Message et action utile |
|---|---|
| Classeur verrouillé/inaccessible | « Impossible de lire ce classeur. Enregistrez-le, fermez-le puis réessayez. » Afficher le fichier concerné. |
| Mauvaise feuille/en-têtes | « Ce classeur ne correspond pas au format attendu. » Donner feuille/colonnes attendues ; conserver l’ancien import valide. |
| Mauvaise cellule | « Correspondance, ligne 8, Nouveau nom : texte attendu. » Proposer de rouvrir le fichier. |
| Disque plein/écriture refusée | « Le SGX n’a pas pu être créé. » Indiquer la destination et permettre une nouvelle tentative. |
| État de session illisible | « Ce traitement ne peut pas être repris. » Conserver les fichiers ; proposer un nouveau traitement, sans écraser le dossier. |

Navigation complète au clavier, ordre de focus naturel, focus visible, libellés explicites, statuts annoncés aux technologies d’assistance. La couleur ne porte jamais seule le sens. Les tableaux ont des en-têtes ; les détails des erreurs sont copiables. Tester une petite fenêtre et un agrandissement du texte.

## Vérification UI

Vérifier A11–A15 avec un vrai binaire Tauri : nouveau traitement, reprise après fermeture, retour partiel d’arbitrage, changement de correspondances, fichier ouvert, erreur de production et présence d’un ancien résultat. Vérifier que la génération appelle Rust et que son état reflète le résultat réel ; les seules données simulées ne constituent pas une validation du parcours.
