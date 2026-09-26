// Étapes du traitement, communes au CLI historique et au dossier de traitement :
// inventaire -> correspondances -> dry-run -> contrôle -> SGX vérifié.
// Les dossiers sont passés explicitement ; aucun répertoire courant implicite.

use std::fs;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::excel::{self, COLONNES_CORRESPONDANCE, COLONNES_DECISION};
use crate::regles::{self, Controle, Correspondances, Modification, Occurrence, Synthese};
use crate::{Contexte, Erreur, Journal, Resultat, sgx};

// Dossiers du mode historique : source, saisies humaines, résultats.
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

// Étape atteinte par une opération (R12 : décision admise et SGX produit sont distincts).
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
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
    // Décisions contrôlées et conformes : la génération est possible.
    ProductionPossible,
    // SGX vérifié et publié.
    SgxProduit,
}

// Bilan d'une opération. Seuls les compteurs des étapes atteintes sont renseignés.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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

pub(crate) struct Inventaire {
    pub(crate) occurrences: Vec<Occurrence>,
    pub(crate) synthese: IndexMap<String, Synthese>,
    pub(crate) modeles_reconnus: usize,
}

impl Bilan {
    pub(crate) fn renseigner_inventaire(&mut self, inventaire: &Inventaire) {
        self.modeles_reconnus = inventaire.modeles_reconnus;
        self.occurrences_inventoriees = inventaire.occurrences.len();
        self.noms_distincts = inventaire.synthese.len();
    }

    pub(crate) fn renseigner_correspondances(
        &mut self,
        correspondances: &Correspondances,
        inventaire: &Inventaire,
    ) {
        self.correspondances_retenues = correspondances.retenues.len();
        self.noms_en_doublon = correspondances.doublons.clone();
        self.noms_inconnus = noms_inconnus(correspondances, inventaire);
    }

    pub(crate) fn renseigner_analyse(&mut self, analyse: &[Modification]) {
        self.propositions = analyse.len();
        self.occurrences_visees = analyse
            .iter()
            .map(|modification| modification.occurrences)
            .sum();
        self.statut = if analyse.is_empty() {
            Statut::AnalyseSansImpact
        } else {
            Statut::DecisionsAttendues
        };
    }

    pub(crate) fn renseigner_controle(&mut self, controle: &Controle, divergences: Vec<String>) {
        self.lignes_admises = controle.validees.len();
        self.lignes_refusees = controle.refusees;
        self.lignes_ignorees = controle.ignorees.len();
        self.lignes_en_attente = controle.en_attente;
        self.propositions_sans_decision = controle.propositions_sans_decision;
        self.occurrences_admises = controle
            .validees
            .iter()
            .map(|modification| modification.occurrences)
            .sum();
        self.divergences = divergences;
        self.statut = if controle.validees.is_empty() {
            Statut::AucuneDecisionAdmissible
        } else if !self.divergences.is_empty() {
            Statut::ControleBloquant
        } else {
            Statut::ProductionPossible
        };
    }
}

fn noms_inconnus(correspondances: &Correspondances, inventaire: &Inventaire) -> Vec<String> {
    correspondances
        .retenues
        .keys()
        .filter(|nom| !inventaire.synthese.contains_key(*nom))
        .cloned()
        .collect()
}

// ---------------------------------------------------------------- Étapes

pub(crate) fn inventorier(source: &Path, journal: &mut Journal<'_>) -> Resultat<Inventaire> {
    let (occurrences, modeles_reconnus) = sgx::extraire_lanes(source, journal)?;
    journal(&format!(
        "[OK] Extraction terminée : {} occurrences",
        occurrences.len()
    ));
    let synthese = regles::synthetiser(&occurrences);
    Ok(Inventaire {
        occurrences,
        synthese,
        modeles_reconnus,
    })
}

pub(crate) fn ecrire_inventaire(
    dossier: &Path,
    inventaire: &Inventaire,
    journal: &mut Journal<'_>,
) -> Resultat<()> {
    ecrire_json(&dossier.join("resultats.json"), &inventaire.occurrences)?;
    journal("[OK] resultats.json généré");
    journal(&format!(
        "[OK] Synthèse terminée : {} lanes uniques",
        inventaire.synthese.len()
    ));
    ecrire_json(&dossier.join("synthese.json"), &inventaire.synthese)?;
    journal("[OK] synthese.json généré");
    excel::ecrire_inventaire(
        &dossier.join("inventaire_swimlanes.xlsx"),
        &inventaire.synthese,
    )?;
    journal(&format!(
        "[OK] Excel généré : {}",
        chemin_affiche(dossier, "inventaire_swimlanes.xlsx")
    ));
    Ok(())
}

// Lit et contrôle un classeur de correspondance (P02). Les cellules invalides
// refusent le classeur entier.
pub(crate) fn lire_correspondances(
    fichier: &Path,
    inventaire: &Inventaire,
    journal: &mut Journal<'_>,
) -> Resultat<Correspondances> {
    let lignes = excel::lire_feuille(fichier, "Correspondance", &COLONNES_CORRESPONDANCE)?;
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
    for nom in noms_inconnus(&correspondances, inventaire) {
        journal(&format!(
            "[INFO] Nom actuel absent de l'inventaire, aucun impact : « {nom} »"
        ));
    }
    Ok(correspondances)
}

pub(crate) fn preparer_analyse(
    dossier: &Path,
    correspondances: &Correspondances,
    inventaire: &Inventaire,
    journal: &mut Journal<'_>,
) -> Resultat<Vec<Modification>> {
    ecrire_json(
        &dossier.join("correspondances.json"),
        &correspondances.retenues,
    )?;
    journal("[OK] correspondances.json généré");
    let analyse = regles::dry_run(&correspondances.retenues, &inventaire.synthese);
    journal(&format!(
        "[OK] Dry-run préparé : {} impacts flux/lane",
        analyse.len()
    ));
    ecrire_json(&dossier.join("analyse_modifications.json"), &analyse)?;
    journal("[OK] analyse_modifications.json généré");
    excel::ecrire_analyse(&dossier.join("analyse_modifications.xlsx"), &analyse)?;
    journal(&format!(
        "[OK] Excel d'analyse généré : {}",
        chemin_affiche(dossier, "analyse_modifications.xlsx")
    ));
    Ok(analyse)
}

// Contrôle les décisions et écrit les rapports dans `dossier`.
pub(crate) fn controler(
    fichier_decisions: &Path,
    analyse: &[Modification],
    dossier: &Path,
    journal: &mut Journal<'_>,
) -> Resultat<Controle> {
    let lignes = excel::lire_feuille(fichier_decisions, "Analyse", &COLONNES_DECISION)?;
    let controle = regles::controler_decisions(analyse, &lignes, journal);
    journal(&format!(
        "[OK] Modifications validées : {}",
        controle.validees.len()
    ));
    ecrire_json(
        &dossier.join("modifications_validees.json"),
        &controle.validees,
    )?;
    journal("[OK] modifications_validees.json généré");
    ecrire_json(
        &dossier.join("modifications_ignorees.json"),
        &controle.ignorees,
    )?;
    journal("[OK] modifications_ignorees.json généré");
    excel::ecrire_controle(&dossier.join("controle_validation.xlsx"), &controle.rapport)?;
    journal(&format!(
        "[OK] Rapport de contrôle généré : {}",
        chemin_affiche(dossier, "controle_validation.xlsx")
    ));
    Ok(controle)
}

// Applique en mémoire les modifications validées et recompte chaque opération.
// Retourne les modèles modifiés et les divergences (vide si conforme).
pub(crate) fn tester_renommages(
    source: &Path,
    validees: &[Modification],
    journal: &mut Journal<'_>,
) -> Resultat<(IndexMap<String, Value>, Vec<String>)> {
    let mut modeles = sgx::charger_modeles(source, validees)?;
    let divergences = regles::tester_renommages(&mut modeles, validees, journal);
    Ok((modeles, divergences))
}

// Écrit, vérifie et publie le SGX modifié, puis complète le bilan.
pub(crate) fn publier(
    source: &Path,
    modeles: &IndexMap<String, Value>,
    validees: &[Modification],
    destination: &Path,
    bilan: &mut Bilan,
    journal: &mut Journal<'_>,
) -> Resultat<()> {
    bilan.transformations = sgx::produire_sgx(source, destination, modeles, validees)?;
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
    bilan.sgx_produit = Some(destination.to_path_buf());
    bilan.statut = Statut::SgxProduit;
    Ok(())
}

// ---------------------------------------------------------------- Mode historique

// Exécute le workflow complet sur input/, work/, output/. Les messages de
// progression sont transmis à `journal` ; une erreur bloquante est retournée,
// sinon le bilan.
pub fn executer(chemins: &Chemins, journal: &mut Journal<'_>) -> Resultat<Bilan> {
    let dossier_sortie = &chemins.output;
    let mut bilan = Bilan::default();

    let source = sgx::trouver_sgx(&chemins.input)?;
    journal(&format!(
        "[OK] Fichier SGX sélectionné : {}",
        nom_fichier(&source)
    ));

    // 1. Inventaire
    let inventaire = inventorier(&source, journal)?;
    creer_dossier(dossier_sortie)?;
    ecrire_inventaire(dossier_sortie, &inventaire, journal)?;
    bilan.renseigner_inventaire(&inventaire);

    // 2. Dry-run
    let fichier_correspondances = chemins.work.join("correspondance_swimlanes.xlsx");
    if !fichier_correspondances.exists() {
        journal(
            "[INFO] Aucun fichier de correspondance trouvé : la préparation des modifications est ignorée",
        );
        return Ok(bilan);
    }
    let correspondances = lire_correspondances(&fichier_correspondances, &inventaire, journal)?;
    bilan.renseigner_correspondances(&correspondances, &inventaire);
    let analyse = preparer_analyse(dossier_sortie, &correspondances, &inventaire, journal)?;
    bilan.renseigner_analyse(&analyse);
    if analyse.is_empty() {
        return Ok(bilan);
    }

    // 3. Validation
    let fichier_decisions = chemins.work.join("validation_modifications.xlsx");
    if !fichier_decisions.exists() {
        journal("[INFO] Aucun fichier de validation trouvé : aucune modification autorisée");
        return Ok(bilan);
    }
    let controle = controler(&fichier_decisions, &analyse, dossier_sortie, journal)?;
    let (modeles, divergences) = if controle.validees.is_empty() {
        (IndexMap::new(), Vec::new())
    } else {
        tester_renommages(&source, &controle.validees, journal)?
    };
    bilan.renseigner_controle(&controle, divergences);
    if bilan.statut != Statut::ProductionPossible {
        return Ok(bilan);
    }

    // 4. Écriture, vérification et publication du SGX
    let nom_sans_extension = source.file_stem().unwrap_or_default().to_string_lossy();
    let destination = chemin_disponible(dossier_sortie, &format!("{nom_sans_extension}_modifie"));
    publier(
        &source,
        &modeles,
        &controle.validees,
        &destination,
        &mut bilan,
        journal,
    )?;
    Ok(bilan)
}

// ---------------------------------------------------------------- Utilitaires

// Premier nom libre : <base>.sgx, puis <base>_2.sgx, <base>_3.sgx...
// Une sortie existante n'est jamais écrasée (P04).
pub(crate) fn chemin_disponible(dossier: &Path, base: &str) -> PathBuf {
    let mut chemin = dossier.join(format!("{base}.sgx"));
    let mut numero = 2;
    while chemin.exists() {
        chemin = dossier.join(format!("{base}_{numero}.sgx"));
        numero += 1;
    }
    chemin
}

pub(crate) fn creer_dossier(dossier: &Path) -> Resultat<()> {
    fs::create_dir_all(dossier).contexte(
        "ecriture_impossible",
        &format!("Création du dossier impossible : {}", dossier.display()),
    )
}

pub(crate) fn ecrire_json<T: Serialize + ?Sized>(chemin: &Path, valeur: &T) -> Resultat<()> {
    fs::write(chemin, serde_json::to_string_pretty(valeur)?).contexte(
        "ecriture_impossible",
        &format!("Écriture impossible : {}", chemin.display()),
    )
}

// Les messages affichent les chemins avec « / », comme la version historique.
fn chemin_affiche(dossier: &Path, nom: &str) -> String {
    format!("{}/{nom}", dossier.display())
}

pub(crate) fn nom_fichier(chemin: &Path) -> String {
    chemin
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}
