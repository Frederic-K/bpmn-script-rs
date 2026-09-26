// Mise en forme des valeurs retournées par le moteur.

export function dateHeure(secondes) {
  return new Date(secondes * 1000).toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" });
}

export function nomFichier(chemin) {
  return chemin.split(/[\\/]/).pop();
}

// Classes Tailwind des pastilles de statut, écrites en entier pour que Tailwind les détecte.
export const PASTILLES = {
  ok: "pastille bg-ok-doux text-ok",
  alerte: "pastille bg-alerte-doux text-alerte",
  erreur: "pastille bg-erreur-doux text-erreur",
  neutre: "pastille bg-neutre-doux text-encre-2",
};

// Libellé et couleur de pastille de chaque Statut du moteur (src/workflow.rs).
export const STATUTS = {
  SgxProduit: { libelle: "SGX produit", couleur: "ok" },
  AucuneDecisionAdmissible: { libelle: "Aucune décision admissible", couleur: "neutre" },
  ControleBloquant: { libelle: "Contrôle bloquant", couleur: "erreur" },
  ProductionPossible: { libelle: "Génération possible", couleur: "ok" },
  DecisionsAttendues: { libelle: "Décisions attendues", couleur: "neutre" },
  AnalyseSansImpact: { libelle: "Aucun changement", couleur: "neutre" },
  InventaireTermine: { libelle: "Inventaire", couleur: "neutre" },
};

export function pastilleDuStatut(statut) {
  return PASTILLES[STATUTS[statut]?.couleur ?? "neutre"];
}
