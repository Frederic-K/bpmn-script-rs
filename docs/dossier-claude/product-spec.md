# BPMN-Script — spécification produit V1

Version 1.0 · 26 septembre 2026 · dossier de préparation au développement.

## Statut et ordre de lecture

Ce dossier distingue **VALIDÉ** (choix explicitement établis dans la passation), **OBSERVÉ** (code ou tests) et **PROPOSÉ** (recommandation de conception). L’autorisation de préparer le dossier ne transforme pas les propositions en décisions métier déjà approuvées. Les propositions constituent une base concrète pour les lots de développement ; les conserver identifiables jusqu’à leur arbitrage. Les instructions actuelles de l’utilisateur priment.

Ce document porte les règles produit et le registre des propositions. [ui-spec.md](ui-spec.md) décrit leur présentation ; [architecture.md](architecture.md) leur mise en œuvre ; [implementation-plan.md](implementation-plan.md) les lots et le prompt initial. [CLAUDE.md](CLAUDE.md) est la consigne de travail. Une divergence entre documents doit être résolue explicitement, sans choisir silencieusement la règle la plus permissive.

## Besoin et périmètre validés

L’opérateur harmonise les noms de swimlanes de plusieurs modèles contenus dans **un seul SGX par traitement**. Il garde le contrôle des décisions métier. Un arbitre peut recevoir les propositions Excel, répondre hors de l’application et retourner son classeur. Ces rôles ne sont pas des comptes applicatifs.

L’application Windows repose sur Svelte 5, Rust et Tauri. Elle doit rendre le traitement compréhensible sans mémoriser les noms de fichiers et leurs dossiers. Excel reste le support de correspondance et d’arbitrage ; aucun tableau éditable intégré ne le remplace en V1. L’import du résultat dans Signavio est manuel.

Parcours : choisir un SGX → inventorier → renseigner les correspondances dans Excel → analyser sans modifier → recueillir OUI/NON → contrôler les décisions et les occurrences → produire un autre SGX.

Hors V1 : dictionnaire Signavio (besoin non défini et absent du code), éditeur graphique BPMN, API Signavio, multi-SGX simultané, comptes, serveur, base de données, synchronisation cloud, plateforme BPMN Governance. La portabilité sur clé USB est un objectif à qualifier, pas une capacité acquise.

## État observé

Le dépôt Rust local examiné est au commit `3c846fc`. Il contient un CLI, sans Svelte/Tauri ni tests unitaires Rust. Le code compile avec Rust 1.88.0 et le verrou Cargo fourni. Trente scénarios externes ont été exécutés : vingt-deux concordent sur les contenus métier et codes de sortie ; huit présentent des écarts. Styles Excel et attributs ZIP sont comparés séparément.

Sur une copie de SAPHIR : 241 modèles, 848 occurrences, 289 noms distincts ; inventaire, analyse et treize renommages expérimentaux concordent entre Python et Rust. Aucun import Signavio n’a été réalisé. Voir [le rapport](references/qualification-rust.md). Ces mesures décrivent cet export et ne sont jamais à coder en dur dans l’application.

## Contrat historique à préserver

| ID | Règle |
|---|---|
| R01 | Source intacte ; génération dans un autre fichier ; aucune écriture par API Signavio. |
| R02 | Identité d’un modèle : chemin interne complet, jamais son seul nom affiché. Reconnaissance par suffixe `model_1_.json`, metadata associée `model_meta.json`. |
| R03 | Parcours récursif des `childShapes`, uniquement `stencil.id == "Lane"`. Inventaire des noms textuels non vides après retrait des espaces périphériques ; doublons conservés. Aucun nettoyage automatique des espaces internes. |
| R04 | Nouveau nom vide : pas de demande. Ancien nom inconnu : aucun impact. Dernière correspondance non vide retenue pour un même ancien nom. Ancien nom lu sans nettoyage supplémentaire. |
| R05 | Une proposition par couple chemin de modèle / nom de lane. Les modèles homonymes restent distincts. « Nombre de flux » historique compte les noms distincts, pas les modèles distincts. |
| R06 | Analyse recalculée depuis la source et les correspondances courantes ; elle n’autorise aucune production à elle seule. |
| R07 | Une décision textuelle est normalisée par retrait des espaces périphériques et mise en majuscules. Seul OUI avec égalité exacte de `(chemin, flux, ancien nom, nouveau nom, occurrences)` est admissible. |
| R08 | OUI non conforme : ignoré avec motif, sans bloquer les autres OUI conformes. NON : refus. Autre valeur ou absence : attente. Ligne absente : aucune autorisation implicite. |
| R09 | Conserver ordre et doublons des décisions admises. Appliquer séquentiellement en mémoire ; recomptabiliser chaque opération. Pas de dédoublonnage ni de renommage simultané introduit silencieusement. |
| R10 | Une divergence de décompte interdit toute nouvelle archive finale pour cette tentative. Au moins une décision admissible est nécessaire pour produire. Ancien nom = nouveau nom reste compté comme dans la référence. |
| R11 | Préserver les données inconnues et les contenus non concernés. La sérialisation peut différer ; pas de promesse d’identité binaire globale ZIP/XLSX. |
| R12 | « Décision admise » et « Archive générée » sont deux résultats distincts. |

## Formats d’échange

Conserver les feuilles et les colonnes suivantes dans cet ordre. Première ligne : en-têtes ; données à partir de la ligne 2. Les chemins techniques restent disponibles dans les classeurs, même si l’interface les masque initialement.

| Fichier historique | Feuille et colonnes |
|---|---|
| `inventaire_swimlanes.xlsx` / `correspondance_swimlanes.xlsx` | Correspondance : Nom actuel ; Nouveau nom ; Occurrences ; Nombre de flux ; Flux concernés ; Flux avec occurrences multiples |
| `analyse_modifications.xlsx` / `validation_modifications.xlsx` | Analyse : Flux ; Nom actuel ; Nouveau nom ; Occurrences ; Validation ; Fichier modèle |
| `controle_validation.xlsx` | Contrôle : Ligne validation ; Flux ; Nom actuel ; Nouveau nom saisi ; Nouveau nom attendu ; Occurrences ; Validation ; Résultat ; Motif ; Fichier modèle |

Conserver les JSON `resultats`, `synthese`, `correspondances`, `analyse_modifications`, `modifications_validees`, `modifications_ignorees`. Le dernier recense les OUI non conformes ; le contrôle Excel contient aussi les refus et attentes. Ne pas prétendre que les lignes absentes figurent dans ce rapport historique : les signaler dans le bilan de l’application.

## Registre des propositions de V1

| ID | Proposition de référence | Écart ou besoin couvert |
|---|---|---|
| P01 | Un dossier de traitement choisi par l’utilisateur, une copie source immuable, des instantanés d’entrées et un petit manifeste JSON versionné. Réouverture explicite de ce dossier. | Reprise entre deux journées, sans base de données ni liste de traitements récente obligatoire. |
| P02 | Import Excel strict sur les champs utilisés : texte pour les noms, chemins et décisions ; occurrences numériques entières non négatives ; rejet des formules, booléens et erreurs. | Les conversions actuelles Python/Rust changent parfois les décisions. Voir les niveaux de rejet ci-dessous. |
| P03 | JSON de modèle reconnu mal formé : diagnostic précis et blocage de la production du traitement. | Éviter un inventaire partiel silencieux. Champs optionnels absents selon les conventions historiques : traités comme vides ; types explicitement invalides : anomalie. |
| P04 | Résultat structuré, sorties liées à une tentative, écriture temporaire puis finalisation sans écraser une sortie existante. | Ancien SGX visible après refus, erreur en cours d’écriture, code de sortie ambigu. |
| P05 | Conserver l’analyse vide et la création automatique des dossiers. | Améliorations Rust à garder ; ne pas reproduire le défaut Python de plage Excel vide. |
| P06 | Préserver les attributs ZIP pertinents, champs supplémentaires compatibles et commentaires ; documenter les transformations nécessaires au recalcul des en-têtes. | Pertes observées dans le portage ; qualification Signavio toujours nécessaire. |
| P07 | Svelte avec TypeScript/Vite, Tauri 2 et moteur Rust partagé entre CLI et application. | Proposition technique détaillée dans architecture.md ; versions exactes à verrouiller lors du scaffold. |

**Précisions P02.** Une feuille attendue absente, des en-têtes attendus déplacés/manquants ou un classeur illisible refusent l’import ; conserver la dernière version adoptée. Les colonnes supplémentaires après les colonnes attendues sont ignorées. Les lignes entièrement vides sont ignorées.

Pour les correspondances, nouveau nom vide ou composé d’espaces : ligne sans demande. Sur les lignes avec demande, ancien et nouveau noms doivent être textuels ; une erreur refuse l’adoption du classeur entier avec liste des cellules à corriger. Conserver les doublons selon R04 et signaler les doublons de façon informative. Une formule dans une colonne utilisée refuse l’import même si son cache semble vide. Les colonnes informatives de l’inventaire ne déterminent jamais les impacts.

Pour les décisions, contrôler les six champs utilisés sans convertir les valeurs. Une ligne OUI aux champs invalides est ignorée avec un motif de cellule, les autres lignes restent traitables. Un nombre de valeur 1.0 est admissible comme occurrence 1 ; texte « 1 », booléen, valeur négative ou fractionnaire ne l’est pas. Une validation non textuelle, y compris une formule, n’accorde aucun droit et reçoit un diagnostic. NON et absence de OUI ne deviennent jamais une autorisation. Conserver la valeur brute lisible dans le rapport ; ne pas évaluer les formules.

Ces règles constituent une évolution explicite : elles ne doivent pas être présentées comme une reproduction exacte du Python, ni être introduites dans le même lot que l’extraction sans changement de comportement.

## Critères d’acceptation

Les critères A01–A08 portent les invariants ; A09–A15 précisent les propositions de V1.

| ID | Cas | Résultat attendu |
|---|---|---|
| A01 | Lanes imbriquées, doublons, nom vide, tâche homonyme | Inventaire des seules lanes non vides ; aucune mutation source. |
| A02 | Modèles homonymes | Deux identités conservées ; ciblage par chemin. |
| A03 | Analyse seule ou aucune décision admissible | Aucun nouveau SGX ; résultat explicite. |
| A04 | Mélange OUI/NON/attente, retour partiel | Seules les décisions présentes, OUI et conformes sont admissibles. |
| A05 | Altération de chacun des cinq champs de proposition | Ligne OUI ignorée avec motif ; autres lignes conformes traitables. |
| A06 | Chaîne, doublon de validation, permutation | Application séquentielle, recomptage ; aucune sortie finale en cas de divergence. |
| A07 | Production avec pièces jointes et propriétés inconnues | Entrées et contenus non concernés préservés ; source inchangée avant/après. |
| A08 | Tout échec | Pas de faux succès ni d’altération du SGX source. |
| A09 | Cellules numériques, formule avec/sans cache, erreur, booléen | P02 appliquée avec diagnostics et sans coercition silencieuse. |
| A10 | Modèle reconnu invalide | P03 ; aucun résultat partiel présenté comme exhaustif. |
| A11 | Fermer puis rouvrir un dossier, original déplacé | Reprise grâce à la copie source contrôlée ; pas de dépendance à l’emplacement original. |
| A12 | Correspondances modifiées, décision réimportée, instantané altéré | Résultats dépendants invalidés ; contrôle recalculé avant production. |
| A13 | Ancienne sortie, erreur d’écriture, interruption, double clic | Sorties liées à la tentative ; aucune archive partielle annoncée comme finale ; pas de double traitement concurrent. |
| A14 | Analyse sans impact, dossier absent, Excel ouvert/verrouillé | Résultat vide normal, création de dossier, ou erreur exploitable conservant la dernière version valide. |
| A15 | Parcours au clavier, erreurs, retour d’arbitrage | Étape et action suivante compréhensibles ; Excel reste édité hors de l’application. |
| A16 | Livraison Windows | Build release et lancement sur poste cible documentés ; WebView2 et installation qualifiés. |
| A17 | Import Signavio manuel | Import dans un emplacement de test et vérification de modèles touchés/non touchés réussis ; résultat consigné par l’opérateur. |

A16 et A17 sont des jalons de livraison non encore franchis. Les tests historiques restent des preuves de caractérisation, pas des oracles qui imposent de reproduire leurs défauts.
