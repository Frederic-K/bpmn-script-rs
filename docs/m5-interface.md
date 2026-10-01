# M5 — Interface Svelte / Tauri

26 septembre 2026 · lot M5 de [implementation-plan.md](dossier-claude/implementation-plan.md), suite de [m4-traitement.md](m4-traitement.md). Branche `claude/m5-interface`.

## Consignes appliquées

Svelte 5 en **JavaScript uniquement** : aucun fichier `.ts`, aucun `lang="ts"`, aucune configuration TypeScript. KISS/YAGNI : aucun store global, aucune mémoïsation, aucune bibliothèque d'interface, aucune couche générique autour de Tauri. Noms explicites et commentaires `//` réservés aux choix non évidents.

## Organisation

| Fichier | Rôle |
|---|---|
| `ui/src/App.svelte` | Seul détenteur de l'état : `etat` (retourné par le moteur, seule source de vérité), écran affiché, action en cours, erreur. Fonction `executer` : une action à la fois, erreur affichée, état remplacé par celui du moteur |
| `ui/src/moteur.js` | Une fonction par commande Tauri, sans couche générique |
| `ui/src/format.js` | Date, nom de fichier, libellés des statuts |
| `ui/src/composants/Etape*.svelte` | Un composant par étape : Source, Correspondances, Analyse, Décisions, Résultat |
| `ui/src/composants/Accueil.svelte` | Nouveau traitement et reprise |
| `Etapes`, `BilanTraitement`, `Message`, `Fichier` | Navigation, panneau de bilan, message de statut, fichier avec ses actions |
| `ui/src/style.css` | Tailwind CSS v4 : couleurs de la maquette validée (`@theme`) et leurs valeurs sombres (`data-theme="dark"`). Migration depuis un CSS classique demandée après M6 |
| `Bouton`, `Pastille`, `Chiffre` | Éléments répétés, en composants plutôt qu'en classes `@apply` (`Tableau` retiré avec les tableaux des écrans Analyse et Décisions) |
| `src-tauri/src/lib.rs` | Commandes Tauri : une par opération du moteur, dialogues, ouverture de fichiers |

Environ 1 150 lignes au total (Svelte, JavaScript et adaptateur Rust).

## Cohérence avec le moteur

- L'interface ne calcule aucune décision : elle affiche l'`Etat` et le `Bilan` retournés par Rust. Chaque opération renvoie l'état complet.
- Chaque opération transmet la révision affichée ; le moteur refuse une révision obsolète.
- Les boutons désactivés reflètent les préconditions, mais **le moteur refait tous les contrôles** à la génération.
- Au retour dans la fenêtre (après Excel), l'état est relu : un classeur modifié apparaît « à relire » et la génération est suspendue.

Ajouts au moteur (sans changement de règle) :
- `Etat.propositions` : l'aperçu de l'analyse ;
- `Etat.dernier_bilan` : enregistré dans le manifeste, pour l'affichage après une reprise ;
- `Bilan.lignes_a_examiner` : les lignes non admises et leur motif. Retiré depuis, avec le tableau de l'écran Décisions : l'écran n'affiche que les compteurs, le détail est dans le rapport de contrôle (voir [SESSION.md](../SESSION.md)).

## Robustesse et sécurité

| Garantie | Mise en œuvre |
|---|---|
| Double action | Côté interface, `executer` ignore une action tant qu'une autre est en cours et désactive les boutons. Côté Rust, le verrou n'est jamais attendu : une seconde demande reçoit `operation_en_cours`. Enfin, la révision attendue est contrôlée |
| Accès aux fichiers | L'interface n'a aucune permission de fichier, de dialogue ni de shell (`core:default` seulement) ; tout passe par les commandes Rust |
| Ouverture d'un fichier | `ouvrir_fichier`, `afficher_dans_dossier` et `enregistrer_copie` n'acceptent que des fichiers du traitement ouvert (chemin canonique vérifié, `..` refusé) |
| Contenu | CSP : `default-src 'self'` |
| Erreurs | Code, message et détails du moteur affichés tels quels (`role="alert"`), détails sélectionnables |

## Accessibilité

- Navigation au clavier : tous les éléments d'action sont des boutons natifs, et le focus est visible.
- Après chaque changement d'étape, le focus est placé sur le titre de l'étape.
- `aria-current="step"` sur l'étape affichée ; les actions en cours sont annoncées par une région `role="status"`.
- Les statuts sont toujours écrits en toutes lettres (pastilles et étiquettes), jamais portés par la seule couleur.
- En-têtes de tableau `scope="col"` et légendes de tableau (tant que l'interface avait des tableaux).
- Étapes en barre horizontale sous l'en-tête (à l'origine, colonne de gauche) ; contenu et bilan sur une colonne en petite fenêtre (moins de 980 px).
- Le compilateur Svelte n'émet aucun avertissement d'accessibilité.

## Choix techniques

- **Versions exactes** : Svelte 5.57.1, Vite 8.3.1, @sveltejs/vite-plugin-svelte 7.3.1, @tauri-apps/api 2.11.1 et @tauri-apps/cli 2.11.5, verrouillées par `package-lock.json`. Tauri 2 (branche stable, la 3 n'étant qu'en alpha), avec `tauri-plugin-dialog` et `tauri-plugin-opener`. Rust 1.89.0 inchangé.
- **Espace de travail Cargo** : `src-tauri` partage le `Cargo.lock` du moteur. Aucune version existante n'y est modifiée ; les paquets de Tauri y sont seulement ajoutés.
- **Dialogues ouverts côté Rust**, dans Documents plutôt que dans « Récents » : aucun paquet JavaScript de plugin.
- **Installateur Windows** : NSIS (`npm run tauri build`).

## Preuves d'exécution (Linux, affichage virtuel Xvfb)

| Contrôle | Résultat |
|---|---|
| `cargo test --workspace --locked` | 81 tests : 26 unitaires, 2 de bibliothèque, 33 CLI, 19 de traitement (dont l'état fourni à l'interface après reprise), 1 de l'adaptateur (fichiers hors du traitement refusés) |
| `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check` | Propres |
| `npm run build` | Aucun avertissement (compilation Svelte, accessibilité comprise) |
| CLI historique, 30 scénarios de qualification | Identiques à M4 |
| **Test de bout en bout** `tests/interface/parcours.py` | 3 exécutions sur 3 réussies, 14 contrôles chacune (détail ci-dessous) |

Le test de bout en bout pilote par WebDriver (tauri-driver et WebKitWebDriver) l'application compilée avec son interface embarquée et le vrai moteur. Les dialogues GTK natifs sont remplis au clavier (xdotool) et les classeurs sont modifiés comme dans Excel (openpyxl). Il vérifie :
- la création refusée sans source ;
- le choix réel du fichier et du dossier ;
- l'inventaire exact ;
- le focus sur le titre ;
- une cellule invalide signalée et rien d'adopté ;
- les noms inconnus signalés ;
- l'aperçu de l'analyse (aujourd'hui ses trois compteurs) ;
- le motif d'une ligne en attente (aujourd'hui vérifié dans le rapport de contrôle, l'écran n'ayant plus de tableau) et le message d'un OUI ignoré ;
- **un double clic sur « Générer » qui ne produit qu'une tentative** ;
- le contenu exact du SGX produit ;
- un classeur modifié après lecture, qui suspend la génération ;
- la fermeture puis la reprise avec le résultat retrouvé.

## Limites

- Compilé et testé sous Linux (WebKitGTK). **Non compilé ni lancé sous Windows** (WebView2, installateur NSIS) : c'est l'objet de M6 (A16).
- L'ouverture d'un classeur dans Excel (application associée) n'est pas testée automatiquement ; en cas d'échec, le message propose « Afficher dans le dossier ».
- L'agrandissement du texte et la lecture par un lecteur d'écran n'ont pas été vérifiés manuellement.
- SAPHIR non rejoué, aucun import Signavio.

## Lot suivant — M6 (qualification)

Build release et installateur Windows, lancement sur un poste cible (WebView2), relance des tests sous Windows, essai sur une copie de SAPHIR, import dans un emplacement Signavio de test par l'opérateur (A16, A17).
