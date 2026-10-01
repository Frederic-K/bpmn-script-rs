// Mise en forme des valeurs retournées par le moteur.

export function dateHeure(secondes) {
  return new Date(secondes * 1000).toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" });
}

export function nomFichier(chemin) {
  return chemin.split(/[\\/]/).pop();
}

// Couleur de pastille (composant Pastille) du statut d'une tentative ; le
// libellé est dans textes.statuts.
export const COULEURS_DES_STATUTS = {
  SgxProduit: "ok",
  AucuneDecisionAdmissible: "neutre",
  ControleBloquant: "erreur",
};
