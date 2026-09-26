//! Enchaînement historique : inventaire -> dry-run -> contrôle -> SGX modifié.
//! Les dossiers sont passés explicitement ; aucun répertoire courant implicite.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{Journal, Res, excel, regles, sgx};

/// Dossiers d'un traitement : source, saisies humaines, résultats.
pub struct Chemins {
    /// Contient l'unique fichier `.sgx` source.
    pub input: PathBuf,
    /// Contient `correspondance_swimlanes.xlsx` et `validation_modifications.xlsx`.
    pub work: PathBuf,
    /// Reçoit les JSON, Excel et SGX produits ; créé s'il est absent.
    pub output: PathBuf,
}

impl Chemins {
    /// Disposition historique `input/`, `work/`, `output/` sous `racine`.
    pub fn historiques(racine: &Path) -> Chemins {
        Chemins {
            input: racine.join("input"),
            work: racine.join("work"),
            output: racine.join("output"),
        }
    }
}

/// Exécute le workflow complet. Les messages de progression sont transmis à
/// `journal` ; une erreur bloquante est retournée.
pub fn executer(chemins: &Chemins, journal: &mut Journal<'_>) -> Res<()> {
    let out = &chemins.output;
    // Les messages affichent les chemins avec « / », comme la version historique.
    let affiche = |nom: &str| format!("{}/{nom}", out.display());

    let source = sgx::trouver_sgx(&chemins.input)?;
    journal(&format!("[OK] Fichier SGX sélectionné : {}", nom(&source)));
    fs::create_dir_all(out)?;

    // 1. Inventaire
    let resultats = sgx::extraire_lanes(&source, journal)?;
    journal(&format!(
        "[OK] Extraction terminée : {} occurrences",
        resultats.len()
    ));
    ecrire_json(&out.join("resultats.json"), &resultats)?;
    journal("[OK] resultats.json généré");

    let synthese = regles::synthetiser(&resultats);
    journal(&format!(
        "[OK] Synthèse terminée : {} lanes uniques",
        synthese.len()
    ));
    ecrire_json(&out.join("synthese.json"), &synthese)?;
    journal("[OK] synthese.json généré");

    excel::ecrire_inventaire(&out.join("inventaire_swimlanes.xlsx"), &synthese)?;
    journal(&format!(
        "[OK] Excel généré : {}",
        affiche("inventaire_swimlanes.xlsx")
    ));

    // 2. Dry-run
    let correspondance = chemins.work.join("correspondance_swimlanes.xlsx");
    if !correspondance.exists() {
        journal(
            "[INFO] Aucun fichier de correspondance trouvé : la préparation des modifications est ignorée",
        );
        return Ok(());
    }
    let correspondances =
        regles::retenir_correspondances(excel::lire_correspondances(&correspondance)?);
    journal(&format!(
        "[OK] Correspondances chargées : {}",
        correspondances.len()
    ));
    ecrire_json(&out.join("correspondances.json"), &correspondances)?;
    journal("[OK] correspondances.json généré");

    let analyse = regles::dry_run(&correspondances, &synthese);
    journal(&format!(
        "[OK] Dry-run préparé : {} impacts flux/lane",
        analyse.len()
    ));
    ecrire_json(&out.join("analyse_modifications.json"), &analyse)?;
    journal("[OK] analyse_modifications.json généré");

    excel::ecrire_analyse(&out.join("analyse_modifications.xlsx"), &analyse)?;
    journal(&format!(
        "[OK] Excel d'analyse généré : {}",
        affiche("analyse_modifications.xlsx")
    ));

    // 3. Validation
    let validation = chemins.work.join("validation_modifications.xlsx");
    if !validation.exists() {
        journal("[INFO] Aucun fichier de validation trouvé : aucune modification autorisée");
        return Ok(());
    }
    let decisions = excel::lire_decisions(&validation)?;
    let controle = regles::controler_decisions(&analyse, decisions, journal);
    let validees = controle.validees;

    journal(&format!("[OK] Modifications validées : {}", validees.len()));
    ecrire_json(&out.join("modifications_validees.json"), &validees)?;
    journal("[OK] modifications_validees.json généré");
    ecrire_json(&out.join("modifications_ignorees.json"), &controle.ignorees)?;
    journal("[OK] modifications_ignorees.json généré");

    excel::ecrire_controle(&out.join("controle_validation.xlsx"), &controle.rapport)?;
    journal(&format!(
        "[OK] Rapport de contrôle généré : {}",
        affiche("controle_validation.xlsx")
    ));

    // 4. Test en mémoire puis génération du SGX modifié
    let mut modeles = sgx::charger_modeles(&source, &validees)?;
    let conformes = regles::tester_renommages(&mut modeles, &validees, journal);
    if !conformes || validees.is_empty() {
        return Ok(());
    }

    let stem = source.file_stem().unwrap_or_default().to_string_lossy();
    let destination = out.join(format!("{stem}_modifie.sgx"));
    sgx::ecrire_sgx(&source, &destination, &modeles)?;
    journal(&format!(
        "[OK] SGX modifié généré : {}",
        destination.display()
    ));
    Ok(())
}

fn ecrire_json<T: Serialize>(chemin: &Path, valeur: &T) -> Res<()> {
    fs::write(chemin, serde_json::to_string_pretty(valeur)?)?;
    Ok(())
}

fn nom(chemin: &Path) -> String {
    chemin
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}
