// Tests d'intégration du CLI historique.
//
// Chaque test exécute le binaire dans un dossier temporaire contenant input/,
// work/ et éventuellement output/, avec un SGX et des Excel synthétiques, et
// vérifie que la source n'a pas changé. Les attentes sont celles du contrat V1
// (docs/m3-contrats-v1.md) ; les tests marqués « caractérisation » figent un
// comportement historique conservé.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use calamine::{Data, Reader, Xlsx, open_workbook};
use rust_xlsxwriter::{Formula, Workbook};
use serde_json::{Value, json};
use zip::write::FullFileOptions;
use zip::{CompressionMethod, DateTime, ZipArchive, ZipWriter};

const MODELE_A: &str = "dossier/a/model_1_.json";
const MODELE_B: &str = "dossier/b/model_1_.json";
const PIECE_JOINTE: &str = "piece_jointe.bin";
const NON_CIBLE: &str = "dossier/c/model_2_.json";
const FLUX: &str = "Même flux éà";
const SORTIE_SGX: &str = "output/export_modifie.sgx";

// ---------------------------------------------------------------- Fixtures

fn lane(nom: Value, enfants: Vec<Value>) -> Value {
    json!({"stencil": {"id": "Lane"}, "properties": {"name": nom}, "childShapes": enfants})
}

// Modèle A : lane « A » imbriquée deux fois (dont une avec espaces), tâche
// homonyme « A », lane « B » et lanes vides. Modèle B : une lane « A », même
// nom de flux que A (modèles homonymes distingués par leur chemin).
fn modele_a() -> Value {
    json!({"resourceId": "canvas-a", "inconnu": {"garder": [1, 2.5, "x"]}, "childShapes": [
        lane(json!(" A "), vec![
            lane(json!("A"), vec![]),
            json!({"stencil": {"id": "Task"}, "properties": {"name": "A"}, "childShapes": []}),
        ]),
        lane(json!("B"), vec![]),
        lane(json!(""), vec![]),
        lane(json!("   "), vec![]),
    ]})
}

fn modele_b() -> Value {
    json!({"childShapes": [lane(json!("A"), vec![])]})
}

fn metadonnees() -> Value {
    json!({"name": FLUX})
}

fn entrees_standard() -> Vec<(String, Vec<u8>)> {
    vec![
        (MODELE_A.into(), modele_a().to_string().into_bytes()),
        (
            MODELE_A.replace("model_1_", "model_meta"),
            metadonnees().to_string().into_bytes(),
        ),
        (MODELE_B.into(), modele_b().to_string().into_bytes()),
        (
            MODELE_B.replace("model_1_", "model_meta"),
            metadonnees().to_string().into_bytes(),
        ),
        (PIECE_JOINTE.into(), b"\x00\xffconserver\r\n".to_vec()),
        (NON_CIBLE.into(), br#"{"untouched":true}"#.to_vec()),
    ]
}

fn ecrire_zip(chemin: &Path, entrees: &[(String, Vec<u8>)]) {
    let mut zip = ZipWriter::new(File::create(chemin).unwrap());
    let options = FullFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (nom, contenu) in entrees {
        zip.start_file(nom.as_str(), options.clone()).unwrap();
        zip.write_all(contenu).unwrap();
    }
    zip.finish().unwrap();
}

// Contenu d'une cellule d'un Excel de saisie.
#[derive(Clone)]
enum Cellule {
    Texte(&'static str),
    Nombre(f64),
    // Formule et éventuelle valeur calculée enregistrée dans le fichier.
    Formule(&'static str, Option<&'static str>),
    Vide,
}

use Cellule::{Nombre, Texte, Vide};

fn ecrire_xlsx(chemin: &Path, feuille: &str, lignes: &[Vec<Cellule>]) {
    let mut classeur = Workbook::new();
    let feuille_excel = classeur.add_worksheet().set_name(feuille).unwrap();
    for (numero_ligne, ligne) in lignes.iter().enumerate() {
        for (numero_colonne, cellule) in ligne.iter().enumerate() {
            let (numero_ligne, numero_colonne) = (numero_ligne as u32, numero_colonne as u16);
            match cellule {
                Texte(texte) => {
                    feuille_excel
                        .write_string(numero_ligne, numero_colonne, *texte)
                        .unwrap();
                }
                Nombre(nombre) => {
                    feuille_excel
                        .write_number(numero_ligne, numero_colonne, *nombre)
                        .unwrap();
                }
                Cellule::Formule(formule, valeur) => {
                    let mut formule = Formula::new(*formule);
                    if let Some(valeur) = valeur {
                        formule = formule.set_result(*valeur);
                    }
                    feuille_excel
                        .write_formula(numero_ligne, numero_colonne, formule)
                        .unwrap();
                }
                Vide => {}
            }
        }
    }
    classeur.save(chemin).unwrap();
}

const ENTETES_DECISION: [&str; 6] = [
    "Flux",
    "Nom actuel",
    "Nouveau nom",
    "Occurrences",
    "Validation",
    "Fichier modèle",
];

// Dossier de travail temporaire, supprimé en fin de test.
struct Espace(PathBuf);

impl Drop for Espace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl Espace {
    fn vide(nom: &str) -> Espace {
        static COMPTEUR: AtomicUsize = AtomicUsize::new(0);
        let numero = COMPTEUR.fetch_add(1, Ordering::SeqCst);
        let dossier =
            std::env::temp_dir().join(format!("bpmn-cli-{}-{numero}-{nom}", std::process::id()));
        let _ = fs::remove_dir_all(&dossier);
        fs::create_dir_all(dossier.join("input")).unwrap();
        fs::create_dir_all(dossier.join("work")).unwrap();
        Espace(dossier)
    }

    // SGX standard dans input/export.sgx ; output/ n'est pas créé.
    fn standard(nom: &str) -> Espace {
        let espace = Espace::vide(nom);
        ecrire_zip(&espace.chemin("input/export.sgx"), &entrees_standard());
        espace
    }

    fn avec_entrees(nom: &str, entrees: &[(String, Vec<u8>)]) -> Espace {
        let espace = Espace::vide(nom);
        ecrire_zip(&espace.chemin("input/export.sgx"), entrees);
        espace
    }

    fn chemin(&self, relatif: &str) -> PathBuf {
        self.0.join(relatif)
    }

    fn correspondances(&self, lignes: &[(Cellule, Cellule)]) {
        let mut contenu = vec![vec![Texte("Nom actuel"), Texte("Nouveau nom")]];
        contenu.extend(
            lignes
                .iter()
                .map(|(ancien, nouveau)| vec![ancien.clone(), nouveau.clone()]),
        );
        ecrire_xlsx(
            &self.chemin("work/correspondance_swimlanes.xlsx"),
            "Correspondance",
            &contenu,
        );
    }

    fn validation(&self, lignes: &[[Cellule; 6]]) {
        let mut contenu = vec![ENTETES_DECISION.map(Texte).to_vec()];
        contenu.extend(lignes.iter().map(|ligne| ligne.to_vec()));
        ecrire_xlsx(
            &self.chemin("work/validation_modifications.xlsx"),
            "Analyse",
            &contenu,
        );
    }

    // Exécute le binaire et vérifie que la source n'a pas changé (R01).
    fn lancer(&self) -> Output {
        let avant = lire_dossier(&self.chemin("input"));
        let sortie = Command::new(env!("CARGO_BIN_EXE_bpmn-script-rs"))
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert_eq!(
            avant,
            lire_dossier(&self.chemin("input")),
            "la source a changé"
        );
        sortie
    }

    fn json(&self, relatif: &str) -> Value {
        serde_json::from_str(&fs::read_to_string(self.chemin(relatif)).unwrap()).unwrap()
    }

    fn existe(&self, relatif: &str) -> bool {
        self.chemin(relatif).exists()
    }

    fn fichiers_sortie(&self) -> Vec<String> {
        let mut noms: Vec<String> = fs::read_dir(self.chemin("output"))
            .map(|entrees| {
                entrees
                    .map(|entree| entree.unwrap().file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        noms.sort();
        noms
    }
}

fn lire_dossier(dossier: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut fichiers: Vec<_> = fs::read_dir(dossier)
        .unwrap()
        .map(|entree| {
            let chemin = entree.unwrap().path();
            let contenu = fs::read(&chemin).unwrap();
            (chemin, contenu)
        })
        .collect();
    fichiers.sort();
    fichiers
}

fn lire_zip(chemin: &Path) -> Vec<(String, Vec<u8>)> {
    let mut archive = ZipArchive::new(File::open(chemin).unwrap()).unwrap();
    (0..archive.len())
        .map(|index| {
            let mut entree = archive.by_index(index).unwrap();
            let mut contenu = Vec::new();
            entree.read_to_end(&mut contenu).unwrap();
            (entree.name().to_string(), contenu)
        })
        .collect()
}

fn lanes(forme: &Value, noms: &mut Vec<Value>) {
    if forme["stencil"]["id"] == "Lane" {
        noms.push(forme["properties"]["name"].clone());
    }
    for enfant in forme["childShapes"].as_array().into_iter().flatten() {
        lanes(enfant, noms);
    }
}

// Noms de toutes les lanes (y compris vides) d'un modèle du SGX produit.
fn lanes_produites(espace: &Espace, modele: &str) -> Vec<Value> {
    let entrees = lire_zip(&espace.chemin(SORTIE_SGX));
    let (_, contenu) = entrees.iter().find(|(nom, _)| nom == modele).unwrap();
    let mut noms = Vec::new();
    lanes(&serde_json::from_slice(contenu).unwrap(), &mut noms);
    noms
}

// Vérifie l'ordre des entrées et l'identité des entrées non modifiées (R11).
fn verifier_preservation(espace: &Espace, modifiees: &[&str]) {
    let source = lire_zip(&espace.chemin("input/export.sgx"));
    let produit = lire_zip(&espace.chemin(SORTIE_SGX));
    let noms = |entrees: &[(String, Vec<u8>)]| {
        entrees
            .iter()
            .map(|(nom, _)| nom.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(noms(&source), noms(&produit));
    for ((nom, avant), (_, apres)) in source.iter().zip(&produit) {
        if modifiees.contains(&nom.as_str()) {
            assert_ne!(avant, apres, "{nom} devait être modifié");
        } else {
            assert_eq!(avant, apres, "{nom} devait être préservé");
        }
    }
}

fn colonne_controle(espace: &Espace, colonne: usize) -> Vec<String> {
    let mut classeur: Xlsx<_> =
        open_workbook(espace.chemin("output/controle_validation.xlsx")).unwrap();
    let plage = classeur.worksheet_range("Contrôle").unwrap();
    plage
        .rows()
        .skip(1)
        .map(|ligne| ligne[colonne].to_string())
        .collect()
}

fn resultats_controle(espace: &Espace) -> Vec<String> {
    colonne_controle(espace, 7)
}

fn stdout(sortie: &Output) -> String {
    String::from_utf8_lossy(&sortie.stdout).into_owned()
}

fn stderr(sortie: &Output) -> String {
    String::from_utf8_lossy(&sortie.stderr).into_owned()
}

fn derniere_ligne(sortie: &Output) -> String {
    stdout(sortie)
        .lines()
        .last()
        .unwrap_or_default()
        .to_string()
}

// Correspondance standard A -> Z.
fn correspondance_a_z(espace: &Espace) {
    espace.correspondances(&[(Texte("A"), Texte("Z"))]);
}

// Lignes de validation conformes au dry-run A -> Z.
fn ligne_a(validation: Cellule) -> [Cellule; 6] {
    [
        Texte(FLUX),
        Texte("A"),
        Texte("Z"),
        Nombre(2.0),
        validation,
        Texte(MODELE_A),
    ]
}

fn ligne_b(validation: Cellule) -> [Cellule; 6] {
    [
        Texte(FLUX),
        Texte("A"),
        Texte("Z"),
        Nombre(1.0),
        validation,
        Texte(MODELE_B),
    ]
}

fn entrees_avec_modele_a(modele: Value) -> Vec<(String, Vec<u8>)> {
    let mut entrees = entrees_standard();
    entrees[0].1 = modele.to_string().into_bytes();
    entrees
}

// ---------------------------------------------------------------- Inventaire

#[test]
fn inventaire_seul_sans_correspondance() {
    let espace = Espace::standard("inventaire");
    let sortie = espace.lancer();
    assert_eq!(sortie.status.code(), Some(0));
    assert!(stdout(&sortie).contains("[INFO] Aucun fichier de correspondance trouvé"));
    assert_eq!(
        derniere_ligne(&sortie),
        "[RÉSULTAT] inventaire produit, correspondances attendues"
    );

    // Dossier output/ créé automatiquement (P05).
    let resultats = espace.json("output/resultats.json");
    let occurrences: Vec<_> = resultats
        .as_array()
        .unwrap()
        .iter()
        .map(|occurrence| {
            (
                occurrence["fichier_modele"].as_str().unwrap(),
                occurrence["lane"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        occurrences,
        [
            (MODELE_A, "A"),
            (MODELE_A, "A"),
            (MODELE_A, "B"),
            (MODELE_B, "A")
        ]
    );
    assert!(
        resultats
            .as_array()
            .unwrap()
            .iter()
            .all(|occurrence| occurrence["flux"] == FLUX)
    );

    let synthese = espace.json("output/synthese.json");
    assert_eq!(
        synthese["A"],
        json!({"occurrences": 3, "flux": [FLUX], "occurrences_par_flux": {FLUX: 3},
               "occurrences_par_modele": {
                   MODELE_A: {"flux": FLUX, "occurrences": 2},
                   MODELE_B: {"flux": FLUX, "occurrences": 1}}})
    );
    assert_eq!(synthese.as_object().unwrap().len(), 2);

    let mut classeur: Xlsx<_> =
        open_workbook(espace.chemin("output/inventaire_swimlanes.xlsx")).unwrap();
    let plage = classeur.worksheet_range("Correspondance").unwrap();
    let lignes: Vec<Vec<String>> = plage
        .rows()
        .map(|ligne| ligne.iter().map(Data::to_string).collect())
        .collect();
    assert_eq!(
        lignes[0],
        [
            "Nom actuel",
            "Nouveau nom",
            "Occurrences",
            "Nombre de flux",
            "Flux concernés",
            "Flux avec occurrences multiples"
        ]
    );
    // « Nombre de flux » compte les noms distincts, pas les modèles (R05).
    assert_eq!(lignes[1], ["A", "", "3", "1", FLUX, &format!("{FLUX} (3)")]);
    assert_eq!(lignes[2], ["B", "", "1", "1", FLUX, ""]);
    assert!(!espace.existe("output/analyse_modifications.json"));
}

fn verifier_anomalie(espace: &Espace, detail: &str) {
    let sortie = espace.lancer();
    assert_eq!(sortie.status.code(), Some(1));
    let erreur = stderr(&sortie);
    assert!(
        erreur.starts_with("[ERREUR] Le SGX contient 1 anomalie(s)"),
        "{erreur}"
    );
    assert!(erreur.contains(detail), "{erreur}");
    // Aucun résultat partiel (P03).
    assert!(espace.fichiers_sortie().is_empty());
}

#[test]
fn metadonnees_absentes_bloquent_le_traitement() {
    let entrees: Vec<_> = entrees_standard()
        .into_iter()
        .filter(|(nom, _)| !nom.ends_with("a/model_meta.json"))
        .collect();
    let espace = Espace::avec_entrees("sans-metadonnees", &entrees);
    verifier_anomalie(&espace, "dossier/a/model_meta.json : entrée illisible");
}

#[test]
fn nom_de_flux_non_textuel_bloque_le_traitement() {
    let mut entrees = entrees_standard();
    entrees[1].1 = br#"{"name": 7}"#.to_vec();
    let espace = Espace::avec_entrees("flux-non-textuel", &entrees);
    verifier_anomalie(&espace, "dossier/a/model_meta.json : name : texte attendu");
}

#[test]
fn json_de_modele_invalide_bloque_le_traitement() {
    let mut entrees = entrees_standard();
    entrees[0].1 = b"{\"childShapes\": [".to_vec();
    let espace = Espace::avec_entrees("json-invalide", &entrees);
    verifier_anomalie(&espace, "dossier/a/model_1_.json : JSON invalide");
}

#[test]
fn nom_de_lane_nul_bloque_le_traitement() {
    let mut modele = modele_a();
    modele["childShapes"][1]["properties"]["name"] = Value::Null;
    let espace = Espace::avec_entrees("nom-nul", &entrees_avec_modele_a(modele));
    verifier_anomalie(
        &espace,
        "dossier/a/model_1_.json : racine.childShapes[1].properties.name : texte attendu",
    );
}

#[test]
fn child_shapes_mal_type_bloque_le_traitement() {
    let espace = Espace::avec_entrees(
        "child-shapes",
        &entrees_avec_modele_a(json!({"childShapes": 7})),
    );
    verifier_anomalie(
        &espace,
        "dossier/a/model_1_.json : racine.childShapes : tableau attendu",
    );
}

#[test]
fn archive_sans_modele_refusee() {
    let espace = Espace::avec_entrees("sans-modele", &[("autre.json".into(), b"{}".to_vec())]);
    let sortie = espace.lancer();
    assert_eq!(sortie.status.code(), Some(1));
    assert!(stderr(&sortie).contains("Aucun modèle compatible trouvé"));
}

#[test]
fn modele_sans_lane_termine_normalement() {
    let espace = Espace::avec_entrees(
        "sans-lane",
        &entrees_avec_modele_a(json!({"childShapes": []})),
    );
    assert_eq!(espace.lancer().status.code(), Some(0));
    assert_eq!(espace.json("output/synthese.json")["A"]["occurrences"], 1);
}

// ---------------------------------------------------------------- Sources

#[test]
fn aucune_source() {
    let espace = Espace::vide("aucune-source");
    let sortie = espace.lancer();
    assert_eq!(sortie.status.code(), Some(1));
    assert_eq!(
        stderr(&sortie).trim(),
        "[ERREUR] Aucun fichier SGX trouvé dans input/"
    );
}

#[test]
fn plusieurs_sources() {
    let espace = Espace::standard("plusieurs-sources");
    fs::copy(
        espace.chemin("input/export.sgx"),
        espace.chemin("input/autre.SGX"),
    )
    .unwrap();
    let sortie = espace.lancer();
    assert_eq!(sortie.status.code(), Some(1));
    assert!(stderr(&sortie).contains("Plusieurs fichiers SGX"));
}

#[test]
fn source_invalide() {
    let espace = Espace::vide("source-invalide");
    fs::write(espace.chemin("input/export.sgx"), b"pas une archive").unwrap();
    let sortie = espace.lancer();
    assert_eq!(sortie.status.code(), Some(1));
    assert!(stderr(&sortie).starts_with("[ERREUR] Ce fichier n'est pas une archive SGX lisible"));
    assert!(!espace.existe("output"));
}

// ---------------------------------------------------------------- Correspondances et analyse

#[test]
fn analyse_sans_validation() {
    let espace = Espace::standard("analyse");
    espace.correspondances(&[
        (Texte("A"), Texte("Y")),
        (Texte("X"), Texte("Inconnu")),
        (Texte("B"), Vide),
        (Texte("B"), Texte("   ")),
        (Vide, Vide),
        (Texte("A"), Texte(" Z ")),
    ]);
    let sortie = espace.lancer();
    assert_eq!(sortie.status.code(), Some(0));
    let console = stdout(&sortie);
    assert!(console.contains("[INFO] Aucun fichier de validation trouvé"));
    assert!(
        console
            .contains("[INFO] Nom actuel renseigné plusieurs fois, dernière ligne retenue : « A »")
    );
    assert!(console.contains("[INFO] Nom actuel absent de l'inventaire, aucun impact : « X »"));
    assert_eq!(
        derniere_ligne(&sortie),
        "[RÉSULTAT] analyse produite, décisions attendues"
    );

    // Nouveau nom vide : pas de demande ; dernière correspondance retenue,
    // nouveau nom nettoyé, ancien nom inconnu conservé (R04).
    assert_eq!(
        espace.json("output/correspondances.json"),
        json!({"A": "Z", "X": "Inconnu"})
    );
    assert_eq!(
        espace.json("output/analyse_modifications.json"),
        json!([
            {"fichier_modele": MODELE_A, "flux": FLUX, "nom_actuel": "A", "nouveau_nom": "Z", "occurrences": 2},
            {"fichier_modele": MODELE_B, "flux": FLUX, "nom_actuel": "A", "nouveau_nom": "Z", "occurrences": 1},
        ])
    );
    assert!(espace.existe("output/analyse_modifications.xlsx"));
    assert!(!espace.existe("output/controle_validation.xlsx"));
    assert!(!espace.existe(SORTIE_SGX));
}

#[test]
fn analyse_vide_si_aucun_nom_connu() {
    // Conservé en V1 (P05) : Python échoue sur ce cas, Rust produit une analyse vide.
    let espace = Espace::standard("analyse-vide");
    espace.correspondances(&[(Texte("X"), Texte("Y"))]);
    espace.validation(&[ligne_a(Texte("OUI"))]);
    let sortie = espace.lancer();
    assert_eq!(sortie.status.code(), Some(0));
    assert_eq!(
        derniere_ligne(&sortie),
        "[RÉSULTAT] aucun changement proposé, vérifiez les nouveaux noms renseignés"
    );
    assert_eq!(espace.json("output/analyse_modifications.json"), json!([]));
    assert!(espace.existe("output/analyse_modifications.xlsx"));
    // Sans proposition, les décisions ne sont pas lues.
    assert!(!espace.existe("output/controle_validation.xlsx"));
}

fn verifier_refus_correspondances(espace: &Espace, detail: &str) {
    let sortie = espace.lancer();
    assert_eq!(sortie.status.code(), Some(1));
    let erreur = stderr(&sortie);
    assert!(erreur.contains(detail), "{erreur}");
    assert!(!espace.existe("output/correspondances.json"));
    assert!(!espace.existe("output/analyse_modifications.json"));
}

#[test]
fn ancien_nom_numerique_refuse() {
    let espace = Espace::standard("ancien-numerique");
    espace.correspondances(&[(Nombre(123.0), Texte("Z"))]);
    verifier_refus_correspondances(
        &espace,
        "Correspondance, ligne 2, colonne A (Nom actuel) : texte attendu (valeur lue : nombre 123)",
    );
}

#[test]
fn nouveau_nom_numerique_refuse() {
    let espace = Espace::standard("nouveau-numerique");
    espace.correspondances(&[(Texte("A"), Nombre(123.0))]);
    verifier_refus_correspondances(
        &espace,
        "Correspondance, ligne 2, colonne B (Nouveau nom) : texte attendu (valeur lue : nombre 123)",
    );
}

#[test]
fn formule_avec_ou_sans_valeur_calculee_refusee() {
    for (nom, valeur) in [("sans-cache", None), ("avec-cache", Some("Z"))] {
        let espace = Espace::standard(&format!("formule-{nom}"));
        espace.correspondances(&[(Texte("A"), Cellule::Formule("=\"Z\"", valeur))]);
        verifier_refus_correspondances(
            &espace,
            "Correspondance, ligne 2, colonne B (Nouveau nom) : formule refusée, saisir le texte directement (valeur lue : formule =\"Z\")",
        );
    }
}

#[test]
fn feuille_ou_entetes_incorrects_refuses() {
    let espace = Espace::standard("mauvaise-feuille");
    ecrire_xlsx(
        &espace.chemin("work/correspondance_swimlanes.xlsx"),
        "Feuil1",
        &[vec![Texte("Nom actuel"), Texte("Nouveau nom")]],
    );
    verifier_refus_correspondances(
        &espace,
        "feuille « Correspondance » absente (feuilles trouvées : Feuil1)",
    );

    let espace = Espace::standard("mauvais-entetes");
    ecrire_xlsx(
        &espace.chemin("work/correspondance_swimlanes.xlsx"),
        "Correspondance",
        &[
            vec![Texte("Nouveau nom"), Texte("Nom actuel")],
            vec![Texte("Z"), Texte("A")],
        ],
    );
    verifier_refus_correspondances(
        &espace,
        "Colonne A : « Nom actuel » attendu, « Nouveau nom » trouvé",
    );
}

#[test]
fn inventaire_complet_accepte_comme_correspondance() {
    // Le classeur d'inventaire, complété dans la colonne B, est un classeur de
    // correspondance valide : colonnes informatives ignorées.
    let espace = Espace::standard("inventaire-complete");
    espace.lancer();
    let mut classeur = Workbook::new();
    let feuille_excel = classeur.add_worksheet().set_name("Correspondance").unwrap();
    let mut inventaire: Xlsx<_> =
        open_workbook(espace.chemin("output/inventaire_swimlanes.xlsx")).unwrap();
    for (numero_ligne, ligne) in inventaire
        .worksheet_range("Correspondance")
        .unwrap()
        .rows()
        .enumerate()
    {
        for (numero_colonne, valeur) in ligne.iter().enumerate() {
            let (numero_ligne, numero_colonne) = (numero_ligne as u32, numero_colonne as u16);
            match valeur {
                Data::String(texte) => {
                    feuille_excel
                        .write_string(numero_ligne, numero_colonne, texte)
                        .unwrap();
                }
                Data::Float(nombre) => {
                    feuille_excel
                        .write_number(numero_ligne, numero_colonne, *nombre)
                        .unwrap();
                }
                _ => {}
            }
        }
    }
    feuille_excel.write_string(1, 1, "Z").unwrap();
    classeur
        .save(espace.chemin("work/correspondance_swimlanes.xlsx"))
        .unwrap();
    assert_eq!(espace.lancer().status.code(), Some(0));
    assert_eq!(
        espace.json("output/correspondances.json"),
        json!({"A": "Z"})
    );
}

// ---------------------------------------------------------------- Décisions

#[test]
fn toutes_decisions_oui_produisent_le_sgx() {
    let espace = Espace::standard("oui");
    correspondance_a_z(&espace);
    espace.validation(&[ligne_a(Texte("OUI")), ligne_b(Texte("OUI"))]);
    let sortie = espace.lancer();
    assert_eq!(sortie.status.code(), Some(0));
    assert!(stdout(&sortie).contains("[OK] SGX modifié généré"));
    assert_eq!(
        derniere_ligne(&sortie),
        format!(
            "[RÉSULTAT] SGX produit et vérifié : {} (2 modèle(s), 3 occurrence(s))",
            // Séparateur de la plateforme dans le chemin affiché.
            Path::new("output").join("export_modifie.sgx").display()
        )
    );

    // La lane « A » est renommée partout ; nom d'origine avec espaces remplacé,
    // tâche homonyme, lane « B » et lanes vides inchangées.
    assert_eq!(
        lanes_produites(&espace, MODELE_A),
        [json!("Z"), json!("Z"), json!("B"), json!(""), json!("   ")]
    );
    assert_eq!(lanes_produites(&espace, MODELE_B), [json!("Z")]);
    verifier_preservation(&espace, &[MODELE_A, MODELE_B]);

    // Propriétés inconnues du modèle conservées (R11).
    let produit = lire_zip(&espace.chemin(SORTIE_SGX));
    let modele: Value = serde_json::from_slice(&produit[0].1).unwrap();
    assert_eq!(modele["inconnu"], json!({"garder": [1, 2.5, "x"]}));
    assert_eq!(
        modele["childShapes"][0]["childShapes"][1]["properties"]["name"],
        "A"
    );

    assert_eq!(
        espace
            .json("output/modifications_validees.json")
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(espace.json("output/modifications_ignorees.json"), json!([]));
    assert_eq!(resultats_controle(&espace), ["VALIDÉE", "VALIDÉE"]);
    // Aucun fichier temporaire laissé en place.
    assert_eq!(
        espace.fichiers_sortie(),
        [
            "analyse_modifications.json",
            "analyse_modifications.xlsx",
            "controle_validation.xlsx",
            "correspondances.json",
            "export_modifie.sgx",
            "inventaire_swimlanes.xlsx",
            "modifications_ignorees.json",
            "modifications_validees.json",
            "resultats.json",
            "synthese.json",
        ]
    );
}

#[test]
fn oui_non_et_attente_melanges() {
    let espace = Espace::standard("melange");
    correspondance_a_z(&espace);
    espace.validation(&[ligne_a(Texte("OUI")), ligne_b(Texte("NON"))]);
    assert_eq!(espace.lancer().status.code(), Some(0));
    assert_eq!(lanes_produites(&espace, MODELE_B), [json!("A")]);
    verifier_preservation(&espace, &[MODELE_A]);
    assert_eq!(resultats_controle(&espace), ["VALIDÉE", "NON VALIDÉE"]);

    for (nom, attente) in [
        ("vide", Vide),
        ("autre", Texte("peut-être")),
        ("oui-non", Texte("OUI/NON")),
        ("formule", Cellule::Formule("=\"OUI\"", Some("OUI"))),
    ] {
        let espace = Espace::standard(&format!("attente-{nom}"));
        correspondance_a_z(&espace);
        espace.validation(&[ligne_a(Texte("OUI")), ligne_b(attente)]);
        assert_eq!(espace.lancer().status.code(), Some(0), "{nom}");
        verifier_preservation(&espace, &[MODELE_A]);
        assert_eq!(
            resultats_controle(&espace),
            ["VALIDÉE", "EN ATTENTE"],
            "{nom}"
        );
    }
}

#[test]
fn oui_normalise() {
    let espace = Espace::standard("oui-normalise");
    correspondance_a_z(&espace);
    espace.validation(&[ligne_a(Texte("  oui ")), ligne_b(Texte("Oui"))]);
    assert_eq!(espace.lancer().status.code(), Some(0));
    verifier_preservation(&espace, &[MODELE_A, MODELE_B]);
    assert_eq!(colonne_controle(&espace, 6), ["OUI", "OUI"]);
}

#[test]
fn arbitrage_partiel_ligne_absente_non_autorisee() {
    let espace = Espace::standard("partiel");
    correspondance_a_z(&espace);
    espace.validation(&[ligne_b(Texte("OUI"))]);
    assert_eq!(espace.lancer().status.code(), Some(0));
    verifier_preservation(&espace, &[MODELE_B]);
    assert_eq!(resultats_controle(&espace), ["VALIDÉE"]);
}

#[test]
fn lignes_vides_ignorees() {
    let espace = Espace::standard("lignes-vides");
    correspondance_a_z(&espace);
    espace.validation(&[
        ligne_a(Texte("OUI")),
        [Vide, Vide, Vide, Vide, Vide, Vide],
        ligne_b(Texte("OUI")),
    ]);
    assert_eq!(espace.lancer().status.code(), Some(0));
    assert_eq!(resultats_controle(&espace), ["VALIDÉE", "VALIDÉE"]);
    // Le numéro de ligne Excel d'origine est conservé.
    assert_eq!(colonne_controle(&espace, 0), ["2", "4"]);
}

#[test]
fn tous_refuses_aucune_sortie() {
    let espace = Espace::standard("refus");
    correspondance_a_z(&espace);
    espace.validation(&[ligne_a(Texte("NON")), ligne_b(Texte("NON"))]);
    let sortie = espace.lancer();
    // Code 2 : décisions fournies mais aucun SGX produit (P04).
    assert_eq!(sortie.status.code(), Some(2));
    assert_eq!(
        derniere_ligne(&sortie),
        "[RÉSULTAT] aucune décision admissible : aucun nouveau SGX n'a été créé"
    );
    assert!(!espace.existe(SORTIE_SGX));
    assert_eq!(espace.json("output/modifications_validees.json"), json!([]));
    assert!(espace.existe("output/controle_validation.xlsx"));
}

#[test]
fn ancienne_sortie_jamais_ecrasee() {
    let espace = Espace::standard("ancienne-sortie");
    fs::create_dir_all(espace.chemin("output")).unwrap();
    fs::write(espace.chemin(SORTIE_SGX), b"ANCIENNE-SORTIE").unwrap();
    correspondance_a_z(&espace);

    espace.validation(&[ligne_a(Texte("NON")), ligne_b(Texte("NON"))]);
    assert_eq!(espace.lancer().status.code(), Some(2));
    assert_eq!(
        fs::read(espace.chemin(SORTIE_SGX)).unwrap(),
        b"ANCIENNE-SORTIE"
    );

    // Une nouvelle production prend un autre nom (P04).
    espace.validation(&[ligne_a(Texte("OUI")), ligne_b(Texte("OUI"))]);
    let sortie = espace.lancer();
    assert_eq!(sortie.status.code(), Some(0));
    let attendu = Path::new("output").join("export_modifie_2.sgx");
    assert!(derniere_ligne(&sortie).contains(&attendu.display().to_string()));
    assert_eq!(
        fs::read(espace.chemin(SORTIE_SGX)).unwrap(),
        b"ANCIENNE-SORTIE"
    );
    assert_eq!(
        lire_zip(&espace.chemin("output/export_modifie_2.sgx")).len(),
        6
    );
}

#[test]
fn alteration_de_chaque_champ_ignore_la_ligne() {
    // Chaque champ de la proposition modifié sur la ligne A : ligne ignorée,
    // la ligne B conforme reste appliquée (R07, R08).
    let alterations = [
        ("flux", 0, Texte("Autre flux")),
        ("nom-actuel", 1, Texte("B")),
        ("nouveau-nom", 2, Texte("Y")),
        ("occurrences", 3, Nombre(5.0)),
        ("chemin", 5, Texte("dossier/x/model_1_.json")),
    ];
    for (nom, colonne, valeur) in alterations {
        let espace = Espace::standard(&format!("alteration-{nom}"));
        correspondance_a_z(&espace);
        let mut ligne = ligne_a(Texte("OUI"));
        ligne[colonne] = valeur;
        espace.validation(&[ligne, ligne_b(Texte("OUI"))]);
        let sortie = espace.lancer();
        assert_eq!(sortie.status.code(), Some(0), "{nom}");
        assert!(
            stdout(&sortie).contains(
                "[ATTENTION] Ligne 2 validée mais non conforme, modification ignorée : Ne correspond plus au dry-run"
            ),
            "{nom}"
        );
        verifier_preservation(&espace, &[MODELE_B]);
        assert_eq!(resultats_controle(&espace), ["IGNORÉE", "VALIDÉE"], "{nom}");
        let ignorees = espace.json("output/modifications_ignorees.json");
        assert_eq!(ignorees.as_array().unwrap().len(), 1, "{nom}");
        assert_eq!(
            ignorees[0]["motif"], "Ne correspond plus au dry-run",
            "{nom}"
        );
    }
}

#[test]
fn champs_de_type_incorrect_ignorent_la_ligne() {
    for (nom, colonne, valeur, motif) in [
        (
            "occurrences-texte",
            3,
            Texte("2"),
            "Occurrences : nombre entier positif attendu (valeur lue : « 2 »)",
        ),
        (
            "nom-numerique",
            2,
            Nombre(123.0),
            "Nouveau nom : texte attendu (valeur lue : nombre 123)",
        ),
        (
            "formule",
            1,
            Cellule::Formule("=\"A\"", Some("A")),
            "Nom actuel : texte attendu (valeur lue : formule =\"A\")",
        ),
    ] {
        let espace = Espace::standard(nom);
        correspondance_a_z(&espace);
        let mut ligne = ligne_a(Texte("OUI"));
        ligne[colonne] = valeur;
        espace.validation(&[ligne, ligne_b(Texte("OUI"))]);
        assert_eq!(espace.lancer().status.code(), Some(0), "{nom}");
        verifier_preservation(&espace, &[MODELE_B]);
        assert_eq!(resultats_controle(&espace), ["IGNORÉE", "VALIDÉE"], "{nom}");
        assert_eq!(colonne_controle(&espace, 8)[0], motif, "{nom}");
    }
}

fn verifier_blocage(espace: &Espace, divergence: &str) {
    let sortie = espace.lancer();
    assert_eq!(sortie.status.code(), Some(2));
    let console = stdout(&sortie);
    assert!(console.contains(divergence), "{console}");
    assert_eq!(
        derniere_ligne(&sortie),
        "[RÉSULTAT] les modifications ne correspondent pas aux occurrences attendues : aucun nouveau SGX n'a été créé"
    );
    assert!(!espace.existe(SORTIE_SGX));
}

#[test]
fn renommages_en_chaine_divergents_bloquent_la_production() {
    let espace = Espace::standard("chaine");
    espace.correspondances(&[(Texte("A"), Texte("B")), (Texte("B"), Texte("C"))]);
    espace.validation(&[
        [
            Texte(FLUX),
            Texte("A"),
            Texte("B"),
            Nombre(2.0),
            Texte("OUI"),
            Texte(MODELE_A),
        ],
        [
            Texte(FLUX),
            Texte("A"),
            Texte("B"),
            Nombre(1.0),
            Texte("OUI"),
            Texte(MODELE_B),
        ],
        [
            Texte(FLUX),
            Texte("B"),
            Texte("C"),
            Nombre(1.0),
            Texte("OUI"),
            Texte(MODELE_A),
        ],
    ]);
    verifier_blocage(
        &espace,
        "[ATTENTION] dossier/a/model_1_.json : « B » -> « C » : 3 occurrence(s) trouvée(s), 1 attendue(s)",
    );
    assert_eq!(
        espace
            .json("output/modifications_validees.json")
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn doublon_de_validation_bloque_la_production() {
    let espace = Espace::standard("doublon-validation");
    correspondance_a_z(&espace);
    espace.validation(&[ligne_a(Texte("OUI")), ligne_a(Texte("OUI"))]);
    verifier_blocage(&espace, "0 occurrence(s) trouvée(s), 2 attendue(s)");
    // Doublons conservés dans les décisions admises (R09).
    assert_eq!(
        espace
            .json("output/modifications_validees.json")
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

// Évolution V1 après la revue du parcours : OUI et NON sur la même proposition
// bloquent la production (auparavant, le OUI était appliqué).
#[test]
fn decisions_contradictoires_bloquent_la_production() {
    let espace = Espace::standard("contradiction");
    correspondance_a_z(&espace);
    espace.validation(&[ligne_a(Texte("NON")), ligne_a(Texte("OUI"))]);
    let sortie = espace.lancer();
    assert_eq!(sortie.status.code(), Some(2));
    assert!(stdout(&sortie).contains(&format!(
        "[ERREUR] Ligne 3 (OUI) et ligne 2 (NON) : décisions contraires pour « A » → « Z » ({MODELE_A})"
    )));
    assert_eq!(
        derniere_ligne(&sortie),
        "[RÉSULTAT] décisions contradictoires (OUI et NON pour la même proposition) : aucun nouveau SGX n'a été créé"
    );
    assert!(!espace.chemin(SORTIE_SGX).exists());
}

#[test]
fn permutation_bloque_la_production() {
    let espace = Espace::standard("permutation");
    espace.correspondances(&[(Texte("A"), Texte("B")), (Texte("B"), Texte("A"))]);
    espace.validation(&[
        [
            Texte(FLUX),
            Texte("A"),
            Texte("B"),
            Nombre(2.0),
            Texte("OUI"),
            Texte(MODELE_A),
        ],
        [
            Texte(FLUX),
            Texte("B"),
            Texte("A"),
            Nombre(1.0),
            Texte("OUI"),
            Texte(MODELE_A),
        ],
    ]);
    // Application séquentielle : B -> A trouve 3 lanes au lieu de 1.
    verifier_blocage(&espace, "3 occurrence(s) trouvée(s), 1 attendue(s)");
}

#[test]
fn ancien_nom_identique_au_nouveau_produit_le_sgx() {
    let espace = Espace::standard("meme-nom");
    espace.correspondances(&[(Texte("A"), Texte("A"))]);
    espace.validation(&[
        [
            Texte(FLUX),
            Texte("A"),
            Texte("A"),
            Nombre(2.0),
            Texte("OUI"),
            Texte(MODELE_A),
        ],
        [
            Texte(FLUX),
            Texte("A"),
            Texte("A"),
            Nombre(1.0),
            Texte("OUI"),
            Texte(MODELE_B),
        ],
    ]);
    assert_eq!(espace.lancer().status.code(), Some(0));
    assert!(espace.existe(SORTIE_SGX));
    // Seul le nom « A » avec espaces change (réécrit sans espaces).
    assert_eq!(lanes_produites(&espace, MODELE_A)[0], json!("A"));
    assert_eq!(lanes_produites(&espace, MODELE_B), [json!("A")]);
}

// ---------------------------------------------------------------- Préservation (P06)

#[test]
fn nombres_json_reproduits_a_l_identique() {
    let texte_modele = r#"{"grand":123456789012345678901234567890,"decimal":1.50,"negatif":-0.000120,"childShapes":[{"stencil":{"id":"Lane"},"properties":{"name":"A"},"childShapes":[]}]}"#;
    let mut entrees = entrees_standard();
    entrees[0].1 = texte_modele.as_bytes().to_vec();
    let espace = Espace::avec_entrees("nombres", &entrees);
    espace.correspondances(&[(Texte("A"), Texte("Z"))]);
    espace.validation(&[[
        Texte(FLUX),
        Texte("A"),
        Texte("Z"),
        Nombre(1.0),
        Texte("OUI"),
        Texte(MODELE_A),
    ]]);
    assert_eq!(espace.lancer().status.code(), Some(0));
    let produit = lire_zip(&espace.chemin(SORTIE_SGX));
    assert_eq!(
        String::from_utf8(produit[0].1.clone()).unwrap(),
        texte_modele.replace(r#""name":"A""#, r#""name":"Z""#)
    );
}

#[test]
fn attributs_zip_conserves() {
    let espace = Espace::vide("attributs-zip");
    let date = DateTime::from_date_and_time(2020, 5, 17, 10, 30, 0).unwrap();
    let mut zip = ZipWriter::new(File::create(espace.chemin("input/export.sgx")).unwrap());
    for (nom, contenu) in entrees_standard() {
        let mut options = FullFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .last_modified_time(date);
        if nom == MODELE_A || nom == PIECE_JOINTE {
            options = options.with_file_comment("commentaire");
            options.add_extra_data(0xcafe, b"OK", false).unwrap();
        }
        zip.start_file(nom, options).unwrap();
        zip.write_all(&contenu).unwrap();
    }
    zip.set_comment("commentaire-archive").unwrap();
    zip.finish().unwrap();
    correspondance_a_z(&espace);
    espace.validation(&[ligne_a(Texte("OUI"))]);
    assert_eq!(espace.lancer().status.code(), Some(0));

    let mut produit = ZipArchive::new(File::open(espace.chemin(SORTIE_SGX)).unwrap()).unwrap();
    assert_eq!(produit.comment(), b"commentaire-archive");
    let extra_attendu: &[u8] = b"\xfe\xca\x02\x00OK";
    for nom in [MODELE_A, PIECE_JOINTE] {
        let entree = produit.by_name(nom).unwrap();
        assert_eq!(entree.comment(), "commentaire", "{nom}");
        assert_eq!(entree.last_modified(), Some(date), "{nom}");
        assert_eq!(entree.compression(), CompressionMethod::Deflated, "{nom}");
        let conserve = entree.extra_data().unwrap_or_default() == extra_attendu;
        // Entrée modifiée : champ supplémentaire conservé. Entrée recopiée sans
        // recompression : la bibliothèque zip ne le recopie pas (limite documentée).
        assert_eq!(conserve, nom == MODELE_A, "{nom}");
    }
}
