//! Tests d'intégration de caractérisation (lot M1).
//!
//! Chaque test exécute le binaire dans un dossier temporaire contenant
//! `input/`, `work/` et éventuellement `output/`, avec un SGX et des Excel
//! synthétiques. Ils figent le comportement actuel, défauts compris : un test
//! marqué « caractérisation : défaut connu » n'est pas une attente V1.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use calamine::{Data, Reader, Xlsx, open_workbook};
use rust_xlsxwriter::Workbook;
use serde_json::{Value, json};
use zip::{CompressionMethod, ZipArchive, ZipWriter, write::SimpleFileOptions};

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

/// Modèle A : lane « A » imbriquée deux fois (dont une avec espaces), tâche
/// homonyme « A », lane « B » et lanes vides. Modèle B : une lane « A », même
/// nom de flux que A (modèles homonymes distingués par leur chemin).
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

fn meta() -> Value {
    json!({"name": FLUX})
}

fn entrees_standard() -> Vec<(String, Vec<u8>)> {
    vec![
        (MODELE_A.into(), modele_a().to_string().into_bytes()),
        (
            MODELE_A.replace("model_1_", "model_meta"),
            meta().to_string().into_bytes(),
        ),
        (MODELE_B.into(), modele_b().to_string().into_bytes()),
        (
            MODELE_B.replace("model_1_", "model_meta"),
            meta().to_string().into_bytes(),
        ),
        (PIECE_JOINTE.into(), b"\x00\xffconserver\r\n".to_vec()),
        (NON_CIBLE.into(), br#"{"untouched":true}"#.to_vec()),
    ]
}

fn ecrire_zip(chemin: &Path, entrees: &[(String, Vec<u8>)]) {
    let mut zip = ZipWriter::new(File::create(chemin).unwrap());
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (nom, contenu) in entrees {
        zip.start_file(nom.as_str(), options).unwrap();
        zip.write_all(contenu).unwrap();
    }
    zip.finish().unwrap();
}

/// Contenu d'une cellule d'un Excel de saisie.
#[derive(Clone)]
enum C {
    T(&'static str),
    N(f64),
    Vide,
}

fn ecrire_xlsx(chemin: &Path, feuille: &str, lignes: &[Vec<C>]) {
    let mut classeur = Workbook::new();
    let ws = classeur.add_worksheet().set_name(feuille).unwrap();
    for (r, ligne) in lignes.iter().enumerate() {
        for (c, cellule) in ligne.iter().enumerate() {
            let (r, c) = (r as u32, c as u16);
            match cellule {
                C::T(s) => ws.write_string(r, c, *s).map(|_| ()).unwrap(),
                C::N(n) => ws.write_number(r, c, *n).map(|_| ()).unwrap(),
                C::Vide => {}
            }
        }
    }
    classeur.save(chemin).unwrap();
}

/// Dossier de travail temporaire, supprimé en fin de test.
struct Espace(PathBuf);

impl Drop for Espace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl Espace {
    fn vide(nom: &str) -> Espace {
        static COMPTEUR: AtomicUsize = AtomicUsize::new(0);
        let n = COMPTEUR.fetch_add(1, Ordering::SeqCst);
        let dossier =
            std::env::temp_dir().join(format!("bpmn-m1-{}-{n}-{nom}", std::process::id()));
        let _ = fs::remove_dir_all(&dossier);
        fs::create_dir_all(dossier.join("input")).unwrap();
        fs::create_dir_all(dossier.join("work")).unwrap();
        Espace(dossier)
    }

    /// SGX standard dans `input/export.sgx` ; `output/` n'est pas créé.
    fn standard(nom: &str) -> Espace {
        let espace = Espace::vide(nom);
        ecrire_zip(&espace.p("input/export.sgx"), &entrees_standard());
        espace
    }

    fn p(&self, relatif: &str) -> PathBuf {
        self.0.join(relatif)
    }

    fn correspondances(&self, lignes: &[(C, C)]) {
        let mut rows = vec![vec![C::T("Nom actuel"), C::T("Nouveau nom")]];
        rows.extend(lignes.iter().map(|(a, n)| vec![a.clone(), n.clone()]));
        ecrire_xlsx(
            &self.p("work/correspondance_swimlanes.xlsx"),
            "Correspondance",
            &rows,
        );
    }

    fn validation(&self, lignes: &[[C; 6]]) {
        let mut rows = vec![
            [
                "Flux",
                "Nom actuel",
                "Nouveau nom",
                "Occurrences",
                "Validation",
                "Fichier modèle",
            ]
            .map(C::T)
            .to_vec(),
        ];
        rows.extend(lignes.iter().map(|l| l.to_vec()));
        ecrire_xlsx(
            &self.p("work/validation_modifications.xlsx"),
            "Analyse",
            &rows,
        );
    }

    /// Exécute le binaire et vérifie que la source n'a pas changé (R01).
    fn lancer(&self) -> Output {
        let avant = lire_dossier(&self.p("input"));
        let sortie = Command::new(env!("CARGO_BIN_EXE_bpmn-script-rs"))
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert_eq!(avant, lire_dossier(&self.p("input")), "la source a changé");
        sortie
    }

    fn json(&self, relatif: &str) -> Value {
        serde_json::from_str(&fs::read_to_string(self.p(relatif)).unwrap()).unwrap()
    }

    fn existe(&self, relatif: &str) -> bool {
        self.p(relatif).exists()
    }
}

fn lire_dossier(dossier: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut fichiers: Vec<_> = fs::read_dir(dossier)
        .unwrap()
        .map(|e| {
            let p = e.unwrap().path();
            let contenu = fs::read(&p).unwrap();
            (p, contenu)
        })
        .collect();
    fichiers.sort();
    fichiers
}

fn lire_zip(chemin: &Path) -> Vec<(String, Vec<u8>)> {
    let mut archive = ZipArchive::new(File::open(chemin).unwrap()).unwrap();
    (0..archive.len())
        .map(|i| {
            let mut entree = archive.by_index(i).unwrap();
            let mut contenu = Vec::new();
            entree.read_to_end(&mut contenu).unwrap();
            (entree.name().to_string(), contenu)
        })
        .collect()
}

fn lanes(shape: &Value, noms: &mut Vec<Value>) {
    if shape["stencil"]["id"] == "Lane" {
        noms.push(shape["properties"]["name"].clone());
    }
    for enfant in shape["childShapes"].as_array().into_iter().flatten() {
        lanes(enfant, noms);
    }
}

/// Noms de toutes les lanes (y compris vides) d'un modèle du SGX produit.
fn lanes_produites(espace: &Espace, modele: &str) -> Vec<Value> {
    let entrees = lire_zip(&espace.p(SORTIE_SGX));
    let (_, contenu) = entrees.iter().find(|(n, _)| n == modele).unwrap();
    let mut noms = Vec::new();
    lanes(&serde_json::from_slice(contenu).unwrap(), &mut noms);
    noms
}

/// Vérifie l'ordre des entrées et l'identité des entrées non modifiées (R11).
fn verifier_preservation(espace: &Espace, modifiees: &[&str]) {
    let source = lire_zip(&espace.p("input/export.sgx"));
    let produit = lire_zip(&espace.p(SORTIE_SGX));
    let noms = |v: &[(String, Vec<u8>)]| v.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>();
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
    let mut classeur: Xlsx<_> = open_workbook(espace.p("output/controle_validation.xlsx")).unwrap();
    let plage = classeur.worksheet_range("Contrôle").unwrap();
    plage
        .rows()
        .skip(1)
        .map(|l| l[colonne].to_string())
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

/// Correspondance standard A -> Z.
fn correspondance_a_z(espace: &Espace) {
    espace.correspondances(&[(C::T("A"), C::T("Z"))]);
}

/// Lignes de validation conformes au dry-run A -> Z.
fn ligne_a(validation: C) -> [C; 6] {
    [
        C::T(FLUX),
        C::T("A"),
        C::T("Z"),
        C::N(2.0),
        validation,
        C::T(MODELE_A),
    ]
}

fn ligne_b(validation: C) -> [C; 6] {
    [
        C::T(FLUX),
        C::T("A"),
        C::T("Z"),
        C::N(1.0),
        validation,
        C::T(MODELE_B),
    ]
}

// ---------------------------------------------------------------- Inventaire

#[test]
fn inventaire_seul_sans_correspondance() {
    let e = Espace::standard("inventaire");
    let sortie = e.lancer();
    assert_eq!(sortie.status.code(), Some(0));
    assert!(stdout(&sortie).contains("[INFO] Aucun fichier de correspondance trouvé"));

    // Dossier output/ créé automatiquement (P05).
    let resultats = e.json("output/resultats.json");
    let occurrences: Vec<_> = resultats
        .as_array()
        .unwrap()
        .iter()
        .map(|o| {
            (
                o["fichier_modele"].as_str().unwrap(),
                o["lane"].as_str().unwrap(),
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
            .all(|o| o["flux"] == FLUX)
    );

    let synthese = e.json("output/synthese.json");
    assert_eq!(
        synthese["A"],
        json!({"occurrences": 3, "flux": [FLUX], "occurrences_par_flux": {FLUX: 3},
               "occurrences_par_modele": {
                   MODELE_A: {"flux": FLUX, "occurrences": 2},
                   MODELE_B: {"flux": FLUX, "occurrences": 1}}})
    );
    assert_eq!(synthese.as_object().unwrap().len(), 2);

    let mut classeur: Xlsx<_> = open_workbook(e.p("output/inventaire_swimlanes.xlsx")).unwrap();
    let plage = classeur.worksheet_range("Correspondance").unwrap();
    let lignes: Vec<Vec<String>> = plage
        .rows()
        .map(|l| l.iter().map(Data::to_string).collect())
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
    assert!(!e.existe("output/analyse_modifications.json"));
}

#[test]
fn modele_sans_metadata_arrete_le_traitement() {
    let e = Espace::vide("sans-meta");
    let entrees: Vec<_> = entrees_standard()
        .into_iter()
        .filter(|(n, _)| !n.ends_with("a/model_meta.json"))
        .collect();
    ecrire_zip(&e.p("input/export.sgx"), &entrees);
    let sortie = e.lancer();
    assert_eq!(sortie.status.code(), Some(1));
    assert!(stderr(&sortie).starts_with("[ERREUR] "));
}

#[test]
fn nom_de_lane_nul_ignore_sans_diagnostic() {
    // caractérisation : défaut connu, voir P03 (inventaire partiel présenté comme complet).
    let e = Espace::vide("nom-nul");
    let mut entrees = entrees_standard();
    let mut a = modele_a();
    a["childShapes"][1]["properties"]["name"] = Value::Null;
    entrees[0].1 = a.to_string().into_bytes();
    ecrire_zip(&e.p("input/export.sgx"), &entrees);
    assert_eq!(e.lancer().status.code(), Some(0));
    let synthese = e.json("output/synthese.json");
    assert!(synthese.get("B").is_none());
    assert_eq!(synthese["A"]["occurrences"], 3);
}

#[test]
fn child_shapes_mal_type_ignore_sans_diagnostic() {
    // caractérisation : défaut connu, voir P03.
    let e = Espace::vide("child-shapes");
    let mut entrees = entrees_standard();
    entrees[0].1 = json!({"childShapes": 7}).to_string().into_bytes();
    ecrire_zip(&e.p("input/export.sgx"), &entrees);
    assert_eq!(e.lancer().status.code(), Some(0));
    let synthese = e.json("output/synthese.json");
    assert_eq!(synthese["A"]["occurrences"], 1);
    assert_eq!(
        synthese["A"]["occurrences_par_modele"]
            .as_object()
            .unwrap()
            .len(),
        1
    );
}

// ---------------------------------------------------------------- Sources

#[test]
fn aucune_source() {
    let e = Espace::vide("aucune-source");
    let sortie = e.lancer();
    assert_eq!(sortie.status.code(), Some(1));
    assert_eq!(
        stderr(&sortie).trim(),
        "[ERREUR] Aucun fichier SGX trouvé dans input/"
    );
}

#[test]
fn plusieurs_sources() {
    let e = Espace::standard("plusieurs-sources");
    fs::copy(e.p("input/export.sgx"), e.p("input/autre.SGX")).unwrap();
    let sortie = e.lancer();
    assert_eq!(sortie.status.code(), Some(1));
    assert!(stderr(&sortie).contains("Plusieurs fichiers SGX"));
}

#[test]
fn source_invalide() {
    let e = Espace::vide("source-invalide");
    fs::write(e.p("input/export.sgx"), b"pas une archive").unwrap();
    let sortie = e.lancer();
    assert_eq!(sortie.status.code(), Some(1));
    assert!(stderr(&sortie).starts_with("[ERREUR] "));
    assert!(!e.existe(SORTIE_SGX));
}

// ---------------------------------------------------------------- Correspondances et analyse

#[test]
fn analyse_sans_validation() {
    let e = Espace::standard("analyse");
    e.correspondances(&[
        (C::T("A"), C::T("Y")),
        (C::T("X"), C::T("Inconnu")),
        (C::T("B"), C::Vide),
        (C::T("B"), C::T("   ")),
        (C::T("A"), C::T(" Z ")),
    ]);
    let sortie = e.lancer();
    assert_eq!(sortie.status.code(), Some(0));
    assert!(stdout(&sortie).contains("[INFO] Aucun fichier de validation trouvé"));

    // Nouveau nom vide : pas de demande ; dernière correspondance retenue,
    // nouveau nom nettoyé, ancien nom inconnu conservé (R04).
    assert_eq!(
        e.json("output/correspondances.json"),
        json!({"A": "Z", "X": "Inconnu"})
    );
    assert_eq!(
        e.json("output/analyse_modifications.json"),
        json!([
            {"fichier_modele": MODELE_A, "flux": FLUX, "nom_actuel": "A", "nouveau_nom": "Z", "occurrences": 2},
            {"fichier_modele": MODELE_B, "flux": FLUX, "nom_actuel": "A", "nouveau_nom": "Z", "occurrences": 1},
        ])
    );
    assert!(e.existe("output/analyse_modifications.xlsx"));
    assert!(!e.existe("output/controle_validation.xlsx"));
    assert!(!e.existe(SORTIE_SGX));
}

#[test]
fn analyse_vide_si_aucun_nom_connu() {
    // Conservé en V1 (P05) : Python échoue sur ce cas, Rust produit une analyse vide.
    let e = Espace::standard("analyse-vide");
    e.correspondances(&[(C::T("X"), C::T("Y"))]);
    assert_eq!(e.lancer().status.code(), Some(0));
    assert_eq!(e.json("output/analyse_modifications.json"), json!([]));
    assert!(e.existe("output/analyse_modifications.xlsx"));
}

#[test]
fn ancien_nom_numerique_converti_en_texte() {
    // caractérisation : défaut connu, voir P02 (conversion silencieuse de 123 en « 123 »).
    let e = Espace::vide("ancien-numerique");
    let mut entrees = entrees_standard();
    let mut a = modele_a();
    a["childShapes"][1]["properties"]["name"] = json!("123");
    entrees[0].1 = a.to_string().into_bytes();
    ecrire_zip(&e.p("input/export.sgx"), &entrees);
    e.correspondances(&[(C::N(123.0), C::T("Z"))]);
    assert_eq!(e.lancer().status.code(), Some(0));
    assert_eq!(e.json("output/correspondances.json"), json!({"123": "Z"}));
    assert_eq!(
        e.json("output/analyse_modifications.json")
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn nouveau_nom_numerique_converti_en_texte() {
    // caractérisation : défaut connu, voir P02.
    let e = Espace::standard("nouveau-numerique");
    e.correspondances(&[(C::T("A"), C::N(123.0))]);
    assert_eq!(e.lancer().status.code(), Some(0));
    assert_eq!(e.json("output/correspondances.json"), json!({"A": "123"}));
}

// ---------------------------------------------------------------- Décisions

#[test]
fn toutes_decisions_oui_produisent_le_sgx() {
    let e = Espace::standard("oui");
    correspondance_a_z(&e);
    e.validation(&[ligne_a(C::T("OUI")), ligne_b(C::T("OUI"))]);
    let sortie = e.lancer();
    assert_eq!(sortie.status.code(), Some(0));
    assert!(stdout(&sortie).contains("[OK] SGX modifié généré"));

    // La lane « A » est renommée partout ; nom d'origine avec espaces remplacé,
    // tâche homonyme, lane « B » et lanes vides inchangées.
    assert_eq!(
        lanes_produites(&e, MODELE_A),
        [json!("Z"), json!("Z"), json!("B"), json!(""), json!("   ")]
    );
    assert_eq!(lanes_produites(&e, MODELE_B), [json!("Z")]);
    verifier_preservation(&e, &[MODELE_A, MODELE_B]);

    // Propriétés inconnues du modèle conservées (R11).
    let produit = lire_zip(&e.p(SORTIE_SGX));
    let a: Value = serde_json::from_slice(&produit[0].1).unwrap();
    assert_eq!(a["inconnu"], json!({"garder": [1, 2.5, "x"]}));
    assert_eq!(
        a["childShapes"][0]["childShapes"][1]["properties"]["name"],
        "A"
    );

    assert_eq!(
        e.json("output/modifications_validees.json")
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(e.json("output/modifications_ignorees.json"), json!([]));
    assert_eq!(resultats_controle(&e), ["VALIDÉE", "VALIDÉE"]);
}

#[test]
fn oui_non_et_attente_melanges() {
    let e = Espace::standard("melange");
    correspondance_a_z(&e);
    e.validation(&[ligne_a(C::T("OUI")), ligne_b(C::T("NON"))]);
    assert_eq!(e.lancer().status.code(), Some(0));
    assert_eq!(lanes_produites(&e, MODELE_B), [json!("A")]);
    verifier_preservation(&e, &[MODELE_A]);
    assert_eq!(resultats_controle(&e), ["VALIDÉE", "NON VALIDÉE"]);

    for (nom, attente) in [
        ("vide", C::Vide),
        ("autre", C::T("peut-être")),
        ("oui-non", C::T("OUI/NON")),
    ] {
        let e = Espace::standard(&format!("attente-{nom}"));
        correspondance_a_z(&e);
        e.validation(&[ligne_a(C::T("OUI")), ligne_b(attente)]);
        assert_eq!(e.lancer().status.code(), Some(0));
        verifier_preservation(&e, &[MODELE_A]);
        assert_eq!(resultats_controle(&e), ["VALIDÉE", "EN ATTENTE"]);
    }
}

#[test]
fn oui_normalise() {
    let e = Espace::standard("oui-normalise");
    correspondance_a_z(&e);
    e.validation(&[ligne_a(C::T("  oui ")), ligne_b(C::T("Oui"))]);
    assert_eq!(e.lancer().status.code(), Some(0));
    verifier_preservation(&e, &[MODELE_A, MODELE_B]);
    assert_eq!(colonne_controle(&e, 6), ["OUI", "OUI"]);
}

#[test]
fn arbitrage_partiel_ligne_absente_non_autorisee() {
    let e = Espace::standard("partiel");
    correspondance_a_z(&e);
    e.validation(&[ligne_b(C::T("OUI"))]);
    assert_eq!(e.lancer().status.code(), Some(0));
    verifier_preservation(&e, &[MODELE_B]);
    assert_eq!(resultats_controle(&e), ["VALIDÉE"]);
}

#[test]
fn tous_refuses_aucune_sortie() {
    let e = Espace::standard("refus");
    correspondance_a_z(&e);
    e.validation(&[ligne_a(C::T("NON")), ligne_b(C::T("NON"))]);
    let sortie = e.lancer();
    // caractérisation : défaut connu, voir P04 (code 0 sans SGX, résultat non explicite).
    assert_eq!(sortie.status.code(), Some(0));
    assert!(!e.existe(SORTIE_SGX));
    assert_eq!(e.json("output/modifications_validees.json"), json!([]));
    assert!(e.existe("output/controle_validation.xlsx"));
}

#[test]
fn ancienne_sortie_conservee_apres_refus() {
    // caractérisation : défaut connu, voir P04 (ancienne sortie non distinguée).
    let e = Espace::standard("ancienne-sortie");
    fs::create_dir_all(e.p("output")).unwrap();
    fs::write(e.p(SORTIE_SGX), b"ANCIENNE-SORTIE").unwrap();
    correspondance_a_z(&e);
    e.validation(&[ligne_a(C::T("NON")), ligne_b(C::T("NON"))]);
    assert_eq!(e.lancer().status.code(), Some(0));
    assert_eq!(fs::read(e.p(SORTIE_SGX)).unwrap(), b"ANCIENNE-SORTIE");
}

#[test]
fn alteration_de_chaque_champ_ignore_la_ligne() {
    // Chaque champ de la proposition modifié sur la ligne A : ligne ignorée,
    // la ligne B conforme reste appliquée (R07, R08).
    let alterations = [
        ("flux", 0, C::T("Autre flux")),
        ("nom-actuel", 1, C::T("B")),
        ("nouveau-nom", 2, C::T("Y")),
        ("occurrences", 3, C::N(5.0)),
        ("chemin", 5, C::T("dossier/x/model_1_.json")),
    ];
    for (nom, colonne, valeur) in alterations {
        let e = Espace::standard(&format!("alteration-{nom}"));
        correspondance_a_z(&e);
        let mut ligne = ligne_a(C::T("OUI"));
        ligne[colonne] = valeur;
        e.validation(&[ligne, ligne_b(C::T("OUI"))]);
        let sortie = e.lancer();
        assert_eq!(sortie.status.code(), Some(0), "{nom}");
        assert!(
            stdout(&sortie).contains("[ATTENTION] Une ligne validée ne correspond plus"),
            "{nom}"
        );
        verifier_preservation(&e, &[MODELE_B]);
        assert_eq!(resultats_controle(&e), ["IGNORÉE", "VALIDÉE"], "{nom}");
        let ignorees = e.json("output/modifications_ignorees.json");
        assert_eq!(ignorees.as_array().unwrap().len(), 1, "{nom}");
        assert_eq!(
            ignorees[0]["motif"], "Ne correspond plus au dry-run",
            "{nom}"
        );
    }
}

#[test]
fn occurrences_non_numeriques_ignorent_la_ligne() {
    let e = Espace::standard("occurrences-texte");
    correspondance_a_z(&e);
    let mut ligne = ligne_a(C::T("OUI"));
    ligne[3] = C::T("2");
    e.validation(&[ligne, ligne_b(C::T("OUI"))]);
    assert_eq!(e.lancer().status.code(), Some(0));
    verifier_preservation(&e, &[MODELE_B]);
    assert_eq!(resultats_controle(&e), ["IGNORÉE", "VALIDÉE"]);
}

#[test]
fn renommages_en_chaine_divergents_bloquent_la_production() {
    let e = Espace::standard("chaine");
    e.correspondances(&[(C::T("A"), C::T("B")), (C::T("B"), C::T("C"))]);
    e.validation(&[
        [
            C::T(FLUX),
            C::T("A"),
            C::T("B"),
            C::N(2.0),
            C::T("OUI"),
            C::T(MODELE_A),
        ],
        [
            C::T(FLUX),
            C::T("A"),
            C::T("B"),
            C::N(1.0),
            C::T("OUI"),
            C::T(MODELE_B),
        ],
        [
            C::T(FLUX),
            C::T("B"),
            C::T("C"),
            C::N(1.0),
            C::T("OUI"),
            C::T(MODELE_A),
        ],
    ]);
    let sortie = e.lancer();
    // caractérisation : défaut connu, voir P04 (code 0 sans SGX).
    assert_eq!(sortie.status.code(), Some(0));
    assert!(
        stdout(&sortie)
            .contains("[ATTENTION] Test en mémoire non conforme : 3 trouvée(s), 1 attendue(s)")
    );
    assert!(!e.existe(SORTIE_SGX));
    assert_eq!(
        e.json("output/modifications_validees.json")
            .as_array()
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn doublon_de_validation_bloque_la_production() {
    let e = Espace::standard("doublon-validation");
    correspondance_a_z(&e);
    e.validation(&[ligne_a(C::T("OUI")), ligne_a(C::T("OUI"))]);
    let sortie = e.lancer();
    assert_eq!(sortie.status.code(), Some(0));
    assert!(stdout(&sortie).contains("0 trouvée(s), 2 attendue(s)"));
    assert!(!e.existe(SORTIE_SGX));
    // Doublons conservés dans les décisions admises (R09).
    assert_eq!(
        e.json("output/modifications_validees.json")
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[test]
fn permutation_bloque_la_production() {
    let e = Espace::standard("permutation");
    e.correspondances(&[(C::T("A"), C::T("B")), (C::T("B"), C::T("A"))]);
    e.validation(&[
        [
            C::T(FLUX),
            C::T("A"),
            C::T("B"),
            C::N(2.0),
            C::T("OUI"),
            C::T(MODELE_A),
        ],
        [
            C::T(FLUX),
            C::T("B"),
            C::T("A"),
            C::N(1.0),
            C::T("OUI"),
            C::T(MODELE_A),
        ],
    ]);
    assert_eq!(e.lancer().status.code(), Some(0));
    // Application séquentielle : B -> A trouve 3 lanes au lieu de 1.
    assert!(!e.existe(SORTIE_SGX));
}

#[test]
fn ancien_nom_identique_au_nouveau_produit_le_sgx() {
    let e = Espace::standard("meme-nom");
    e.correspondances(&[(C::T("A"), C::T("A"))]);
    e.validation(&[
        [
            C::T(FLUX),
            C::T("A"),
            C::T("A"),
            C::N(2.0),
            C::T("OUI"),
            C::T(MODELE_A),
        ],
        [
            C::T(FLUX),
            C::T("A"),
            C::T("A"),
            C::N(1.0),
            C::T("OUI"),
            C::T(MODELE_B),
        ],
    ]);
    assert_eq!(e.lancer().status.code(), Some(0));
    assert!(e.existe(SORTIE_SGX));
    // Seul le nom « A » avec espaces change (réécrit sans espaces).
    assert_eq!(lanes_produites(&e, MODELE_A)[0], json!("A"));
    assert_eq!(lanes_produites(&e, MODELE_B), [json!("A")]);
}
