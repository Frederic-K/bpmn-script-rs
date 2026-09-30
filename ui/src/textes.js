// Textes de l'interface, en français, regroupés par écran. Un texte à
// paramètres est une fonction. Les messages d'erreur et les motifs de contrôle
// sont rédigés par le moteur Rust : ils sont affichés tels quels et ne figurent
// pas ici.

export const textes = {
  application: {
    nom: "BPMN-Script",
    afficherDossier: "Afficher le dossier du traitement",
    fermer: "Fermer le traitement",
    themeSombre: "Thème sombre",
    etiquetteErreur: "Erreur",
  },

  // Libellés affichés pendant une action (« Lecture des correspondances… »).
  actions: {
    choixFichier: "Choix du fichier",
    choixDossier: "Choix du dossier",
    choixClasseur: "Choix du classeur",
    creation: "Copie et inventaire du SGX",
    ouverture: "Ouverture et vérification du traitement",
    fermeture: "Fermeture",
    ouvertureDossier: "Ouverture du dossier",
    ouvertureClasseur: "Ouverture du classeur",
    ouvertureRapport: "Ouverture du rapport",
    lectureCorrespondances: "Lecture des correspondances",
    preparationAnalyse: "Préparation de l'analyse",
    lectureDecisions: "Lecture et contrôle des décisions",
    generation: "Génération et vérification du SGX",
    copie: "Enregistrement de la copie",
  },

  etapes: {
    navigation: "Étapes du traitement",
    source: "Source",
    correspondances: "Correspondances",
    analyse: "Analyse",
    decisions: "Décisions",
    resultat: "Résultat",
    aRelire: "à relire",
    aActualiser: "à actualiser",
    terminee: "terminée",
    enCours: "en cours",
  },

  // Statuts du moteur (Statut dans src/workflow.rs).
  statuts: {
    SgxProduit: "SGX produit",
    AucuneDecisionAdmissible: "Aucune décision admissible",
    ControleBloquant: "Contrôle bloquant",
    ProductionPossible: "Génération possible",
    DecisionsAttendues: "Décisions attendues",
    AnalyseSansImpact: "Aucun changement",
    InventaireTermine: "Inventaire",
  },

  accueil: {
    titre: "Harmoniser les noms de swimlanes d'un export Signavio",
    introduction:
      "Un traitement réunit un SGX, les classeurs Excel de correspondance et de décision, et les résultats. Il est enregistré dans un dossier que vous choisissez et peut être repris plus tard.",
    nouveauTitre: "Nouveau traitement",
    nouveauExplication:
      "Choisissez un export .sgx et le dossier qui accueillera le traitement. Le fichier choisi n'est jamais modifié : l'application travaille sur une copie.",
    changer: "Changer",
    choisirSgx: "Choisir le fichier SGX",
    choisirDossier: "Choisir le dossier des traitements",
    titreDialogueDossier: "Dossier qui accueillera le traitement",
    inventorier: "Inventorier les lanes",
    reprendreTitre: "Reprendre un traitement",
    reprendreExplication:
      "Ouvrez le dossier d'un traitement existant. Son état est vérifié : copie de la source, classeurs lus et résultats.",
    ouvrirTraitement: "Ouvrir un dossier de traitement",
    titreDialogueTraitement: "Ouvrir un dossier de traitement",
  },

  source: {
    titre: "Source",
    copie:
      "L'application travaille sur une copie contrôlée par empreinte, dans le dossier du traitement. L'original peut être déplacé sans gêner la reprise.",
    modeles: "modèles reconnus",
    occurrences: "occurrences de lanes",
    noms: "noms distincts",
    rienAHarmoniser: "Les modèles de ce SGX ne contiennent aucune lane nommée : il n'y a rien à harmoniser.",
    suivant: "Préparer les correspondances",
  },

  correspondances: {
    titre: "Correspondances",
    consigne: "Renseignez les nouveaux noms dans Excel. Une cellule Nouveau nom vide laisse le nom inchangé.",
    ouvrir: "Ouvrir le classeur",
    etiquetteARelire: "À relire",
    aRelire: "Ce classeur a changé depuis sa dernière lecture. Relisez-le pour actualiser le traitement.",
    etiquetteInfo: "Info",
    fermerAvantLecture: "Enregistrez puis fermez le classeur avant de le lire.",
    etiquetteLu: "Lu",
    lus: (nombre) => `${nombre} nom(s) avec une demande de renommage ont été lus.`,
    etiquetteUtilise: "Utilisé",
    lecture: (date, fichier, classeurDuTraitement) =>
      classeurDuTraitement
        ? `Correspondances lues le ${date} depuis le classeur du traitement.`
        : `Correspondances importées le ${date} depuis « ${fichier} ». Ce fichier n'est plus relu : pour corriger, modifiez le classeur du traitement puis relisez-le.`,
    etiquetteNonAdopte: "Non adopté",
    nonAdopte: (date) => `Classeur non adopté. Les correspondances lues le ${date} restent utilisées.`,
    etiquetteAVerifier: "À vérifier",
    nomsInconnus: "Ces noms n'existent pas dans l'inventaire et n'auront aucun effet (faute de frappe ?) :",
    nomsEnDoublon: "Ces noms sont renseignés plusieurs fois ; seule la dernière ligne est retenue. Vérifiez que c'est voulu :",
    lire: "Lire les correspondances",
    relire: "Relire les correspondances",
    copier: "Enregistrer une copie",
    nomCopie: (traitement) => `correspondances_${traitement}.xlsx`,
    copieSansEcrasement:
      "« Enregistrer une copie » : le classeur tel qu'il est, à l'emplacement de votre choix. Choisissez un nouveau nom : aucun fichier existant n'est remplacé.",
    etiquetteCopie: "Copie",
    copieEnregistree: (chemin) => `Copie enregistrée : ${chemin}`,
    importer: "Importer un classeur de correspondances",
    importerExplication:
      "Un classeur importé remplace les correspondances lues et devient le classeur du traitement ; l'analyse et les décisions seront à refaire.",
    voirAnalyse: "Voir l'analyse",
    preparerAnalyse: "Préparer l'analyse",
  },

  analyse: {
    titre: "Analyse",
    etiquetteNonPreparee: "À faire",
    nonPreparee: "L'analyse n'est pas encore préparée : lisez les correspondances puis préparez l'analyse.",
    etiquetteResultat: "Résultat",
    aucunChangement: "Aucun changement proposé. Vérifiez les nouveaux noms renseignés.",
    retourCorrespondances: "Revenir aux correspondances",
    etiquetteAActualiser: "À actualiser",
    aActualiser:
      "Dernière analyse — à actualiser : le classeur de correspondance a changé. Relisez les correspondances, puis préparez l'analyse avant de reprendre la validation.",
    apercu: "Aperçu en lecture seule des changements proposés. Rien n'est encore modifié.",
    propositions: "propositions",
    occurrences: "occurrences visées",
    modeles: "modèles concernés",
    legende: "Une ligne par modèle et par nom ; des flux homonymes restent distincts par leur fichier modèle.",
    colonneFlux: "Flux",
    colonneNomActuel: "Nom actuel",
    colonneNouveauNom: "Nouveau nom",
    colonneOccurrences: "Occ.",
    colonneFichier: "Fichier modèle",
    suivant: "Passer aux décisions",
  },

  decisions: {
    titre: "Décisions",
    consigne:
      "Chaque proposition attend OUI ou NON dans la colonne Validation. Seules les lignes OUI conformes seront appliquées ; une ligne sans réponse n'autorise rien.",
    etiquetteARelire: "À relire",
    aRelire:
      "Le classeur de décision a changé depuis sa dernière lecture. Relisez-le pour actualiser le traitement ; la génération est suspendue d'ici là.",
    correspondancesARelire:
      "Le classeur de correspondance a changé depuis sa dernière lecture. Relisez les correspondances et préparez l'analyse avant de valider.",
    localTitre: "Je valide moi-même",
    localConsigne:
      "Ouvrez le classeur de décision et renseignez uniquement la colonne Validation (OUI ou NON). Enregistrez et fermez Excel, puis lisez le classeur.",
    ouvrir: "Ouvrir le classeur de décision",
    lire: "Lire le classeur de décision",
    externeTitre: "Je fais valider ailleurs (autre personne, réunion…)",
    externeConsigne:
      "Enregistrez une copie : elle contient les propositions, sans aucune réponse. Seule la colonne Validation doit y être remplie, sans modifier les autres colonnes.",
    copier: "Enregistrer une copie",
    nomCopie: (traitement) => `validation_${traitement}.xlsx`,
    copieSansEcrasement: "Choisissez un nouveau nom : aucun fichier existant n'est remplacé.",
    etiquetteCopie: "Copie",
    copieEnregistree: (chemin) => `Copie enregistrée : ${chemin}`,
    importerConsigne:
      "Une fois la copie remplie, importez-la. Elle remplace toutes les décisions lues jusqu'ici (aucune fusion) et devient le classeur de décision du traitement.",
    importer: "Importer un classeur de décision",
    etiquetteUtilisees: "Utilisées",
    lecture: (date, fichier, classeurDuTraitement) =>
      classeurDuTraitement
        ? `Décisions lues le ${date} depuis le classeur de décision du traitement.`
        : `Décisions importées le ${date} depuis « ${fichier} ». Ce fichier n'est plus relu : pour corriger, modifiez le classeur de décision puis relisez-le.`,
    etiquetteNonAdopte: "Non adopté",
    nonAdopte: (date) => `Classeur non adopté. Les décisions lues le ${date} restent utilisées.`,
    dernierControle: "Dernier contrôle",
    dernierControleAActualiser: "Dernier contrôle — à actualiser",
    admises: "Admises",
    occurrencesPrevues: (nombre) => `${nombre} occurrences prévues`,
    refusees: "Refusées",
    refuseesDetail: "NON",
    ignorees: "Ignorées",
    ignoreesDetail: "OUI non conforme",
    enAttente: "En attente",
    enAttenteDetail: "sans OUI ni NON",
    sansDecision: "Sans décision",
    sansDecisionDetail: "propositions absentes du classeur",
    legende: "Lignes non admises : elles ne seront pas appliquées.",
    colonneLigne: "Ligne",
    colonneNomActuel: "Nom actuel",
    colonneNouveauNom: "Nouveau nom",
    colonneResultat: "Résultat",
    colonneMotif: "Motif",
    etiquetteBloquant: "Bloquant",
    contradictions:
      "Des propositions ont reçu à la fois OUI et NON. Aucun nouveau SGX ne peut être créé : gardez une seule réponse par proposition, puis relisez le classeur.",
    bloquant:
      "Les modifications ne correspondent pas aux occurrences attendues. Aucun nouveau SGX ne peut être créé : corrigez les décisions ou les correspondances.",
    etiquetteAucunSgx: "Aucun SGX",
    aucuneAdmissible: "Aucune décision admissible : aucun SGX ne peut être généré.",
    etiquetteControle: "Contrôlé",
    controle:
      "Contrôle terminé, aucune anomalie bloquante : les décisions correspondent à l'analyse et le renommage testé en mémoire donne le nombre d'occurrences attendu.",
    seraApplique: (lignes, occurrences) =>
      `La génération appliquera ${lignes} décision(s) admise(s), soit ${occurrences} occurrence(s). Aucune autre ligne n'est appliquée.`,
    generer: "Générer le SGX modifié",
    ouvrirRapport: "Ouvrir le rapport de contrôle",
  },

  resultat: {
    titre: "Résultat",
    etiquetteInterrompue: "Interrompue",
    interrompue:
      "Une production a été interrompue avant d'être enregistrée. Son contenu n'est pas confirmé et n'est jamais présenté comme un résultat :",
    etiquetteARelire: "À relire",
    entreesModifiees:
      "Un classeur a changé depuis sa dernière lecture : ce résultat correspond aux dernières entrées lues, pas au contenu actuel du classeur.",
    etiquetteProduit: "Produit",
    produit: "SGX produit et vérifié : relu après écriture, seules les lanes validées diffèrent de la source.",
    etiquetteAttention: "Attention",
    fichierAbsent: "Ce fichier a été supprimé depuis sa production.",
    fichierModifie: "Ce fichier a été modifié depuis sa production : il ne correspond plus au résultat vérifié.",
    enregistrerCopie: "Enregistrer une copie",
    copieSansEcrasement: "Choisissez un nouveau nom : aucun fichier existant n'est remplacé.",
    etiquetteCopie: "Copie",
    copieEnregistree: (chemin) => `Copie enregistrée : ${chemin}`,
    modeles: "modèles concernés par la production",
    occurrences: "occurrences traitées",
    tentative: (numero) => `tentative ${numero}`,
    afficher: "Afficher le SGX dans le dossier",
    ouvrirRapport: "Ouvrir le rapport de contrôle",
    etiquetteSuivante: "Étape suivante",
    suivante:
      "Importez ce fichier dans un emplacement Signavio de test et vérifiez les modèles concernés avant tout import en production.",
    etiquetteAucunSgx: "Aucun SGX",
    aucunSgx: (cause) => `Aucun nouveau SGX n'a été créé : ${cause}`,
    causeAucuneAdmissible: "aucune décision admissible (lignes refusées, en attente ou non conformes).",
    causeBloquant: "le contrôle est bloquant (décisions contradictoires ou recomptage non conforme) ; voir le rapport de contrôle.",
    retourDecisions: "Revenir aux décisions",
    aucunResultat: "Aucun résultat pour les correspondances et décisions actuellement lues.",
    precedents: "Résultats précédents",
    precedentsExplication: "Non produits à partir des correspondances et décisions actuellement lues.",
    tentativeDatee: (numero, date) => `Tentative ${numero} · ${date}`,
  },

  bilan: {
    region: "Bilan du traitement",
    titre: "Traitement",
    modeles: "Modèles",
    occurrences: "Occurrences",
    noms: "Noms distincts",
    demandesLues: "Demandes lues",
    propositions: "Propositions",
    admises: "Décisions admises",
    aActualiser: "à actualiser",
    source: (nom) => `Source : ${nom}, copie contrôlée par empreinte.`,
    derniereTentative: "Dernière tentative",
    tentative: (numero) => `Tentative ${numero}`,
    entreesActuelles: "entrées actuelles",
    entreesPrecedentes: "entrées précédentes",
    interrompues: (nombre) => `${nombre} tentative(s) interrompue(s)`,
  },
};
