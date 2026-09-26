# Dossier de développement pour Claude

BPMN-Script · Svelte 5 / Rust / Tauri · 26 septembre 2026 · version 1.0.

**Commencer par le prompt de M0 dans [implementation-plan.md](implementation-plan.md).** Il demande à Claude de vérifier le dépôt courant avant de modifier le moteur. Aucun développement ni envoi à Claude n’a été effectué pour préparer ce dossier.

## Contenu

| Document | Rôle |
|---|---|
| [product-spec.md](product-spec.md) | Périmètre, invariants R01–R12, propositions P01–P07, critères A01–A17 |
| [ui-spec.md](ui-spec.md) | Parcours guidé, échanges Excel, états, reprise et messages |
| [architecture.md](architecture.md) | Responsabilités, contrats du moteur, fichiers de traitement, cohérence et publication |
| [CLAUDE.md](CLAUDE.md) | Consignes de développement et preuves attendues |
| [implementation-plan.md](implementation-plan.md) | Lots M0–M6, tests et prompt initial prêt à copier |
| [references/qualification-rust.md](references/qualification-rust.md) | Résultats et limites des essais réellement exécutés |
| [references/qualification-rust-reproductible.zip](references/qualification-rust-reproductible.zip) | Sources de référence, pilote, résultats et reproduction de la qualification |

## À transmettre

Fournir l’archive complète à Claude avec accès au dépôt `bpmn-script-rs`, puis copier le prompt initial. Le dépôt observé lors de la revue était `E:\1-DEVELOPMENT\bpmn-script-rs`, commit `3c846fc` ; vérifier son état actuel. Les références contenues dans le paquet ne remplacent pas cette inspection.

Le dossier peut ensuite être intégré au dépôt dans `docs/development/`, et CLAUDE.md adapté à la racine. Ne pas écraser les consignes existantes ; les liens relatifs devront alors être adaptés. Aucun chemin particulier de ce poste n’est requis pour utiliser le paquet de qualification.

SAPHIR n’est pas inclus. Le pilote peut tester cet export localement sur copie si l’utilisateur le fournit ; les scénarios synthétiques restent disponibles sans lui. Les résultats déjà consignés pour SAPHIR ne constituent pas une validation des choix métier de renommage ou de l’import Signavio.

## Ce qui reste proposé

La reprise par dossier, l’import Excel strict, le blocage des JSON mal formés, la publication des sorties, les garanties ZIP et le scaffold détaillé sont explicités dans le registre P01–P07. Ils forment une base de travail, sans prétendre à un arbitrage utilisateur déjà acquis. Les règles métier historiques et le périmètre validé sont distingués de ces propositions.

La qualification disponible comprend un build de développement Rust et des essais comparatifs, pas une application Tauri livrable. Packaging Windows, rendu final et import Signavio restent des jalons. Le dictionnaire Signavio demeure hors V1 tant que son besoin n’est pas défini.
