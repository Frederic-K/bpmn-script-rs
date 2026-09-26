// Mise en forme des valeurs retournées par le moteur.

export function dateHeure(secondes) {
  return new Date(secondes * 1000).toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" });
}

export function nomFichier(chemin) {
  return chemin.split(/[\\/]/).pop();
}

// Libellé et couleur de pastille (composant Pastille) de chaque Statut du moteur (src/workflow.rs).
export const STATUTS = {
  SgxProduit: { libelle: "SGX produit", couleur: "ok" },
  AucuneDecisionAdmissible: { libelle: "Aucune décision admissible", couleur: "neutre" },
  ControleBloquant: { libelle: "Contrôle bloquant", couleur: "erreur" },
  ProductionPossible: { libelle: "Génération possible", couleur: "ok" },
  DecisionsAttendues: { libelle: "Décisions attendues", couleur: "neutre" },
  AnalyseSansImpact: { libelle: "Aucun changement", couleur: "neutre" },
  InventaireTermine: { libelle: "Inventaire", couleur: "neutre" },
};
