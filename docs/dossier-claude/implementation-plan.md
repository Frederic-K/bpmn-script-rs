# BPMN-Script — plan de développement et prompt de démarrage

Version 1.0 · 26 septembre 2026. Référence : [product-spec.md](product-spec.md), registre P01–P07 et critères A01–A17. Les propositions sont des choix de référence explicites, pas des validations métier déjà obtenues.

## Lots et sorties vérifiables

| Lot | Travail | Preuve de sortie |
|---|---|---|
| M0 — État réel | Lire les consignes, inspecter le dépôt, reproduire build et comparaison de la référence | Versions, commit, état local, résultats des 30 scénarios ; écarts par rapport aux mesures du dossier expliqués |
| M1 — Tests Rust | Introduire fixtures et tests de règles sans changer le métier | Lanes imbriquées, homonymes, OUI/NON/attente, altérations, arbitrage partiel, chaînes/doublons, source et non-cibles vérifiés |
| M2 — Extraction | Bibliothèque Rust, CLI léger, séparation calcul/SGX/Excel, chemins explicites | Comparaison au comportement de départ inchangée sur les cas caractérisés ; aucune régression inexpliquée |
| M3 — Contrats V1 | Traiter P02/P03/P05/P06 et résultats explicites de P04, un changement à la fois | Tests avant/après, liste des divergences volontaires, documentation alignée |
| M4 — Traitements | P01/P04 : manifeste, instantanés, reprise, invalidations, publication des sorties | A11–A14 : fermeture, fichiers modifiés/absents, erreur d’écriture et double lancement |
| M5 — Interface | Scaffold verrouillé P07, adaptateur Tauri, parcours Svelte défini | Parcours complet avec vrai moteur ; clavier, erreurs et retour d’Excel vérifiés |
| M6 — Qualification | Build release, packaging Windows, essai sur copie SAPHIR et import Signavio par opérateur | A16/A17 documentés avec environnement, observations et limites |

Chaque lot doit pouvoir être relu séparément. Les tests se développent au plus près des comportements extraits ; éviter un gros refactor préalable destiné uniquement à rendre les tests possibles. M1 et M2 peuvent progresser par petits incréments cohérents, en conservant la séparation de leurs objectifs.

M0 ne nécessite aucun arbitrage produit. M1/M2 peuvent caractériser les comportements actuels avant choix sur P02/P03. M3 conserve le registre des propositions et leurs tests V1 distincts. Les choix de distribution ou de dictionnaire encore ouverts n’empêchent pas de travailler sur le moteur.

## Utiliser le paquet de qualification

Décompresser [qualification-rust-reproductible.zip](references/qualification-rust-reproductible.zip) dans un dossier de travail distinct. Lire son `QUALIFICATION-README.md`. Le script fourni compile **la référence incluse**, pas automatiquement la branche de développement.

Pour comparer ensuite un binaire candidat :

```powershell
& $Python .\qualify_rust.py --root $DossierQualification --binary $BinaireCandidat
```

Ces variables représentent des chemins absolus choisis localement ; ne pas recopier des chemins propres au poste de revue. Le candidat doit encore supporter l’entrée CLI historique utilisée par les fixtures (répertoire de travail avec `input`, `work`, `output`). Si la CLI évolue, conserver un mode compatible ou adapter explicitement le pilote, sans modifier les données de référence.

`check_qualification.py` attend les comportements initiaux, y compris les défauts. Après M3, séparer les tests de caractérisation historique des attentes V1. Ne pas modifier la liste d’écarts pour faire passer le candidat sans une décision et un cas avant/après.

Les comparaisons de cellules ne suffisent pas à certifier le rendu Excel. Inspecter visuellement les trois classeurs et vérifier leurs choix OUI/NON. Une comparaison des contenus ZIP ne suffit pas à certifier les attributs : conserver les tests dédiés.

## Compléments indispensables à la suite

| Domaine | Cas à ajouter ou renforcer | Critères |
|---|---|---|
| Excel | Formules avec et sans cache ; erreurs/booléens ; occurrences 1, 1.0, « 1 », négatif, fraction ; classeur incomplet ; mauvais en-têtes ; fichier verrouillé | A09/A14 |
| Modèles | Nom absent/vide/nul/nombre ; enfants absents/mal typés ; metadata absente/invalide ; tâche homonyme ; clés inconnues | A01/A07/A10 |
| Renommage | Ancien = nouveau, permutations, doublons, conflits, mapping final vide | A04–A06 |
| ZIP | Commentaires, permissions, dates, champs supplémentaires, pièces jointes, grands nombres JSON et Unicode | A07/A08 |
| Reprise | Original externe déplacé, copie interne altérée, dossier déplacé, classeur modifié après contrôle, manifeste inconnu | A11/A12 |
| Publication | Ancienne sortie, disque plein ou erreur simulée, arrêt après fichier mais avant manifeste, collision, deux commandes concurrentes | A08/A13 |
| Interface | Parcours réel, sans résultat, erreur, reprise, clavier, petite fenêtre, agrandissement texte | A15 |

Pour les erreurs d’écriture, un point d’injection contrôlé dans les tests permet de vérifier les garanties sans remplir réellement le disque. Une simulation n’est pas un essai sur toutes les causes matérielles ; préciser cette limite.

## Recommandations à présenter pendant le développement

Adopter les contrats P02/P03 pour rendre visibles les données invalides ; garder les corrections d’analyse vide et de création de dossier ; préserver les attributs ZIP pertinents ; identifier explicitement chaque résultat. Ces propositions améliorent l’existant et doivent apparaître dans le compte rendu du lot concerné.

La reprise proposée est volontairement limitée à un dossier ouvert explicitement. Pas de catalogue global, de synchronisation ou de gestion multi-utilisateur à développer pour V1. La durée de conservation et le nettoyage automatique ne sont pas spécifiés : aucune suppression automatique des traitements utilisateur.

## Prompt de démarrage à copier dans Claude Code

```text
Nous démarrons l’application BPMN-Script Windows Svelte 5 / Rust / Tauri.

Lis le dossier de préparation joint : README.md, CLAUDE.md, product-spec.md,
ui-spec.md, architecture.md et implementation-plan.md, puis le rapport de
qualification dans references/. Distingue les exigences validées, les faits
observés et les propositions P01–P07. Les instructions actuelles de cette
conversation priment sur les références historiques.

Commence par M0 uniquement : inspecte l’état réel du dépôt bpmn-script-rs,
les consignes applicables et les changements locaux. Compare cet état à la
référence datée 3c846fc sans supposer qu’il est encore identique. Reproduis
la compilation avec l’outillage approprié et les tests de qualification
sur des copies, en préservant le code et les données d’origine.

Le paquet de qualification compile sa propre référence. Si tu qualifies le
dépôt courant, construis son binaire et indique ce chemin au pilote avec
--binary. Ne présente pas le test de la référence incluse comme un test
du code courant. Les trente scénarios caractérisent aussi des défauts :
ne les corrige pas et ne change pas leurs attentes pendant M0.

Livre un état des lieux, les versions utilisées, les tests réellement
exécutés, leurs résultats, les divergences et le contenu précis du lot M1
que tu proposes. Si un outil manque, explicite le blocage et avance sur les
vérifications indépendantes. Ne commence pas l’interface ni un grand refactor
dans cette première livraison. Ne modifie pas le chantier Python pédagogique.
```

Ce prompt limite intentionnellement le premier échange à une vérification du point de départ. Les lots suivants sont décrits et prêts à être confiés séparément, sans demander à Claude de concevoir toute l’application à partir d’un seul message.

## Format de compte rendu attendu

Pour chaque lot : problème traité, comportement obtenu, périmètre des fichiers modifiés, tests et observations, écarts volontaires avec leur référence P/R/A, limites restantes, contenu du lot suivant. Si le travail comporte un choix produit proposé, conserver son statut et décrire son effet concret. Un compte rendu de réussite ne remplace pas les preuves d’exécution.
