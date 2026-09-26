// Tests du dossier de traitement (lot M4, P01/P04) : création, reprise,
// verrou, révisions, invalidations, intégrité et tentatives (A11 à A14).

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use bpmn_script_rs::{Etape, EtatFichier, Statut, Traitement};
use rust_xlsxwriter::Workbook;
use serde_json::{Value, json};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

const MODELE_A: &str = "a/model_1_.json";
const MODELE_B: &str = "b/model_1_.json";

fn lane(nom: &str) -> Value {
    json!({"stencil": {"id": "Lane"}, "properties": {"name": nom}, "childShapes": []})
}

// Dossier temporaire contenant un SGX source (A: 2 lanes « A », B: 1 lane « A »).
struct Espace(PathBuf);

impl Drop for Espace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl Espace {
    fn nouveau(nom: &str) -> Espace {
        static COMPTEUR: AtomicUsize = AtomicUsize::new(0);
        let numero = COMPTEUR.fetch_add(1, Ordering::SeqCst);
        let dossier = std::env::temp_dir().join(format!(
            "bpmn-traitement-{}-{numero}-{nom}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dossier);
        fs::create_dir_all(&dossier).unwrap();
        let espace = Espace(dossier);
        espace.ecrire_sgx(&[
            (
                MODELE_A,
                json!({"childShapes": [lane("A"), lane("A"), lane("B")]}),
            ),
            (MODELE_B, json!({"childShapes": [lane("A")]})),
        ]);
        espace
    }

    fn source(&self) -> PathBuf {
        self.0.join("export.sgx")
    }

    fn parent(&self) -> PathBuf {
        self.0.join("traitements")
    }

    fn ecrire_sgx(&self, modeles: &[(&str, Value)]) {
        let mut zip = ZipWriter::new(File::create(self.source()).unwrap());
        for (chemin, modele) in modeles {
            zip.start_file(*chemin, SimpleFileOptions::default())
                .unwrap();
            zip.write_all(modele.to_string().as_bytes()).unwrap();
            zip.start_file(
                chemin.replace("model_1_", "model_meta"),
                SimpleFileOptions::default(),
            )
            .unwrap();
            zip.write_all(br#"{"name": "Flux"}"#).unwrap();
        }
        zip.finish().unwrap();
    }

    fn creer(&self) -> Traitement {
        Traitement::creer(&self.source(), &self.parent(), &mut |_| {})
            .unwrap()
            .0
    }
}

fn ecrire_xlsx(chemin: &Path, feuille: &str, lignes: &[Vec<Value>]) {
    let mut classeur = Workbook::new();
    let feuille_excel = classeur.add_worksheet().set_name(feuille).unwrap();
    for (numero_ligne, ligne) in lignes.iter().enumerate() {
        for (numero_colonne, valeur) in ligne.iter().enumerate() {
            let (numero_ligne, numero_colonne) = (numero_ligne as u32, numero_colonne as u16);
            match valeur {
                Value::String(texte) => {
                    feuille_excel
                        .write_string(numero_ligne, numero_colonne, texte)
                        .unwrap();
                }
                Value::Number(nombre) => {
                    feuille_excel
                        .write_number(numero_ligne, numero_colonne, nombre.as_f64().unwrap())
                        .unwrap();
                }
                _ => {}
            }
        }
    }
    classeur.save(chemin).unwrap();
}

fn correspondances(chemin: &Path, lignes: &[(&str, &str)]) {
    let mut contenu = vec![vec![json!("Nom actuel"), json!("Nouveau nom")]];
    contenu.extend(
        lignes
            .iter()
            .map(|(ancien, nouveau)| vec![json!(ancien), json!(nouveau)]),
    );
    ecrire_xlsx(chemin, "Correspondance", &contenu);
}

fn decisions(chemin: &Path, lignes: &[(&str, &str, &str, u64, &str)]) {
    let mut contenu = vec![
        [
            "Flux",
            "Nom actuel",
            "Nouveau nom",
            "Occurrences",
            "Validation",
            "Fichier modèle",
        ]
        .map(|titre| json!(titre))
        .to_vec(),
    ];
    contenu.extend(
        lignes
            .iter()
            .map(|(modele, ancien, nouveau, occurrences, validation)| {
                vec![
                    json!("Flux"),
                    json!(ancien),
                    json!(nouveau),
                    json!(occurrences),
                    json!(validation),
                    json!(modele),
                ]
            }),
    );
    ecrire_xlsx(chemin, "Analyse", &contenu);
}

fn decisions_a_z(chemin: &Path, validation_a: &str, validation_b: &str) {
    decisions(
        chemin,
        &[
            (MODELE_A, "A", "Z", 2, validation_a),
            (MODELE_B, "A", "Z", 1, validation_b),
        ],
    );
}

fn lanes_du_sgx(chemin: &Path, modele: &str) -> Vec<String> {
    let mut archive = ZipArchive::new(File::open(chemin).unwrap()).unwrap();
    let mut contenu = String::new();
    archive
        .by_name(modele)
        .unwrap()
        .read_to_string(&mut contenu)
        .unwrap();
    let modele: Value = serde_json::from_str(&contenu).unwrap();
    modele["childShapes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|lane| lane["properties"]["name"].as_str().unwrap().to_string())
        .collect()
}

// Mène un traitement jusqu'aux décisions contrôlées (A -> Z, OUI / OUI).
fn jusqu_aux_decisions(traitement: &mut Traitement) {
    let etat = traitement.etat().unwrap();
    correspondances(&etat.edition_correspondances, &[("A", "Z")]);
    traitement
        .adopter_correspondances(traitement.revision(), None, &mut |_| {})
        .unwrap();
    traitement
        .preparer_analyse(traitement.revision(), &mut |_| {})
        .unwrap();
    let edition = traitement.etat().unwrap().edition_decisions.unwrap();
    decisions_a_z(&edition, "OUI", "OUI");
    let bilan = traitement
        .adopter_decisions(traitement.revision(), None, &mut |_| {})
        .unwrap();
    assert_eq!(bilan.statut, Statut::ProductionPossible);
}

// ---------------------------------------------------------------- Parcours

#[test]
fn parcours_complet() {
    let espace = Espace::nouveau("parcours");
    let source_avant = fs::read(espace.source()).unwrap();
    let (mut traitement, bilan) =
        Traitement::creer(&espace.source(), &espace.parent(), &mut |_| {}).unwrap();
    assert_eq!(
        traitement.dossier(),
        espace.parent().join("export - traitement")
    );
    assert_eq!(
        (
            bilan.modeles_reconnus,
            bilan.occurrences_inventoriees,
            bilan.noms_distincts
        ),
        (2, 4, 2)
    );

    let etat = traitement.etat().unwrap();
    assert_eq!(etat.etape, Etape::InventairePret);
    assert_eq!(etat.revision, 1);
    assert!(etat.edition_correspondances.exists());
    assert!(!etat.correspondances_a_relire);

    // Correspondances saisies dans le classeur d'édition : à relire.
    correspondances(&etat.edition_correspondances, &[("A", "Z"), ("X", "Y")]);
    assert!(traitement.etat().unwrap().correspondances_a_relire);
    let bilan = traitement
        .adopter_correspondances(1, None, &mut |_| {})
        .unwrap();
    assert_eq!(bilan.correspondances_retenues, 2);
    assert_eq!(bilan.noms_inconnus, ["X"]);
    let etat = traitement.etat().unwrap();
    assert_eq!(etat.etape, Etape::CorrespondancesPretes);
    assert!(!etat.correspondances_a_relire);

    let bilan = traitement.preparer_analyse(2, &mut |_| {}).unwrap();
    assert_eq!(
        (bilan.statut, bilan.propositions, bilan.occurrences_visees),
        (Statut::DecisionsAttendues, 2, 3)
    );
    let etat = traitement.etat().unwrap();
    assert_eq!(etat.etape, Etape::AnalysePrete);
    let edition_decisions = etat.edition_decisions.unwrap();

    decisions_a_z(&edition_decisions, "OUI", "NON");
    let bilan = traitement.adopter_decisions(3, None, &mut |_| {}).unwrap();
    assert_eq!(
        (bilan.statut, bilan.lignes_admises, bilan.lignes_refusees),
        (Statut::ProductionPossible, 1, 1)
    );
    let etat = traitement.etat().unwrap();
    assert_eq!(etat.etape, Etape::DecisionsControlees);
    assert!(etat.fichier_controle.unwrap().exists());

    let bilan = traitement.produire(4, &mut |_| {}).unwrap();
    assert_eq!(bilan.statut, Statut::SgxProduit);
    let sgx = bilan.sgx_produit.unwrap();
    assert_eq!(
        sgx,
        traitement
            .dossier()
            .join("sorties/tentative-001/export_modifie.sgx")
    );
    assert_eq!(lanes_du_sgx(&sgx, MODELE_A), ["Z", "Z", "B"]);
    assert_eq!(lanes_du_sgx(&sgx, MODELE_B), ["A"]);

    let etat = traitement.etat().unwrap();
    assert_eq!(etat.etape, Etape::ResultatProduit);
    assert_eq!(etat.tentatives.len(), 1);
    assert_eq!(etat.tentatives[0].etat_sgx, Some(EtatFichier::Disponible));
    assert!(etat.tentatives[0].courante);
    assert_eq!(fs::read(espace.source()).unwrap(), source_avant);
}

#[test]
fn reprise_apres_fermeture_et_original_deplace() {
    let espace = Espace::nouveau("reprise");
    let dossier = {
        let mut traitement = espace.creer();
        jusqu_aux_decisions(&mut traitement);
        traitement.dossier().to_path_buf()
    };
    fs::rename(espace.source(), espace.0.join("deplace.sgx")).unwrap();

    let mut traitement = Traitement::ouvrir(&dossier).unwrap();
    let etat = traitement.etat().unwrap();
    assert_eq!(etat.etape, Etape::DecisionsControlees);
    assert_eq!(etat.source_nom, "export.sgx");
    let bilan = traitement.produire(etat.revision, &mut |_| {}).unwrap();
    assert_eq!(bilan.statut, Statut::SgxProduit);
}

#[test]
fn deuxieme_ouverture_refusee_tant_que_le_traitement_est_ouvert() {
    let espace = Espace::nouveau("verrou");
    let traitement = espace.creer();
    let erreur = Traitement::ouvrir(traitement.dossier()).err().unwrap();
    assert_eq!(erreur.code, "traitement_deja_ouvert");
    let dossier = traitement.dossier().to_path_buf();
    drop(traitement);
    assert!(Traitement::ouvrir(&dossier).is_ok());
}

#[test]
fn revision_obsolete_refusee() {
    let espace = Espace::nouveau("revision");
    let mut traitement = espace.creer();
    correspondances(
        &traitement.etat().unwrap().edition_correspondances,
        &[("A", "Z")],
    );
    traitement
        .adopter_correspondances(1, None, &mut |_| {})
        .unwrap();
    // Une deuxième demande fondée sur la révision 1 (double clic) est refusée.
    let erreur = traitement
        .adopter_correspondances(1, None, &mut |_| {})
        .err()
        .unwrap();
    assert_eq!(erreur.code, "revision_obsolete");
    let erreur = traitement.produire(1, &mut |_| {}).err().unwrap();
    assert_eq!(erreur.code, "revision_obsolete");
}

// ---------------------------------------------------------------- Intégrité

#[test]
fn copie_source_alteree_bloque_la_reprise() {
    let espace = Espace::nouveau("source-alteree");
    let dossier = espace.creer().dossier().to_path_buf();
    fs::write(dossier.join("source/export.sgx"), b"altere").unwrap();
    let erreur = Traitement::ouvrir(&dossier).err().unwrap();
    assert_eq!(erreur.code, "traitement_altere");
    assert_eq!(erreur.details, ["source/export.sgx : contenu modifié"]);
}

#[test]
fn instantane_altere_ou_supprime_bloque_la_reprise() {
    let espace = Espace::nouveau("instantane");
    let dossier = {
        let mut traitement = espace.creer();
        jusqu_aux_decisions(&mut traitement);
        traitement.dossier().to_path_buf()
    };
    fs::remove_file(dossier.join("entrees/decisions-r004.xlsx")).unwrap();
    fs::write(dossier.join("entrees/correspondances-r002.xlsx"), b"altere").unwrap();
    let erreur = Traitement::ouvrir(&dossier).err().unwrap();
    assert_eq!(erreur.code, "traitement_altere");
    assert_eq!(
        erreur.details,
        [
            "entrees/correspondances-r002.xlsx : contenu modifié",
            "entrees/decisions-r004.xlsx : fichier absent ou illisible"
        ]
    );
}

#[test]
fn manifeste_de_version_inconnue_refuse_sans_reecriture() {
    let espace = Espace::nouveau("version");
    let dossier = espace.creer().dossier().to_path_buf();
    let chemin = dossier.join("traitement.json");
    let mut manifeste: Value = serde_json::from_str(&fs::read_to_string(&chemin).unwrap()).unwrap();
    manifeste["version_schema"] = json!(99);
    let texte = serde_json::to_string(&manifeste).unwrap();
    fs::write(&chemin, &texte).unwrap();
    let erreur = Traitement::ouvrir(&dossier).err().unwrap();
    assert_eq!(erreur.code, "traitement_version_inconnue");
    assert_eq!(fs::read_to_string(&chemin).unwrap(), texte);

    fs::write(&chemin, b"{").unwrap();
    assert_eq!(
        Traitement::ouvrir(&dossier).err().unwrap().code,
        "traitement_illisible"
    );
}

#[test]
fn chemin_hors_du_traitement_refuse() {
    let espace = Espace::nouveau("chemin");
    let dossier = espace.creer().dossier().to_path_buf();
    let chemin = dossier.join("traitement.json");
    let mut manifeste: Value = serde_json::from_str(&fs::read_to_string(&chemin).unwrap()).unwrap();
    manifeste["source"]["copie"]["chemin"] = json!("../../export.sgx");
    fs::write(&chemin, serde_json::to_string(&manifeste).unwrap()).unwrap();
    assert_eq!(
        Traitement::ouvrir(&dossier).err().unwrap().code,
        "traitement_illisible"
    );
}

// ---------------------------------------------------------------- Classeurs

#[test]
fn classeur_modifie_apres_lecture_suspend_la_production() {
    let espace = Espace::nouveau("classeur-modifie");
    let mut traitement = espace.creer();
    jusqu_aux_decisions(&mut traitement);
    let edition = traitement.etat().unwrap().edition_decisions.unwrap();
    decisions_a_z(&edition, "OUI", "NON");
    assert!(traitement.etat().unwrap().decisions_a_relire);

    let erreur = traitement
        .produire(traitement.revision(), &mut |_| {})
        .err()
        .unwrap();
    assert_eq!(erreur.code, "classeur_modifie");
    assert!(!traitement.dossier().join("sorties").exists());

    let bilan = traitement
        .adopter_decisions(traitement.revision(), None, &mut |_| {})
        .unwrap();
    assert_eq!(bilan.lignes_admises, 1);
    let bilan = traitement
        .produire(traitement.revision(), &mut |_| {})
        .unwrap();
    assert_eq!(bilan.occurrences_modifiees, 2);
}

#[test]
fn nouvelles_correspondances_invalident_analyse_et_decisions() {
    let espace = Espace::nouveau("invalidation");
    let mut traitement = espace.creer();
    jusqu_aux_decisions(&mut traitement);
    let edition_decisions = traitement.etat().unwrap().edition_decisions.unwrap();
    // Décisions modifiées puis correspondances changées : les décisions sont obsolètes.
    decisions_a_z(&edition_decisions, "NON", "NON");
    correspondances(
        &traitement.etat().unwrap().edition_correspondances,
        &[("A", "Y")],
    );
    traitement
        .adopter_correspondances(traitement.revision(), None, &mut |_| {})
        .unwrap();
    let etat = traitement.etat().unwrap();
    assert_eq!(etat.etape, Etape::CorrespondancesPretes);
    assert!(!etat.analyse_preparee && !etat.decisions_adoptees);
    let erreur = traitement
        .produire(traitement.revision(), &mut |_| {})
        .err()
        .unwrap();
    assert_eq!(erreur.code, "etape_prealable");

    // La nouvelle analyse remplace le classeur de décision ; la version modifiée
    // non adoptée est conservée dans edition/precedents/.
    traitement
        .preparer_analyse(traitement.revision(), &mut |_| {})
        .unwrap();
    let precedents: Vec<_> = fs::read_dir(traitement.dossier().join("edition/precedents"))
        .unwrap()
        .collect();
    assert_eq!(precedents.len(), 1);
    assert!(!traitement.etat().unwrap().decisions_a_relire);
}

#[test]
fn import_invalide_conserve_l_adoption_precedente() {
    let espace = Espace::nouveau("import-invalide");
    let mut traitement = espace.creer();
    correspondances(
        &traitement.etat().unwrap().edition_correspondances,
        &[("A", "Z")],
    );
    traitement
        .adopter_correspondances(1, None, &mut |_| {})
        .unwrap();

    let externe = espace.0.join("externe.xlsx");
    ecrire_xlsx(
        &externe,
        "Correspondance",
        &[
            vec![json!("Nom actuel"), json!("Nouveau nom")],
            vec![json!("A"), json!(123)],
        ],
    );
    let externe_avant = fs::read(&externe).unwrap();
    let erreur = traitement
        .adopter_correspondances(2, Some(&externe), &mut |_| {})
        .err()
        .unwrap();
    assert_eq!(erreur.code, "classeur_cellules");
    let etat = traitement.etat().unwrap();
    assert_eq!(
        (etat.revision, etat.etape),
        (2, Etape::CorrespondancesPretes)
    );
    assert_eq!(fs::read(&externe).unwrap(), externe_avant);
    assert!(!traitement.dossier().join("lecture-en-cours.xlsx").exists());

    // Fichier absent ou illisible : même garantie.
    let erreur = traitement
        .adopter_correspondances(2, Some(&espace.0.join("absent.xlsx")), &mut |_| {})
        .err()
        .unwrap();
    assert_eq!(erreur.code, "classeur_illisible");
    assert_eq!(traitement.revision(), 2);
}

#[test]
fn decisions_d_un_arbitre_importees_depuis_un_fichier_externe() {
    let espace = Espace::nouveau("arbitre");
    let mut traitement = espace.creer();
    correspondances(
        &traitement.etat().unwrap().edition_correspondances,
        &[("A", "Z")],
    );
    traitement
        .adopter_correspondances(1, None, &mut |_| {})
        .unwrap();
    traitement.preparer_analyse(2, &mut |_| {}).unwrap();

    // Retour partiel : seule la ligne du modèle B est présente.
    let retour = espace.0.join("retour-arbitre.xlsx");
    decisions(&retour, &[(MODELE_B, "A", "Z", 1, "OUI")]);
    let bilan = traitement
        .adopter_decisions(3, Some(&retour), &mut |_| {})
        .unwrap();
    assert_eq!(
        (bilan.lignes_admises, bilan.propositions_sans_decision),
        (1, 1)
    );
    assert!(!traitement.etat().unwrap().decisions_a_relire);

    let sgx = traitement
        .produire(4, &mut |_| {})
        .unwrap()
        .sgx_produit
        .unwrap();
    assert_eq!(lanes_du_sgx(&sgx, MODELE_A), ["A", "A", "B"]);
    assert_eq!(lanes_du_sgx(&sgx, MODELE_B), ["Z"]);
}

#[test]
fn analyse_sans_impact_retire_l_ancien_classeur_de_decision() {
    let espace = Espace::nouveau("sans-impact");
    let mut traitement = espace.creer();
    jusqu_aux_decisions(&mut traitement);
    correspondances(
        &traitement.etat().unwrap().edition_correspondances,
        &[("Inconnu", "Z")],
    );
    traitement
        .adopter_correspondances(traitement.revision(), None, &mut |_| {})
        .unwrap();
    let bilan = traitement
        .preparer_analyse(traitement.revision(), &mut |_| {})
        .unwrap();
    assert_eq!(bilan.statut, Statut::AnalyseSansImpact);
    assert!(traitement.etat().unwrap().edition_decisions.is_none());
    let erreur = traitement
        .adopter_decisions(traitement.revision(), None, &mut |_| {})
        .err()
        .unwrap();
    assert_eq!(erreur.code, "etape_prealable");
}

// ---------------------------------------------------------------- Tentatives

#[test]
fn production_sans_decision_admissible_enregistree_sans_sgx() {
    let espace = Espace::nouveau("sans-admissible");
    let mut traitement = espace.creer();
    jusqu_aux_decisions(&mut traitement);
    decisions_a_z(
        &traitement.etat().unwrap().edition_decisions.unwrap(),
        "NON",
        "NON",
    );
    let bilan = traitement
        .adopter_decisions(traitement.revision(), None, &mut |_| {})
        .unwrap();
    assert_eq!(bilan.statut, Statut::AucuneDecisionAdmissible);

    // Le moteur refait le contrôle : aucune production, même sollicitée.
    let bilan = traitement
        .produire(traitement.revision(), &mut |_| {})
        .unwrap();
    assert_eq!(bilan.statut, Statut::AucuneDecisionAdmissible);
    let etat = traitement.etat().unwrap();
    assert_eq!(etat.etape, Etape::DecisionsControlees);
    assert_eq!(etat.tentatives[0].sgx, None);
    let fichiers: Vec<_> = fs::read_dir(traitement.dossier().join("sorties/tentative-001"))
        .unwrap()
        .map(|entree| entree.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|nom| nom.ends_with(".sgx"))
        .collect();
    assert!(fichiers.is_empty());
}

#[test]
fn sgx_produit_supprime_ou_modifie_signale() {
    let espace = Espace::nouveau("sgx-altere");
    let mut traitement = espace.creer();
    jusqu_aux_decisions(&mut traitement);
    let premier = traitement
        .produire(traitement.revision(), &mut |_| {})
        .unwrap()
        .sgx_produit
        .unwrap();
    let second = traitement
        .produire(traitement.revision(), &mut |_| {})
        .unwrap()
        .sgx_produit
        .unwrap();
    assert_ne!(premier, second);
    fs::remove_file(&premier).unwrap();
    fs::write(&second, b"altere").unwrap();
    let etat = traitement.etat().unwrap();
    assert_eq!(etat.tentatives[0].etat_sgx, Some(EtatFichier::Absent));
    assert_eq!(etat.tentatives[1].etat_sgx, Some(EtatFichier::Modifie));
    assert!(!etat.tentatives[0].courante && etat.tentatives[1].courante);
}

#[test]
fn tentative_interrompue_signalee_et_jamais_reutilisee() {
    let espace = Espace::nouveau("interrompue");
    let mut traitement = espace.creer();
    jusqu_aux_decisions(&mut traitement);
    // Arrêt simulé pendant une production : dossier sans enregistrement.
    let interrompue = traitement.dossier().join("sorties/tentative-001");
    fs::create_dir_all(&interrompue).unwrap();
    fs::write(interrompue.join("export_modifie.sgx"), b"partiel").unwrap();

    let etat = traitement.etat().unwrap();
    assert_eq!(
        etat.tentatives_interrompues,
        std::slice::from_ref(&interrompue)
    );
    let sgx = traitement
        .produire(traitement.revision(), &mut |_| {})
        .unwrap()
        .sgx_produit
        .unwrap();
    assert_eq!(
        sgx,
        traitement
            .dossier()
            .join("sorties/tentative-002/export_modifie.sgx")
    );
    assert_eq!(
        fs::read(interrompue.join("export_modifie.sgx")).unwrap(),
        b"partiel"
    );
}

// ---------------------------------------------------------------- Création

#[test]
fn creation_en_echec_ne_laisse_aucun_dossier() {
    let espace = Espace::nouveau("creation-echec");
    espace.ecrire_sgx(&[(MODELE_A, json!({"childShapes": 7}))]);
    let erreur = Traitement::creer(&espace.source(), &espace.parent(), &mut |_| {})
        .err()
        .unwrap();
    assert_eq!(erreur.code, "modeles_invalides");
    assert_eq!(fs::read_dir(espace.parent()).unwrap().count(), 0);
}

#[test]
fn deux_traitements_de_la_meme_source_dans_des_dossiers_distincts() {
    let espace = Espace::nouveau("homonymes");
    let premier = espace.creer();
    let second = espace.creer();
    assert_eq!(
        premier.dossier(),
        espace.parent().join("export - traitement")
    );
    assert_eq!(
        second.dossier(),
        espace.parent().join("export - traitement (2)")
    );
}
