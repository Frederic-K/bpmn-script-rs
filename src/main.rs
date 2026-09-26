// bpmn-script-rs : CLI historique.
//
// À lancer depuis le dossier qui contient input/, work/ et output/ :
// inventaire -> correspondance humaine -> dry-run -> validation humaine -> SGX modifié.
//
// Code de sortie : 0 = étape terminée normalement, 1 = erreur bloquante,
// 2 = décisions fournies mais aucun SGX produit (aucune décision admissible
// ou recomptage divergent).

use std::path::Path;

use bpmn_script_rs::{Chemins, Statut, executer};

fn main() {
    let chemins = Chemins::historiques(Path::new(""));
    let bilan = match executer(&chemins, &mut |message| println!("{message}")) {
        Ok(bilan) => bilan,
        Err(erreur) => {
            eprintln!("[ERREUR] {erreur}");
            std::process::exit(1);
        }
    };

    let (conclusion, code) = match bilan.statut {
        Statut::InventaireTermine => ("inventaire produit, correspondances attendues".to_string(), 0),
        Statut::AnalyseSansImpact => (
            "aucun changement proposé, vérifiez les nouveaux noms renseignés".to_string(),
            0,
        ),
        Statut::DecisionsAttendues => ("analyse produite, décisions attendues".to_string(), 0),
        Statut::AucuneDecisionAdmissible => (
            "aucune décision admissible : aucun nouveau SGX n'a été créé".to_string(),
            2,
        ),
        Statut::ControleBloquant if !bilan.contradictions.is_empty() => (
            "décisions contradictoires (OUI et NON pour la même proposition) : aucun nouveau SGX n'a été créé".to_string(),
            2,
        ),
        Statut::ControleBloquant => (
            "les modifications ne correspondent pas aux occurrences attendues : aucun nouveau SGX n'a été créé".to_string(),
            2,
        ),
        // Étape intermédiaire du dossier de traitement, jamais retournée par ce mode.
        Statut::ProductionPossible => ("décisions conformes".to_string(), 0),
        Statut::SgxProduit => (
            format!(
                "SGX produit et vérifié : {} ({} modèle(s), {} occurrence(s))",
                bilan
                    .sgx_produit
                    .as_deref()
                    .unwrap_or(Path::new(""))
                    .display(),
                bilan.modeles_modifies,
                bilan.occurrences_modifiees
            ),
            0,
        ),
    };
    for divergence in &bilan.divergences {
        println!("[ATTENTION] {divergence}");
    }
    println!("[RÉSULTAT] {conclusion}");
    std::process::exit(code);
}
