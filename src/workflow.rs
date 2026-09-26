// Enchaînement historique : inventaire -> dry-run -> contrôle -> SGX modifié.
// Les dossiers sont passés explicitement ; aucun répertoire courant implicite.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::excel::{self, COLONNES_CORRESPONDANCE, COLONNES_DECISION};
use crate::{Contexte, Erreur, Journal, Resultat, regles, sgx};

// Dossiers d'un traitement : source, saisies humaines, résultats.
pub struct Chemins {
    // Contient l'unique fichier .sgx source.
    pub input: PathBuf,
    // Contient correspondance_swimlanes.xlsx et validation_modifications.xlsx.
    pub work: PathBuf,
    // Reçoit les JSON, Excel et SGX produits ; créé s'il est absent.
    pub output: PathBuf,
}

impl Chemins {
    // Disposition historique input/, work/, output/ sous `racine`.
    pub fn historiques(racine: &Path) -> Chemins {
        Chemins {
            input: racine.join("input"),
            work: racine.join("work"),
            output: racine.join("output"),
        }
    }
}

// Étape atteinte par une exécution (R12 : décision admise et SGX produit sont distincts).
#[derive(Serialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Statut {
    // Inventaire produit, aucune correspondance fournie.
    #[default]
    InventaireTermine,
    // Correspondances lues, aucun changement proposé.
    AnalyseSansImpact,
    // Analyse produite, décisions pas encore fournies.
    DecisionsAttendues,
    // Décisions contrôlées, aucune n'est admissible : aucun SGX.
    AucuneDecisionAdmissible,
    // Le recomptage en mémoire diverge : aucun SGX.
    ControleBloquant,
    // SGX vérifié et publié.
    SgxProduit,
}

// Bilan d'une exécution. Seuls les compteurs des étapes atteintes sont renseignés.
#[derive(Serialize, Debug, Default)]
pub struct Bilan {
    pub statut: Statut,
    pub modeles_reconnus: usize,
    pub occurrences_inventoriees: usize,
    pub noms_distincts: usize,
    pub correspondances_retenues: usize,
    // Anciens noms absents de l'inventaire : aucun impact (souvent une faute de frappe).
    pub noms_inconnus: Vec<String>,
    // Anciens noms saisis plusieurs fois : la dernière ligne est retenue.
    pub noms_en_doublon: Vec<String>,
    pub propositions: usize,
    pub occurrences_visees: u64,
    pub lignes_admises: usize,
    pub lignes_refusees: usize,
    pub lignes_ignorees: usize,
    pub lignes_en_attente: usize,
    pub propositions_sans_decision: usize,
    pub occurrences_admises: u64,
    pub divergences: Vec<String>,
    pub modeles_modifies: usize,
    pub occurrences_modifiees: u64,
    // Attributs ZIP qui n'ont pas pu être conservés sur les entrées réécrites.
    pub transformations: Vec<String>,
    pub sgx_produit: Option<PathBuf>,
}

// Exécute le workflow complet. Les messages de progression sont transmis à
// `journal` ; une erreur bloquante est retournée, sinon le bilan.
pub fn executer(chemins: &Chemins, journal: &mut Journal<'_>) -> Resultat<Bilan> {
    let dossier_sortie = &chemins.output;
    // Les messages affichent les chemins avec « / », comme la version historique.
    let chemin_affiche = |nom: &str| format!("{}/{nom}", dossier_sortie.display());
    let mut bilan = Bilan::default();

    let source = sgx::trouver_sgx(&chemins.input)?;
    journal(&format!(
        "[OK] Fichier SGX sélectionné : {}",
        nom_fichier(&source)
    ));

    // 1. Inventaire
    let (occurrences, modeles_reconnus) = sgx::extraire_lanes(&source, journal)?;
    fs::create_dir_all(dossier_sortie).contexte(
        "ecriture_impossible",
        &format!(
            "Création du dossier impossible : {}",
            dossier_sortie.display()
        ),
    )?;
    journal(&format!(
        "[OK] Extraction terminée : {} occurrences",
        occurrences.len()
    ));
    ecrire_json(&dossier_sortie.join("resultats.json"), &occurrences)?;
    journal("[OK] resultats.json généré");

    let synthese = regles::synthetiser(&occurrences);
    journal(&format!(
        "[OK] Synthèse terminée : {} lanes uniques",
        synthese.len()
    ));
    ecrire_json(&dossier_sortie.join("synthese.json"), &synthese)?;
    journal("[OK] synthese.json généré");

    excel::ecrire_inventaire(&dossier_sortie.join("inventaire_swimlanes.xlsx"), &synthese)?;
    journal(&format!(
        "[OK] Excel généré : {}",
        chemin_affiche("inventaire_swimlanes.xlsx")
    ));
    bilan.modeles_reconnus = modeles_reconnus;
    bilan.occurrences_inventoriees = occurrences.len();
    bilan.noms_distincts = synthese.len();

    // 2. Dry-run
    let correspondance = chemins.work.join("correspondance_swimlanes.xlsx");
    if !correspondance.exists() {
        journal(
            "[INFO] Aucun fichier de correspondance trouvé : la préparation des modifications est ignorée",
        );
        return Ok(bilan);
    }
    let lignes = excel::lire_feuille(&correspondance, "Correspondance", &COLONNES_CORRESPONDANCE)?;
    let correspondances = regles::lire_correspondances(&lignes).map_err(|diagnostics| {
        Erreur::nouvelle(
            "classeur_cellules",
            "Le classeur de correspondance contient des cellules à corriger : il n'a pas été pris en compte.",
        )
        .avec_details(diagnostics)
    })?;
    journal(&format!(
        "[OK] Correspondances chargées : {}",
        correspondances.retenues.len()
    ));
    for nom in &correspondances.doublons {
        journal(&format!(
            "[INFO] Nom actuel renseigné plusieurs fois, dernière ligne retenue : « {nom} »"
        ));
    }
    bilan.correspondances_retenues = correspondances.retenues.len();
    bilan.noms_en_doublon = correspondances.doublons.clone();
    bilan.noms_inconnus = correspondances
        .retenues
        .keys()
        .filter(|nom| !synthese.contains_key(*nom))
        .cloned()
        .collect();
    for nom in &bilan.noms_inconnus {
        journal(&format!(
            "[INFO] Nom actuel absent de l'inventaire, aucun impact : « {nom} »"
        ));
    }
    ecrire_json(
        &dossier_sortie.join("correspondances.json"),
        &correspondances.retenues,
    )?;
    journal("[OK] correspondances.json généré");

    let analyse = regles::dry_run(&correspondances.retenues, &synthese);
    journal(&format!(
        "[OK] Dry-run préparé : {} impacts flux/lane",
        analyse.len()
    ));
    ecrire_json(&dossier_sortie.join("analyse_modifications.json"), &analyse)?;
    journal("[OK] analyse_modifications.json généré");

    excel::ecrire_analyse(&dossier_sortie.join("analyse_modifications.xlsx"), &analyse)?;
    journal(&format!(
        "[OK] Excel d'analyse généré : {}",
        chemin_affiche("analyse_modifications.xlsx")
    ));
    bilan.propositions = analyse.len();
    bilan.occurrences_visees = analyse
        .iter()
        .map(|modification| modification.occurrences)
        .sum();
    if analyse.is_empty() {
        bilan.statut = Statut::AnalyseSansImpact;
        return Ok(bilan);
    }

    // 3. Validation
    let validation = chemins.work.join("validation_modifications.xlsx");
    if !validation.exists() {
        journal("[INFO] Aucun fichier de validation trouvé : aucune modification autorisée");
        bilan.statut = Statut::DecisionsAttendues;
        return Ok(bilan);
    }
    let lignes = excel::lire_feuille(&validation, "Analyse", &COLONNES_DECISION)?;
    let controle = regles::controler_decisions(&analyse, &lignes, journal);
    let validees = &controle.validees;

    journal(&format!("[OK] Modifications validées : {}", validees.len()));
    ecrire_json(
        &dossier_sortie.join("modifications_validees.json"),
        validees,
    )?;
    journal("[OK] modifications_validees.json généré");
    ecrire_json(
        &dossier_sortie.join("modifications_ignorees.json"),
        &controle.ignorees,
    )?;
    journal("[OK] modifications_ignorees.json généré");

    excel::ecrire_controle(
        &dossier_sortie.join("controle_validation.xlsx"),
        &controle.rapport,
    )?;
    journal(&format!(
        "[OK] Rapport de contrôle généré : {}",
        chemin_affiche("controle_validation.xlsx")
    ));
    bilan.lignes_admises = validees.len();
    bilan.lignes_refusees = controle.refusees;
    bilan.lignes_ignorees = controle.ignorees.len();
    bilan.lignes_en_attente = controle.en_attente;
    bilan.propositions_sans_decision = controle.propositions_sans_decision;
    bilan.occurrences_admises = validees
        .iter()
        .map(|modification| modification.occurrences)
        .sum();
    if validees.is_empty() {
        bilan.statut = Statut::AucuneDecisionAdmissible;
        return Ok(bilan);
    }

    // 4. Test en mémoire, puis écriture, vérification et publication du SGX
    let mut modeles = sgx::charger_modeles(&source, validees)?;
    bilan.divergences = regles::tester_renommages(&mut modeles, validees, journal);
    if !bilan.divergences.is_empty() {
        bilan.statut = Statut::ControleBloquant;
        return Ok(bilan);
    }

    let nom_sans_extension = source.file_stem().unwrap_or_default().to_string_lossy();
    let destination = chemin_disponible(dossier_sortie, &format!("{nom_sans_extension}_modifie"));
    bilan.transformations = sgx::produire_sgx(&source, &destination, &modeles, validees)?;
    for transformation in &bilan.transformations {
        journal(&format!(
            "[INFO] Attribut ZIP non conservé : {transformation}"
        ));
    }
    journal(&format!(
        "[OK] SGX modifié généré : {}",
        destination.display()
    ));
    bilan.modeles_modifies = modeles.len();
    bilan.occurrences_modifiees = bilan.occurrences_admises;
    bilan.sgx_produit = Some(destination);
    bilan.statut = Statut::SgxProduit;
    Ok(bilan)
}

// Premier nom libre : <base>.sgx, puis <base>_2.sgx, <base>_3.sgx...
// Une sortie existante n'est jamais écrasée (P04).
fn chemin_disponible(dossier: &Path, base: &str) -> PathBuf {
    let mut chemin = dossier.join(format!("{base}.sgx"));
    let mut numero = 2;
    while chemin.exists() {
        chemin = dossier.join(format!("{base}_{numero}.sgx"));
        numero += 1;
    }
    chemin
}

fn ecrire_json<T: Serialize + ?Sized>(chemin: &Path, valeur: &T) -> Resultat<()> {
    fs::write(chemin, serde_json::to_string_pretty(valeur)?).contexte(
        "ecriture_impossible",
        &format!("Écriture impossible : {}", chemin.display()),
    )
}

fn nom_fichier(chemin: &Path) -> String {
    chemin
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}
