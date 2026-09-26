// Dossier de traitement (P01) : un SGX, ses classeurs et ses résultats, dans un
// dossier autonome que l'on peut fermer et rouvrir.
//
//   traitement.json        manifeste versionné (état, révision, empreintes)
//   traitement.verrou      verrou système : un seul processus à la fois
//   source/<nom>.sgx       copie de la source, contrôlée par empreinte
//   inventaire/            inventaire produit à la création
//   edition/               classeurs ouverts et modifiés dans Excel
//   edition/precedents/    classeurs d'édition modifiés puis remplacés
//   entrees/               instantanés adoptés, jamais modifiés
//   analyse/               analyse des correspondances adoptées
//   controle/              dernier contrôle des décisions adoptées
//   sorties/tentative-NNN/ rapports et SGX d'une tentative de production
//
// Chaque opération qui modifie le traitement reçoit la révision attendue et la
// refuse si elle est obsolète. Les fichiers fournis par l'utilisateur ne sont
// jamais modifiés ni déplacés.

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::workflow::{self, Bilan, Inventaire, Statut};
use crate::{Contexte, Erreur, Journal, Resultat};

const VERSION_SCHEMA: u32 = 1;
const MANIFESTE: &str = "traitement.json";
const VERROU: &str = "traitement.verrou";
const DOSSIER_INVENTAIRE: &str = "inventaire";
const DOSSIER_ANALYSE: &str = "analyse";
const DOSSIER_CONTROLE: &str = "controle";
const EDITION_CORRESPONDANCES: &str = "edition/correspondance_swimlanes.xlsx";
const EDITION_DECISIONS: &str = "edition/validation_modifications.xlsx";

// ---------------------------------------------------------------- Manifeste

// Fichier interne du traitement : chemin relatif (séparateur « / ») et empreinte SHA-256.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct FichierSuivi {
    chemin: String,
    empreinte: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Source {
    nom_original: String,
    // Informatif : la copie contrôlée est la référence (A11).
    chemin_original: String,
    copie: FichierSuivi,
}

// Classeur adopté : instantané immuable et nom du fichier lu.
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Adoption {
    instantane: FichierSuivi,
    fichier_lu: String,
    horodatage: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Tentative {
    numero: u32,
    dossier: String,
    // Révision du traitement après enregistrement de la tentative.
    revision: u64,
    horodatage: u64,
    statut: Statut,
    sgx: Option<FichierSuivi>,
    modeles_modifies: usize,
    occurrences_modifiees: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Manifeste {
    version_schema: u32,
    nom: String,
    revision: u64,
    cree_le: u64,
    source: Source,
    modeles_reconnus: usize,
    occurrences_inventoriees: usize,
    noms_distincts: usize,
    // Empreinte de référence de chaque classeur d'édition : version générée ou
    // dernière version adoptée. Toute différence signale un classeur à relire.
    reference_edition_correspondances: String,
    reference_edition_decisions: Option<String>,
    correspondances: Option<Adoption>,
    analyse_preparee: bool,
    decisions: Option<Adoption>,
    tentatives: Vec<Tentative>,
}

// ---------------------------------------------------------------- État publié

// Étape du parcours (ui-spec.md, « États et transitions »).
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Etape {
    InventairePret,
    CorrespondancesPretes,
    AnalysePrete,
    DecisionsControlees,
    ResultatProduit,
}

// État d'un SGX produit lors d'une tentative, vérifié à chaque lecture de l'état.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EtatFichier {
    Disponible,
    Absent,
    Modifie,
}

#[derive(Serialize, Debug, Clone)]
pub struct EtatTentative {
    pub numero: u32,
    pub horodatage: u64,
    pub statut: Statut,
    pub dossier: PathBuf,
    pub sgx: Option<PathBuf>,
    pub etat_sgx: Option<EtatFichier>,
    pub modeles_modifies: usize,
    pub occurrences_modifiees: u64,
    // Vrai si la tentative porte sur les entrées actuellement adoptées.
    pub courante: bool,
}

// Vue complète du traitement pour l'interface.
#[derive(Serialize, Debug, Clone)]
pub struct Etat {
    pub dossier: PathBuf,
    pub nom: String,
    pub revision: u64,
    pub etape: Etape,
    pub source_nom: String,
    pub source_chemin_original: String,
    pub modeles_reconnus: usize,
    pub occurrences_inventoriees: usize,
    pub noms_distincts: usize,
    pub edition_correspondances: PathBuf,
    // Classeur modifié depuis sa génération ou sa dernière lecture : à relire.
    pub correspondances_a_relire: bool,
    pub correspondances_adoptees: bool,
    pub analyse_preparee: bool,
    pub fichier_analyse: Option<PathBuf>,
    pub edition_decisions: Option<PathBuf>,
    pub decisions_a_relire: bool,
    pub decisions_adoptees: bool,
    pub fichier_controle: Option<PathBuf>,
    pub tentatives: Vec<EtatTentative>,
    // Dossiers de sortie sans enregistrement dans le manifeste (arrêt pendant une production).
    pub tentatives_interrompues: Vec<PathBuf>,
}

// ---------------------------------------------------------------- Traitement

pub struct Traitement {
    dossier: PathBuf,
    manifeste: Manifeste,
    // Verrou système tenu tant que le traitement est ouvert.
    _verrou: File,
}

impl Traitement {
    // Crée un traitement pour `source` dans un nouveau sous-dossier de
    // `dossier_parent`, copie la source et produit l'inventaire. En cas
    // d'échec, le sous-dossier créé est supprimé.
    pub fn creer(
        source: &Path,
        dossier_parent: &Path,
        journal: &mut Journal<'_>,
    ) -> Resultat<(Traitement, Bilan)> {
        let nom_original = workflow::nom_fichier(source);
        let nom_sans_extension = source
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        let dossier = creer_sous_dossier(
            dossier_parent,
            &format!("{nom_sans_extension} - traitement"),
        )?;
        let resultat = Traitement::initialiser(
            &dossier,
            source,
            &nom_original,
            &nom_sans_extension,
            journal,
        );
        if resultat.is_err() {
            let _ = fs::remove_dir_all(&dossier);
        }
        resultat
    }

    fn initialiser(
        dossier: &Path,
        source: &Path,
        nom_original: &str,
        nom_sans_extension: &str,
        journal: &mut Journal<'_>,
    ) -> Resultat<(Traitement, Bilan)> {
        let verrou = verrouiller(dossier)?;
        let contenu = fs::read(source).contexte(
            "source_illisible",
            &format!("Impossible de lire le SGX : {}", source.display()),
        )?;
        let chemin_copie = format!("source/{nom_original}");
        let copie = ecrire_fichier_interne(dossier, &chemin_copie, &contenu)?;
        journal(&format!("[OK] Copie de la source : {chemin_copie}"));

        let inventaire = workflow::inventorier(&dossier.join(&chemin_copie), journal)?;
        let dossier_inventaire = dossier.join(DOSSIER_INVENTAIRE);
        workflow::creer_dossier(&dossier_inventaire)?;
        workflow::ecrire_inventaire(&dossier_inventaire, &inventaire, journal)?;
        workflow::creer_dossier(&dossier.join("edition"))?;
        let edition = fs::read(dossier_inventaire.join("inventaire_swimlanes.xlsx"))?;
        let reference = ecrire_fichier_interne(dossier, EDITION_CORRESPONDANCES, &edition)?;

        let mut bilan = Bilan::default();
        bilan.renseigner_inventaire(&inventaire);
        let manifeste = Manifeste {
            version_schema: VERSION_SCHEMA,
            nom: nom_sans_extension.to_string(),
            revision: 1,
            cree_le: maintenant(),
            source: Source {
                nom_original: nom_original.to_string(),
                chemin_original: source.display().to_string(),
                copie,
            },
            modeles_reconnus: inventaire.modeles_reconnus,
            occurrences_inventoriees: inventaire.occurrences.len(),
            noms_distincts: inventaire.synthese.len(),
            reference_edition_correspondances: reference.empreinte,
            reference_edition_decisions: None,
            correspondances: None,
            analyse_preparee: false,
            decisions: None,
            tentatives: Vec::new(),
        };
        enregistrer_manifeste(dossier, &manifeste)?;
        Ok((
            Traitement {
                dossier: dossier.to_path_buf(),
                manifeste,
                _verrou: verrou,
            },
            bilan,
        ))
    }

    // Rouvre un traitement existant. Refuse un manifeste inconnu ou illisible,
    // et toute altération de la copie source ou d'un instantané adopté (A11).
    pub fn ouvrir(dossier: &Path) -> Resultat<Traitement> {
        let chemin_manifeste = dossier.join(MANIFESTE);
        let illisible = format!(
            "Ce traitement ne peut pas être repris : {} est absent ou illisible.",
            chemin_manifeste.display()
        );
        let texte =
            fs::read_to_string(&chemin_manifeste).contexte("traitement_illisible", &illisible)?;
        let brut: serde_json::Value =
            serde_json::from_str(&texte).contexte("traitement_illisible", &illisible)?;
        let version = brut
            .get("version_schema")
            .and_then(serde_json::Value::as_u64);
        if version != Some(VERSION_SCHEMA as u64) {
            return Err(Erreur::nouvelle(
                "traitement_version_inconnue",
                format!(
                    "Ce traitement a été créé par une version de l'application non prise en charge (schéma {}).",
                    version.map_or("absent".to_string(), |version| version.to_string())
                ),
            ));
        }
        let manifeste: Manifeste =
            serde_json::from_str(&texte).contexte("traitement_illisible", &illisible)?;
        let verrou = verrouiller(dossier)?;
        let traitement = Traitement {
            dossier: dossier.to_path_buf(),
            manifeste,
            _verrou: verrou,
        };
        traitement.verifier_integrite()?;
        Ok(traitement)
    }

    pub fn dossier(&self) -> &Path {
        &self.dossier
    }

    pub fn revision(&self) -> u64 {
        self.manifeste.revision
    }

    pub fn etat(&self) -> Resultat<Etat> {
        let manifeste = &self.manifeste;
        let chemin_decisions = self.dossier.join(EDITION_DECISIONS);
        let derniere_production = manifeste.tentatives.last().filter(|tentative| {
            tentative.statut == Statut::SgxProduit && tentative.revision == manifeste.revision
        });
        let etape = if derniere_production.is_some() {
            Etape::ResultatProduit
        } else if manifeste.decisions.is_some() {
            Etape::DecisionsControlees
        } else if manifeste.analyse_preparee {
            Etape::AnalysePrete
        } else if manifeste.correspondances.is_some() {
            Etape::CorrespondancesPretes
        } else {
            Etape::InventairePret
        };

        let mut tentatives = Vec::new();
        for tentative in &manifeste.tentatives {
            let (sgx, etat_sgx) = match &tentative.sgx {
                Some(fichier) => {
                    let chemin = self.chemin_interne(&fichier.chemin)?;
                    let etat = match empreinte_fichier(&chemin) {
                        Ok(empreinte) if empreinte == fichier.empreinte => EtatFichier::Disponible,
                        Ok(_) => EtatFichier::Modifie,
                        Err(_) => EtatFichier::Absent,
                    };
                    (Some(chemin), Some(etat))
                }
                None => (None, None),
            };
            tentatives.push(EtatTentative {
                numero: tentative.numero,
                horodatage: tentative.horodatage,
                statut: tentative.statut,
                dossier: self.chemin_interne(&tentative.dossier)?,
                sgx,
                etat_sgx,
                modeles_modifies: tentative.modeles_modifies,
                occurrences_modifiees: tentative.occurrences_modifiees,
                courante: tentative.revision == manifeste.revision,
            });
        }

        Ok(Etat {
            dossier: self.dossier.clone(),
            nom: manifeste.nom.clone(),
            revision: manifeste.revision,
            etape,
            source_nom: manifeste.source.nom_original.clone(),
            source_chemin_original: manifeste.source.chemin_original.clone(),
            modeles_reconnus: manifeste.modeles_reconnus,
            occurrences_inventoriees: manifeste.occurrences_inventoriees,
            noms_distincts: manifeste.noms_distincts,
            edition_correspondances: self.dossier.join(EDITION_CORRESPONDANCES),
            correspondances_a_relire: self.edition_modifiee(
                EDITION_CORRESPONDANCES,
                Some(&manifeste.reference_edition_correspondances),
            ),
            correspondances_adoptees: manifeste.correspondances.is_some(),
            analyse_preparee: manifeste.analyse_preparee,
            fichier_analyse: manifeste.analyse_preparee.then(|| {
                self.dossier
                    .join(DOSSIER_ANALYSE)
                    .join("analyse_modifications.xlsx")
            }),
            edition_decisions: chemin_decisions.exists().then_some(chemin_decisions),
            decisions_a_relire: self.edition_modifiee(
                EDITION_DECISIONS,
                manifeste.reference_edition_decisions.as_ref(),
            ),
            decisions_adoptees: manifeste.decisions.is_some(),
            fichier_controle: manifeste.decisions.is_some().then(|| {
                self.dossier
                    .join(DOSSIER_CONTROLE)
                    .join("controle_validation.xlsx")
            }),
            tentatives,
            tentatives_interrompues: self.tentatives_interrompues(),
        })
    }

    // Lit un classeur de correspondance (par défaut le classeur d'édition) et
    // l'adopte s'il est valide (P02). Sinon, l'adoption précédente est conservée.
    // L'analyse et les décisions deviennent obsolètes.
    pub fn adopter_correspondances(
        &mut self,
        revision_attendue: u64,
        fichier: Option<&Path>,
        journal: &mut Journal<'_>,
    ) -> Resultat<Bilan> {
        self.verifier_revision(revision_attendue)?;
        self.verifier_integrite()?;
        let (contenu, fichier_lu) = self.lire_classeur_fourni(fichier, EDITION_CORRESPONDANCES)?;

        let inventaire = self.inventorier(journal)?;
        let lecture = self.fichier_temporaire("lecture-en-cours.xlsx", &contenu)?;
        let correspondances = workflow::lire_correspondances(&lecture, &inventaire, journal);
        let _ = fs::remove_file(&lecture);
        let correspondances = correspondances?;

        let mut bilan = Bilan::default();
        bilan.renseigner_inventaire(&inventaire);
        bilan.renseigner_correspondances(&correspondances, &inventaire);

        let empreinte = empreinte(&contenu);
        let deja_adopte = self
            .manifeste
            .correspondances
            .as_ref()
            .is_some_and(|adoption| adoption.instantane.empreinte == empreinte);
        if deja_adopte {
            journal(
                "[INFO] Correspondances identiques à celles déjà adoptées : aucune modification",
            );
            self.manifeste.reference_edition_correspondances = self.remplacer_edition(
                EDITION_CORRESPONDANCES,
                &contenu,
                Some(&self.manifeste.reference_edition_correspondances.clone()),
            )?;
            enregistrer_manifeste(&self.dossier, &self.manifeste)?;
            return Ok(bilan);
        }

        let revision = self.manifeste.revision + 1;
        let instantane = ecrire_fichier_interne(
            &self.dossier,
            &format!("entrees/correspondances-r{revision:03}.xlsx"),
            &contenu,
        )?;
        let reference = self.remplacer_edition(
            EDITION_CORRESPONDANCES,
            &contenu,
            Some(&self.manifeste.reference_edition_correspondances.clone()),
        )?;
        let mut manifeste = self.manifeste.clone();
        manifeste.revision = revision;
        manifeste.reference_edition_correspondances = reference;
        manifeste.correspondances = Some(Adoption {
            instantane,
            fichier_lu,
            horodatage: maintenant(),
        });
        manifeste.analyse_preparee = false;
        manifeste.decisions = None;
        self.enregistrer(manifeste)?;
        journal("[OK] Correspondances adoptées : analyse et décisions précédentes à refaire");
        Ok(bilan)
    }

    // Prépare l'analyse à partir des correspondances adoptées et crée le
    // classeur de décision (copie de l'analyse avec colonne Validation).
    pub fn preparer_analyse(
        &mut self,
        revision_attendue: u64,
        journal: &mut Journal<'_>,
    ) -> Resultat<Bilan> {
        self.verifier_revision(revision_attendue)?;
        self.verifier_integrite()?;
        let (inventaire, correspondances) = self.charger_correspondances(journal)?;
        let dossier_analyse = self.dossier.join(DOSSIER_ANALYSE);
        workflow::creer_dossier(&dossier_analyse)?;
        let analyse =
            workflow::preparer_analyse(&dossier_analyse, &correspondances, &inventaire, journal)?;

        let mut bilan = Bilan::default();
        bilan.renseigner_inventaire(&inventaire);
        bilan.renseigner_correspondances(&correspondances, &inventaire);
        bilan.renseigner_analyse(&analyse);

        let mut manifeste = self.manifeste.clone();
        if !self.manifeste.analyse_preparee {
            manifeste.revision += 1;
            manifeste.analyse_preparee = true;
            manifeste.decisions = None;
            if analyse.is_empty() {
                // Aucun changement proposé : un ancien classeur de décision n'a plus d'objet.
                self.archiver_edition(EDITION_DECISIONS)?;
                manifeste.reference_edition_decisions = None;
            } else {
                let classeur = fs::read(dossier_analyse.join("analyse_modifications.xlsx"))?;
                manifeste.reference_edition_decisions = Some(self.remplacer_edition(
                    EDITION_DECISIONS,
                    &classeur,
                    self.manifeste.reference_edition_decisions.clone().as_ref(),
                )?);
            }
        }
        self.enregistrer(manifeste)?;
        Ok(bilan)
    }

    // Lit un classeur de décision (par défaut le classeur d'édition), l'adopte
    // et contrôle les décisions contre l'analyse recalculée (R07, R08), y
    // compris le recomptage en mémoire (R09, R10).
    pub fn adopter_decisions(
        &mut self,
        revision_attendue: u64,
        fichier: Option<&Path>,
        journal: &mut Journal<'_>,
    ) -> Resultat<Bilan> {
        self.verifier_revision(revision_attendue)?;
        self.verifier_integrite()?;
        if !self.manifeste.analyse_preparee {
            return Err(Erreur::nouvelle(
                "etape_prealable",
                "Préparez l'analyse avant d'importer des décisions.",
            ));
        }
        let (inventaire, correspondances) = self.charger_correspondances(journal)?;
        let analyse = crate::regles::dry_run(&correspondances.retenues, &inventaire.synthese);
        if analyse.is_empty() {
            return Err(Erreur::nouvelle(
                "etape_prealable",
                "Aucun changement proposé : il n'y a aucune décision à importer.",
            ));
        }
        let (contenu, fichier_lu) = self.lire_classeur_fourni(fichier, EDITION_DECISIONS)?;

        // Contrôle sur une copie : un classeur illisible ou au mauvais format
        // n'est pas adopté et l'adoption précédente est conservée.
        let lecture = self.fichier_temporaire("lecture-en-cours.xlsx", &contenu)?;
        let dossier_controle = self.dossier.join(DOSSIER_CONTROLE);
        workflow::creer_dossier(&dossier_controle)?;
        let controle = workflow::controler(&lecture, &analyse, &dossier_controle, journal);
        let _ = fs::remove_file(&lecture);
        let controle = controle?;

        let divergences = if controle.validees.is_empty() {
            Vec::new()
        } else {
            workflow::tester_renommages(&self.chemin_source()?, &controle.validees, journal)?.1
        };
        let mut bilan = Bilan::default();
        bilan.renseigner_inventaire(&inventaire);
        bilan.renseigner_correspondances(&correspondances, &inventaire);
        bilan.renseigner_analyse(&analyse);
        bilan.renseigner_controle(&controle, divergences);

        let revision = self.manifeste.revision + 1;
        let instantane = ecrire_fichier_interne(
            &self.dossier,
            &format!("entrees/decisions-r{revision:03}.xlsx"),
            &contenu,
        )?;
        let reference = self.remplacer_edition(
            EDITION_DECISIONS,
            &contenu,
            self.manifeste.reference_edition_decisions.clone().as_ref(),
        )?;
        let mut manifeste = self.manifeste.clone();
        manifeste.revision = revision;
        manifeste.reference_edition_decisions = Some(reference);
        manifeste.decisions = Some(Adoption {
            instantane,
            fichier_lu,
            horodatage: maintenant(),
        });
        self.enregistrer(manifeste)?;
        Ok(bilan)
    }

    // Produit le SGX à partir des seuls instantanés adoptés et vérifiés : tout
    // est recalculé, puis écrit dans un nouveau dossier de tentative. Refusé si
    // un classeur d'édition a changé sans être relu.
    pub fn produire(
        &mut self,
        revision_attendue: u64,
        journal: &mut Journal<'_>,
    ) -> Resultat<Bilan> {
        self.verifier_revision(revision_attendue)?;
        self.verifier_integrite()?;
        let Some(decisions) = self.manifeste.decisions.clone() else {
            return Err(Erreur::nouvelle(
                "etape_prealable",
                "Importez et contrôlez les décisions avant de générer le SGX.",
            ));
        };
        let reference_decisions = self.manifeste.reference_edition_decisions.clone();
        let reference_correspondances = self.manifeste.reference_edition_correspondances.clone();
        if self.edition_modifiee(EDITION_CORRESPONDANCES, Some(&reference_correspondances))
            || self.edition_modifiee(EDITION_DECISIONS, reference_decisions.as_ref())
        {
            return Err(Erreur::nouvelle(
                "classeur_modifie",
                "Un classeur a changé depuis sa dernière lecture. Relisez-le pour actualiser le traitement avant de générer le SGX.",
            ));
        }

        let source = self.chemin_source()?;
        let (inventaire, correspondances) = self.charger_correspondances(journal)?;
        let analyse = crate::regles::dry_run(&correspondances.retenues, &inventaire.synthese);

        let numero = self.prochain_numero_tentative()?;
        let dossier_relatif = format!("sorties/tentative-{numero:03}");
        let dossier_tentative = self.dossier.join(&dossier_relatif);
        workflow::creer_dossier(&self.dossier.join("sorties"))?;
        fs::create_dir(&dossier_tentative).contexte(
            "ecriture_impossible",
            &format!(
                "Création du dossier impossible : {}",
                dossier_tentative.display()
            ),
        )?;
        journal(&format!("[OK] Tentative {numero} : {dossier_relatif}"));

        let controle = workflow::controler(
            &self.chemin_interne(&decisions.instantane.chemin)?,
            &analyse,
            &dossier_tentative,
            journal,
        )?;
        let (modeles, divergences) = if controle.validees.is_empty() {
            (Default::default(), Vec::new())
        } else {
            workflow::tester_renommages(&source, &controle.validees, journal)?
        };
        let mut bilan = Bilan::default();
        bilan.renseigner_inventaire(&inventaire);
        bilan.renseigner_correspondances(&correspondances, &inventaire);
        bilan.renseigner_analyse(&analyse);
        bilan.renseigner_controle(&controle, divergences);

        let mut sgx = None;
        if bilan.statut == Statut::ProductionPossible {
            let destination = dossier_tentative.join(format!("{}_modifie.sgx", self.manifeste.nom));
            workflow::publier(
                &source,
                &modeles,
                &controle.validees,
                &destination,
                &mut bilan,
                journal,
            )?;
            sgx = Some(FichierSuivi {
                chemin: format!("{dossier_relatif}/{}_modifie.sgx", self.manifeste.nom),
                empreinte: empreinte_fichier(&destination)?,
            });
        }
        workflow::ecrire_json(&dossier_tentative.join("bilan.json"), &bilan)?;

        let mut manifeste = self.manifeste.clone();
        manifeste.revision += 1;
        manifeste.tentatives.push(Tentative {
            numero,
            dossier: dossier_relatif,
            revision: manifeste.revision,
            horodatage: maintenant(),
            statut: bilan.statut,
            sgx,
            modeles_modifies: bilan.modeles_modifies,
            occurrences_modifiees: bilan.occurrences_modifiees,
        });
        if let Err(erreur) = self.enregistrer(manifeste) {
            return Err(Erreur::nouvelle(
                "etat_non_confirme",
                format!(
                    "Le résultat a été écrit dans {} mais le traitement n'a pas pu l'enregistrer : il n'est pas confirmé. ({})",
                    dossier_tentative.display(),
                    erreur.message
                ),
            ));
        }
        Ok(bilan)
    }

    // ------------------------------------------------------------ Interne

    fn verifier_revision(&self, revision_attendue: u64) -> Resultat<()> {
        if revision_attendue != self.manifeste.revision {
            return Err(Erreur::nouvelle(
                "revision_obsolete",
                format!(
                    "Le traitement a changé entre-temps (révision {} au lieu de {revision_attendue}) : actualisez l'affichage.",
                    self.manifeste.revision
                ),
            ));
        }
        Ok(())
    }

    // Vérifie la copie source et les instantanés adoptés : leur altération
    // bloque le traitement (A11, A12).
    fn verifier_integrite(&self) -> Resultat<()> {
        let mut suivis = vec![&self.manifeste.source.copie];
        suivis.extend(
            self.manifeste
                .correspondances
                .iter()
                .map(|adoption| &adoption.instantane),
        );
        suivis.extend(
            self.manifeste
                .decisions
                .iter()
                .map(|adoption| &adoption.instantane),
        );
        let mut alterations = Vec::new();
        for fichier in suivis {
            let chemin = self.chemin_interne(&fichier.chemin)?;
            match empreinte_fichier(&chemin) {
                Ok(empreinte) if empreinte == fichier.empreinte => {}
                Ok(_) => alterations.push(format!("{} : contenu modifié", fichier.chemin)),
                Err(_) => {
                    alterations.push(format!("{} : fichier absent ou illisible", fichier.chemin))
                }
            }
        }
        if alterations.is_empty() {
            Ok(())
        } else {
            Err(Erreur::nouvelle(
                "traitement_altere",
                "Des fichiers internes du traitement ont été modifiés ou supprimés : il ne peut pas être poursuivi. Créez un nouveau traitement depuis la source.",
            )
            .avec_details(alterations))
        }
    }

    // Résout un chemin du manifeste dans le dossier du traitement, en refusant
    // tout chemin absolu ou remontant hors du dossier.
    fn chemin_interne(&self, relatif: &str) -> Resultat<PathBuf> {
        let chemin = Path::new(relatif);
        let valide = !relatif.is_empty()
            && chemin
                .components()
                .all(|composant| matches!(composant, Component::Normal(_)));
        if !valide {
            return Err(Erreur::nouvelle(
                "traitement_illisible",
                format!("Chemin invalide dans le manifeste : {relatif}"),
            ));
        }
        Ok(self.dossier.join(chemin))
    }

    fn chemin_source(&self) -> Resultat<PathBuf> {
        self.chemin_interne(&self.manifeste.source.copie.chemin)
    }

    fn inventorier(&self, journal: &mut Journal<'_>) -> Resultat<Inventaire> {
        workflow::inventorier(&self.chemin_source()?, journal)
    }

    fn charger_correspondances(
        &self,
        journal: &mut Journal<'_>,
    ) -> Resultat<(Inventaire, crate::regles::Correspondances)> {
        let Some(adoption) = &self.manifeste.correspondances else {
            return Err(Erreur::nouvelle(
                "etape_prealable",
                "Lisez un classeur de correspondance avant de préparer l'analyse.",
            ));
        };
        let inventaire = self.inventorier(journal)?;
        let correspondances = workflow::lire_correspondances(
            &self.chemin_interne(&adoption.instantane.chemin)?,
            &inventaire,
            journal,
        )?;
        Ok((inventaire, correspondances))
    }

    // Contenu du classeur à lire : fichier fourni, ou classeur d'édition.
    fn lire_classeur_fourni(
        &self,
        fichier: Option<&Path>,
        edition: &str,
    ) -> Resultat<(Vec<u8>, String)> {
        let chemin = match fichier {
            Some(chemin) => chemin.to_path_buf(),
            None => self.dossier.join(edition),
        };
        let contenu = fs::read(&chemin).contexte(
            "classeur_illisible",
            &format!(
                "Impossible de lire ce classeur. Enregistrez-le, fermez-le puis réessayez : {}",
                chemin.display()
            ),
        )?;
        Ok((contenu, chemin.display().to_string()))
    }

    // Vrai si le classeur d'édition diffère de sa version de référence.
    fn edition_modifiee(&self, edition: &str, reference: Option<&String>) -> bool {
        let chemin = self.dossier.join(edition);
        match (reference, empreinte_fichier(&chemin)) {
            (None, _) => false,
            (Some(reference), Ok(empreinte)) => &empreinte != reference,
            (Some(_), Err(_)) => true,
        }
    }

    // Remplace un classeur d'édition par `contenu`. S'il avait été modifié sans
    // être adopté, l'ancienne version est d'abord conservée dans edition/precedents/.
    // Retourne la nouvelle empreinte de référence.
    fn remplacer_edition(
        &self,
        edition: &str,
        contenu: &[u8],
        reference: Option<&String>,
    ) -> Resultat<String> {
        let chemin = self.dossier.join(edition);
        if let Ok(empreinte_actuelle) = empreinte_fichier(&chemin) {
            if empreinte_actuelle == empreinte(contenu) {
                return Ok(empreinte_actuelle);
            }
            if reference != Some(&empreinte_actuelle) {
                self.archiver_edition(edition)?;
            }
        }
        Ok(ecrire_fichier_interne(&self.dossier, edition, contenu)?.empreinte)
    }

    // Déplace un classeur d'édition dans edition/precedents/ (s'il existe).
    fn archiver_edition(&self, edition: &str) -> Resultat<()> {
        let chemin = self.dossier.join(edition);
        if !chemin.exists() {
            return Ok(());
        }
        let precedents = self.dossier.join("edition/precedents");
        workflow::creer_dossier(&precedents)?;
        let nom = Path::new(edition)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let mut destination = precedents.join(format!("{nom}-{}.xlsx", maintenant()));
        let mut numero = 2;
        while destination.exists() {
            destination = precedents.join(format!("{nom}-{}-{numero}.xlsx", maintenant()));
            numero += 1;
        }
        fs::rename(&chemin, &destination).contexte(
            "ecriture_impossible",
            &format!(
                "Impossible de conserver le classeur modifié : {}",
                chemin.display()
            ),
        )
    }

    fn fichier_temporaire(&self, nom: &str, contenu: &[u8]) -> Resultat<PathBuf> {
        let chemin = self.dossier.join(nom);
        fs::write(&chemin, contenu).contexte(
            "ecriture_impossible",
            &format!("Écriture impossible : {}", chemin.display()),
        )?;
        Ok(chemin)
    }

    fn prochain_numero_tentative(&self) -> Resultat<u32> {
        let mut plus_grand = self
            .manifeste
            .tentatives
            .iter()
            .map(|tentative| tentative.numero)
            .max()
            .unwrap_or(0);
        for dossier in self.dossiers_tentatives() {
            if let Some(numero) = numero_tentative(&dossier) {
                plus_grand = plus_grand.max(numero);
            }
        }
        Ok(plus_grand + 1)
    }

    fn dossiers_tentatives(&self) -> Vec<PathBuf> {
        let mut dossiers: Vec<PathBuf> = fs::read_dir(self.dossier.join("sorties"))
            .into_iter()
            .flatten()
            .flatten()
            .map(|entree| entree.path())
            .filter(|chemin| chemin.is_dir() && numero_tentative(chemin).is_some())
            .collect();
        dossiers.sort();
        dossiers
    }

    fn tentatives_interrompues(&self) -> Vec<PathBuf> {
        self.dossiers_tentatives()
            .into_iter()
            .filter(|dossier| {
                let numero = numero_tentative(dossier);
                !self
                    .manifeste
                    .tentatives
                    .iter()
                    .any(|tentative| Some(tentative.numero) == numero)
            })
            .collect()
    }

    fn enregistrer(&mut self, manifeste: Manifeste) -> Resultat<()> {
        enregistrer_manifeste(&self.dossier, &manifeste)?;
        self.manifeste = manifeste;
        Ok(())
    }
}

// ---------------------------------------------------------------- Fichiers

fn numero_tentative(dossier: &Path) -> Option<u32> {
    dossier
        .file_name()?
        .to_str()?
        .strip_prefix("tentative-")?
        .parse()
        .ok()
}

// Crée « base », ou « base (2) », « base (3) »... sans jamais réutiliser un dossier existant.
fn creer_sous_dossier(parent: &Path, base: &str) -> Resultat<PathBuf> {
    fs::create_dir_all(parent).contexte(
        "ecriture_impossible",
        &format!("Création du dossier impossible : {}", parent.display()),
    )?;
    for numero in 1.. {
        let nom = if numero == 1 {
            base.to_string()
        } else {
            format!("{base} ({numero})")
        };
        let dossier = parent.join(nom);
        match fs::create_dir(&dossier) {
            Ok(()) => return Ok(dossier),
            Err(erreur) if erreur.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(erreur) => {
                return Err(Erreur::nouvelle(
                    "ecriture_impossible",
                    format!(
                        "Création du dossier impossible : {} ({erreur})",
                        dossier.display()
                    ),
                ));
            }
        }
    }
    unreachable!("la boucle ne se termine que par un retour")
}

// Verrou système exclusif : libéré automatiquement à la fermeture, même en cas d'arrêt brutal.
fn verrouiller(dossier: &Path) -> Resultat<File> {
    let chemin = dossier.join(VERROU);
    let fichier = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&chemin)
        .contexte(
            "ecriture_impossible",
            &format!(
                "Impossible de verrouiller le traitement : {}",
                chemin.display()
            ),
        )?;
    if fichier.try_lock().is_err() {
        return Err(Erreur::nouvelle(
            "traitement_deja_ouvert",
            "Ce traitement est déjà ouvert dans une autre fenêtre ou une autre instance de l'application.",
        ));
    }
    Ok(fichier)
}

// Écrit un fichier interne par remplacement contrôlé (temporaire puis renommage)
// et retourne son suivi.
fn ecrire_fichier_interne(dossier: &Path, relatif: &str, contenu: &[u8]) -> Resultat<FichierSuivi> {
    let chemin = dossier.join(relatif);
    if let Some(parent) = chemin.parent() {
        workflow::creer_dossier(parent)?;
    }
    let message = format!("Écriture impossible : {}", chemin.display());
    let mut nom_temporaire = chemin.file_name().unwrap_or_default().to_os_string();
    nom_temporaire.push(".en-cours");
    let temporaire = chemin.with_file_name(nom_temporaire);
    let resultat = (|| -> std::io::Result<()> {
        let mut fichier = File::create(&temporaire)?;
        fichier.write_all(contenu)?;
        fichier.sync_all()?;
        fs::rename(&temporaire, &chemin)
    })();
    if resultat.is_err() {
        let _ = fs::remove_file(&temporaire);
    }
    resultat.contexte("ecriture_impossible", &message)?;
    Ok(FichierSuivi {
        chemin: relatif.to_string(),
        empreinte: empreinte(contenu),
    })
}

fn enregistrer_manifeste(dossier: &Path, manifeste: &Manifeste) -> Resultat<()> {
    let texte = serde_json::to_string_pretty(manifeste)?;
    ecrire_fichier_interne(dossier, MANIFESTE, texte.as_bytes())?;
    Ok(())
}

fn empreinte(contenu: &[u8]) -> String {
    Sha256::digest(contenu)
        .iter()
        .map(|octet| format!("{octet:02x}"))
        .collect()
}

fn empreinte_fichier(chemin: &Path) -> std::io::Result<String> {
    Ok(empreinte(&fs::read(chemin)?))
}

fn maintenant() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duree| duree.as_secs())
        .unwrap_or_default()
}
