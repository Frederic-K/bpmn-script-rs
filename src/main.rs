//! bpmn-script-rs — portage Rust de la V0.1 de bpmn-script.
//!
//! Même workflow, mêmes dossiers, mêmes fichiers produits :
//! inventaire -> correspondance humaine -> dry-run -> validation humaine -> SGX modifié.

use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use calamine::{Data, Reader, Xlsx, open_workbook};
use indexmap::{IndexMap, map::Entry};
use rust_xlsxwriter::{DataValidation, Format, FormatAlign, FormatPattern, Workbook};
use serde::Serialize;
use serde_json::{Value, json};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

type Res<T> = Result<T, Box<dyn Error>>;

const CORRESPONDANCE: &str = "work/correspondance_swimlanes.xlsx";
const VALIDATION: &str = "work/validation_modifications.xlsx";

#[derive(Serialize)]
struct Occurrence {
    fichier_modele: String,
    flux: String,
    lane: String,
}

#[derive(Serialize, Default)]
struct Synthese {
    occurrences: u64,
    flux: Vec<String>,
    occurrences_par_flux: IndexMap<String, u64>,
    occurrences_par_modele: IndexMap<String, ParModele>,
}

#[derive(Serialize)]
struct ParModele {
    flux: String,
    occurrences: u64,
}

/// Une proposition du dry-run (toutes les valeurs sont connues).
#[derive(Serialize, Clone, PartialEq)]
struct Modification {
    fichier_modele: String,
    flux: String,
    nom_actuel: String,
    nouveau_nom: String,
    occurrences: u64,
}

/// Une ligne relue dans l'Excel de validation (les cellules peuvent être vides).
#[derive(Serialize)]
struct Ligne {
    fichier_modele: Option<String>,
    flux: Option<String>,
    nom_actuel: Option<String>,
    nouveau_nom: Option<String>,
    occurrences: Option<u64>,
}

impl Ligne {
    fn proposition(&self) -> Option<Modification> {
        Some(Modification {
            fichier_modele: self.fichier_modele.clone()?,
            flux: self.flux.clone()?,
            nom_actuel: self.nom_actuel.clone()?,
            nouveau_nom: self.nouveau_nom.clone()?,
            occurrences: self.occurrences?,
        })
    }
}

#[derive(Serialize)]
struct Ignoree {
    #[serde(flatten)]
    ligne: Ligne,
    motif: &'static str,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("[ERREUR] {e}");
        std::process::exit(1);
    }
}

fn run() -> Res<()> {
    let sgx = trouver_sgx()?;
    println!("[OK] Fichier SGX sélectionné : {}", nom(&sgx));
    fs::create_dir_all("output")?;

    // 1. Inventaire
    let resultats = extraire_lanes(&sgx)?;
    println!("[OK] Extraction terminée : {} occurrences", resultats.len());
    ecrire_json("output/resultats.json", &resultats)?;
    println!("[OK] resultats.json généré");

    let synthese = synthetiser(&resultats);
    println!("[OK] Synthèse terminée : {} lanes uniques", synthese.len());
    ecrire_json("output/synthese.json", &synthese)?;
    println!("[OK] synthese.json généré");

    ecrire_inventaire(&synthese)?;
    println!("[OK] Excel généré : output/inventaire_swimlanes.xlsx");

    // 2. Dry-run
    if !Path::new(CORRESPONDANCE).exists() {
        println!(
            "[INFO] Aucun fichier de correspondance trouvé : la préparation des modifications est ignorée"
        );
        return Ok(());
    }
    let correspondances = lire_correspondances()?;
    println!("[OK] Correspondances chargées : {}", correspondances.len());
    ecrire_json("output/correspondances.json", &correspondances)?;
    println!("[OK] correspondances.json généré");

    let analyse = dry_run(&correspondances, &synthese);
    println!("[OK] Dry-run préparé : {} impacts flux/lane", analyse.len());
    ecrire_json("output/analyse_modifications.json", &analyse)?;
    println!("[OK] analyse_modifications.json généré");

    ecrire_analyse(&analyse)?;
    println!("[OK] Excel d'analyse généré : output/analyse_modifications.xlsx");

    // 3. Validation
    if !Path::new(VALIDATION).exists() {
        println!("[INFO] Aucun fichier de validation trouvé : aucune modification autorisée");
        return Ok(());
    }
    let validees = controler_validation(&analyse)?;

    // 4. Génération du SGX modifié
    generer_sgx(&sgx, &validees)
}

// ---------------------------------------------------------------- SGX

fn trouver_sgx() -> Res<PathBuf> {
    let mut fichiers: Vec<PathBuf> = fs::read_dir("input")
        .into_iter()
        .flatten()
        .flatten()
        .map(|entree| entree.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("sgx")))
        .collect();

    match fichiers.len() {
        0 => Err("Aucun fichier SGX trouvé dans input/".into()),
        1 => Ok(fichiers.remove(0)),
        _ => Err("Plusieurs fichiers SGX trouvés dans input/ : un seul fichier est attendu".into()),
    }
}

fn lire_json(archive: &mut ZipArchive<File>, chemin: &str) -> Res<Value> {
    let mut contenu = String::new();
    archive.by_name(chemin)?.read_to_string(&mut contenu)?;
    Ok(serde_json::from_str(&contenu)?)
}

fn extraire_lanes(sgx: &Path) -> Res<Vec<Occurrence>> {
    let mut archive = ZipArchive::new(File::open(sgx)?)?;
    println!("[OK] Archive SGX ouverte");

    let modeles: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.name_for_index(i).map(str::to_string))
        .filter(|n| n.ends_with("model_1_.json"))
        .collect();

    let mut resultats = Vec::new();
    for fichier_modele in modeles {
        let meta = lire_json(
            &mut archive,
            &fichier_modele.replace("model_1_.json", "model_meta.json"),
        )?;
        let flux = meta["name"].as_str().unwrap_or("").to_string();

        let mut lanes = Vec::new();
        trouver_lanes(&lire_json(&mut archive, &fichier_modele)?, &mut lanes);

        for lane in lanes {
            resultats.push(Occurrence {
                fichier_modele: fichier_modele.clone(),
                flux: flux.clone(),
                lane,
            });
        }
    }
    Ok(resultats)
}

fn est_lane(shape: &Value) -> bool {
    shape["stencil"]["id"] == "Lane"
}

fn trouver_lanes(shape: &Value, lanes: &mut Vec<String>) {
    if est_lane(shape)
        && let Some(nom) = shape["properties"]["name"].as_str().map(str::trim)
        && !nom.is_empty()
    {
        lanes.push(nom.to_string());
    }
    for enfant in shape["childShapes"].as_array().into_iter().flatten() {
        trouver_lanes(enfant, lanes);
    }
}

fn renommer_lanes(shape: &mut Value, nom_actuel: &str, nouveau_nom: &str) -> u64 {
    let mut n = 0;
    if est_lane(shape) && shape["properties"]["name"].as_str().map(str::trim) == Some(nom_actuel) {
        shape["properties"]["name"] = nouveau_nom.into();
        n += 1;
    }
    for enfant in shape["childShapes"].as_array_mut().into_iter().flatten() {
        n += renommer_lanes(enfant, nom_actuel, nouveau_nom);
    }
    n
}

fn generer_sgx(sgx: &Path, validees: &[Modification]) -> Res<()> {
    let mut archive = ZipArchive::new(File::open(sgx)?)?;

    // Test des renommages en mémoire
    let mut modeles: IndexMap<String, Value> = IndexMap::new();
    let mut conformes = true;

    for m in validees {
        let modele = match modeles.entry(m.fichier_modele.clone()) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(lire_json(&mut archive, &m.fichier_modele)?),
        };
        let n = renommer_lanes(modele, &m.nom_actuel, &m.nouveau_nom);
        if n == m.occurrences {
            println!("[OK] Test en mémoire conforme : {n} modification(s)");
        } else {
            conformes = false;
            println!(
                "[ATTENTION] Test en mémoire non conforme : {n} trouvée(s), {} attendue(s)",
                m.occurrences
            );
        }
    }

    if !conformes || validees.is_empty() {
        return Ok(());
    }

    // Écriture de la nouvelle archive : entrées inchangées copiées telles quelles
    let stem = sgx.file_stem().unwrap_or_default().to_string_lossy();
    let chemin = PathBuf::from("output").join(format!("{stem}_modifie.sgx"));
    let mut sortie = ZipWriter::new(File::create(&chemin)?);

    for i in 0..archive.len() {
        let entree = archive.by_index_raw(i)?;
        match modeles.get(entree.name()) {
            None => sortie.raw_copy_file(entree)?,
            Some(modele) => {
                let options = SimpleFileOptions::default()
                    .compression_method(entree.compression())
                    .last_modified_time(entree.last_modified().unwrap_or_default());
                sortie.start_file(entree.name(), options)?;
                sortie.write_all(serde_json::to_string(modele)?.as_bytes())?;
            }
        }
    }
    sortie.finish()?;

    println!("[OK] SGX modifié généré : {}", chemin.display());
    Ok(())
}

// ---------------------------------------------------------------- Traitements

fn synthetiser(resultats: &[Occurrence]) -> IndexMap<String, Synthese> {
    let mut synthese: IndexMap<String, Synthese> = IndexMap::new();
    for r in resultats {
        let s = synthese.entry(r.lane.clone()).or_default();
        s.occurrences += 1;
        if !s.flux.contains(&r.flux) {
            s.flux.push(r.flux.clone());
        }
        *s.occurrences_par_flux.entry(r.flux.clone()).or_default() += 1;
        s.occurrences_par_modele
            .entry(r.fichier_modele.clone())
            .or_insert(ParModele {
                flux: r.flux.clone(),
                occurrences: 0,
            })
            .occurrences += 1;
    }
    synthese
}

fn lire_correspondances() -> Res<IndexMap<String, String>> {
    let mut correspondances = IndexMap::new();
    for ligne in lire_feuille(CORRESPONDANCE, "Correspondance", 2)? {
        let (Some(actuel), Some(nouveau)) = (texte(&ligne[0]), texte(&ligne[1])) else {
            continue;
        };
        let nouveau = nouveau.trim();
        if !nouveau.is_empty() {
            correspondances.insert(actuel, nouveau.to_string());
        }
    }
    Ok(correspondances)
}

fn dry_run(
    correspondances: &IndexMap<String, String>,
    synthese: &IndexMap<String, Synthese>,
) -> Vec<Modification> {
    let mut analyse = Vec::new();
    for (nom_actuel, nouveau_nom) in correspondances {
        let Some(infos) = synthese.get(nom_actuel) else {
            continue;
        };
        for (fichier_modele, pm) in &infos.occurrences_par_modele {
            analyse.push(Modification {
                fichier_modele: fichier_modele.clone(),
                flux: pm.flux.clone(),
                nom_actuel: nom_actuel.clone(),
                nouveau_nom: nouveau_nom.clone(),
                occurrences: pm.occurrences,
            });
        }
    }
    analyse
}

/// Relit la validation humaine, produit les JSON et le rapport de contrôle,
/// et retourne les modifications validées conformes au dry-run.
fn controler_validation(analyse: &[Modification]) -> Res<Vec<Modification>> {
    let mut validees = Vec::new();
    let mut ignorees = Vec::new();
    let mut controle = Vec::new();

    for (i, c) in lire_feuille(VALIDATION, "Analyse", 6)?.iter().enumerate() {
        let ligne = Ligne {
            flux: texte(&c[0]),
            nom_actuel: texte(&c[1]),
            nouveau_nom: texte(&c[2]),
            occurrences: entier(&c[3]),
            fichier_modele: texte(&c[5]),
        };
        let validation = texte(&c[4]).map(|v| v.trim().to_uppercase());

        let attendu = analyse.iter().find(|m| {
            Some(&m.fichier_modele) == ligne.fichier_modele.as_ref()
                && Some(&m.flux) == ligne.flux.as_ref()
                && Some(&m.nom_actuel) == ligne.nom_actuel.as_ref()
                && Some(m.occurrences) == ligne.occurrences
        });

        let (resultat, motif) = match validation.as_deref() {
            Some("OUI") => match ligne.proposition().filter(|p| analyse.contains(p)) {
                Some(p) => {
                    validees.push(p);
                    ("VALIDÉE", "Conforme au dry-run")
                }
                None => {
                    println!(
                        "[ATTENTION] Une ligne validée ne correspond plus au dry-run : modification ignorée"
                    );
                    ("IGNORÉE", "Ne correspond plus au dry-run")
                }
            },
            Some("NON") => ("NON VALIDÉE", "Refus humain"),
            _ => ("EN ATTENTE", "Aucune validation renseignée"),
        };

        controle.push(vec![
            json!(i + 2),
            json!(ligne.flux),
            json!(ligne.nom_actuel),
            json!(ligne.nouveau_nom),
            json!(attendu.map(|m| &m.nouveau_nom)),
            json!(ligne.occurrences),
            json!(validation),
            json!(resultat),
            json!(motif),
            json!(ligne.fichier_modele),
        ]);
        if resultat == "IGNORÉE" {
            ignorees.push(Ignoree { ligne, motif });
        }
    }

    println!("[OK] Modifications validées : {}", validees.len());
    ecrire_json("output/modifications_validees.json", &validees)?;
    println!("[OK] modifications_validees.json généré");
    ecrire_json("output/modifications_ignorees.json", &ignorees)?;
    println!("[OK] modifications_ignorees.json généré");

    ecrire_excel(
        "output/controle_validation.xlsx",
        "Contrôle",
        &[
            ("Ligne validation", 18.0),
            ("Flux", 70.0),
            ("Nom actuel", 35.0),
            ("Nouveau nom saisi", 35.0),
            ("Nouveau nom attendu", 35.0),
            ("Occurrences", 15.0),
            ("Validation", 15.0),
            ("Résultat", 18.0),
            ("Motif", 40.0),
            ("Fichier modèle", 250.0),
        ],
        &controle,
        &[1, 8, 9],
        None,
    )?;
    println!("[OK] Rapport de contrôle généré : output/controle_validation.xlsx");

    Ok(validees)
}

// ---------------------------------------------------------------- Excel

fn ecrire_inventaire(synthese: &IndexMap<String, Synthese>) -> Res<()> {
    let mut lanes: Vec<_> = synthese.iter().collect();
    lanes.sort_by(|a, b| a.0.cmp(b.0));

    let lignes: Vec<Vec<Value>> = lanes
        .into_iter()
        .map(|(lane, infos)| {
            let multiples: Vec<String> = infos
                .occurrences_par_flux
                .iter()
                .filter(|(_, n)| **n > 1)
                .map(|(flux, n)| format!("{flux} ({n})"))
                .collect();
            vec![
                json!(lane),
                Value::Null,
                json!(infos.occurrences),
                json!(infos.flux.len()),
                json!(infos.flux.join(", ")),
                json!(multiples.join(", ")),
            ]
        })
        .collect();

    ecrire_excel(
        "output/inventaire_swimlanes.xlsx",
        "Correspondance",
        &[
            ("Nom actuel", 35.0),
            ("Nouveau nom", 35.0),
            ("Occurrences", 15.0),
            ("Nombre de flux", 18.0),
            ("Flux concernés", 90.0),
            ("Flux avec occurrences multiples", 90.0),
        ],
        &lignes,
        &[4, 5],
        Some((1, false)),
    )
}

fn ecrire_analyse(analyse: &[Modification]) -> Res<()> {
    let lignes: Vec<Vec<Value>> = analyse
        .iter()
        .map(|m| {
            vec![
                json!(m.flux),
                json!(m.nom_actuel),
                json!(m.nouveau_nom),
                json!(m.occurrences),
                Value::Null,
                json!(m.fichier_modele),
            ]
        })
        .collect();

    ecrire_excel(
        "output/analyse_modifications.xlsx",
        "Analyse",
        &[
            ("Flux", 70.0),
            ("Nom actuel", 35.0),
            ("Nouveau nom", 35.0),
            ("Occurrences", 15.0),
            ("Validation", 15.0),
            ("Fichier modèle", 250.0),
        ],
        &lignes,
        &[0, 5],
        Some((4, true)),
    )
}

/// Écrit une feuille unique : en-tête en gras, volet figé, filtre automatique,
/// colonnes à retour à la ligne, et éventuellement une colonne de saisie
/// surlignée (avec liste OUI/NON si demandé).
fn ecrire_excel(
    chemin: &str,
    feuille: &str,
    colonnes: &[(&str, f64)],
    lignes: &[Vec<Value>],
    colonnes_wrap: &[u16],
    saisie: Option<(u16, bool)>,
) -> Res<()> {
    let gras = Format::new().set_bold();
    let wrap = Format::new().set_text_wrap().set_align(FormatAlign::Top);
    let jaune = Format::new()
        .set_pattern(FormatPattern::Solid)
        .set_background_color(0xFFF2CC);

    let mut classeur = Workbook::new();
    let ws = classeur.add_worksheet().set_name(feuille)?;

    for (c, (titre, largeur)) in colonnes.iter().enumerate() {
        ws.write_string_with_format(0, c as u16, *titre, &gras)?;
        ws.set_column_width(c as u16, *largeur)?;
    }

    for (r, ligne) in lignes.iter().enumerate() {
        let r = r as u32 + 1;
        for (c, valeur) in ligne.iter().enumerate() {
            let c = c as u16;
            let format = if saisie.is_some_and(|(s, _)| s == c) {
                &jaune
            } else if colonnes_wrap.contains(&c) {
                &wrap
            } else {
                &Format::default()
            };
            match valeur {
                Value::Null => ws.write_blank(r, c, format)?,
                Value::Number(n) => {
                    ws.write_number_with_format(r, c, n.as_f64().unwrap_or_default(), format)?
                }
                Value::String(s) => ws.write_string_with_format(r, c, s, format)?,
                autre => ws.write_string_with_format(r, c, autre.to_string(), format)?,
            };
        }
    }

    let derniere = lignes.len() as u32;
    ws.set_freeze_panes(1, 0)?;
    ws.autofilter(0, 0, derniere, colonnes.len() as u16 - 1)?;

    if let Some((c, true)) = saisie
        && derniere > 0
    {
        let oui_non = DataValidation::new().allow_list_strings(&["OUI", "NON"])?;
        ws.add_data_validation(1, c, derniere, c, &oui_non)?;
    }

    classeur.save(chemin)?;
    Ok(())
}

/// Lit les lignes de données (à partir de la ligne 2) d'une feuille.
fn lire_feuille(chemin: &str, feuille: &str, colonnes: u32) -> Res<Vec<Vec<Data>>> {
    let mut classeur: Xlsx<_> = open_workbook(chemin)?;
    let plage = classeur.worksheet_range(feuille)?;
    let Some((fin, _)) = plage.end() else {
        return Ok(Vec::new());
    };

    Ok((1..=fin)
        .map(|r| {
            (0..colonnes)
                .map(|c| plage.get_value((r, c)).cloned().unwrap_or_default())
                .collect()
        })
        .collect())
}

fn texte(cellule: &Data) -> Option<String> {
    match cellule {
        Data::Empty => None,
        Data::String(s) => Some(s.clone()),
        autre => Some(autre.to_string()),
    }
}

fn entier(cellule: &Data) -> Option<u64> {
    match cellule {
        Data::Int(i) => u64::try_from(*i).ok(),
        Data::Float(f) if *f >= 0.0 && f.fract() == 0.0 => Some(*f as u64),
        _ => None,
    }
}

// ---------------------------------------------------------------- Utilitaires

fn ecrire_json<T: Serialize>(chemin: &str, valeur: &T) -> Res<()> {
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

// ---------------------------------------------------------------- Tests

/// Tests de caractérisation (lot M1) : ils figent le comportement actuel,
/// défauts compris. Un test marqué « caractérisation : défaut connu » décrit
/// un comportement à revoir en M3, pas une attente V1.
#[cfg(test)]
mod tests {
    use super::*;

    fn lane(nom: Value, enfants: Vec<Value>) -> Value {
        json!({"stencil": {"id": "Lane"}, "properties": {"name": nom}, "childShapes": enfants})
    }

    fn tache(nom: &str) -> Value {
        json!({"stencil": {"id": "Task"}, "properties": {"name": nom}, "childShapes": []})
    }

    fn lanes_de(modele: &Value) -> Vec<String> {
        let mut lanes = Vec::new();
        trouver_lanes(modele, &mut lanes);
        lanes
    }

    fn occ(fichier_modele: &str, flux: &str, lane: &str) -> Occurrence {
        Occurrence {
            fichier_modele: fichier_modele.into(),
            flux: flux.into(),
            lane: lane.into(),
        }
    }

    // --- Inventaire (R03)

    #[test]
    fn inventaire_lanes_imbriquees_nettoyees_et_doublons_conserves() {
        let modele = json!({"childShapes": [
            lane(json!(" A "), vec![lane(json!("A"), vec![]), tache("A")]),
            lane(json!("B"), vec![]),
            lane(json!(""), vec![]),
            lane(json!("   "), vec![]),
        ]});
        assert_eq!(lanes_de(&modele), ["A", "A", "B"]);
    }

    #[test]
    fn inventaire_espaces_internes_non_nettoyes() {
        let modele = json!({"childShapes": [lane(json!(" A  B\nC "), vec![])]});
        assert_eq!(lanes_de(&modele), ["A  B\nC"]);
    }

    #[test]
    fn inventaire_ignore_les_formes_non_lane_homonymes() {
        let modele = json!({"childShapes": [tache("A"), json!({"stencil": {"id": "Pool"},
            "properties": {"name": "A"}, "childShapes": [lane(json!("L"), vec![])]})]});
        assert_eq!(lanes_de(&modele), ["L"]);
    }

    #[test]
    fn inventaire_nom_non_textuel_ignore_silencieusement() {
        // caractérisation : défaut connu, voir P03 (anomalie non signalée).
        let modele = json!({"childShapes": [
            lane(Value::Null, vec![lane(json!("Enfant"), vec![])]),
            lane(json!(123), vec![]),
            json!({"stencil": {"id": "Lane"}, "childShapes": []}),
        ]});
        assert_eq!(lanes_de(&modele), ["Enfant"]);
    }

    #[test]
    fn inventaire_child_shapes_mal_type_ignore_silencieusement() {
        // caractérisation : défaut connu, voir P03 (enfants non parcourus sans diagnostic).
        assert!(lanes_de(&json!({"childShapes": 7})).is_empty());
        let modele = json!({"childShapes": [
            json!({"stencil": {"id": "Lane"}, "properties": {"name": "A"}, "childShapes": {"x": 1}}),
        ]});
        assert_eq!(lanes_de(&modele), ["A"]);
    }

    // --- Renommage (R09)

    #[test]
    fn renommage_compte_les_occurrences_imbriquees_apres_trim() {
        let mut modele = json!({"childShapes": [
            lane(json!(" A "), vec![lane(json!("A"), vec![]), tache("A")]),
            lane(json!("B"), vec![]),
        ]});
        assert_eq!(renommer_lanes(&mut modele, "A", "Z"), 2);
        assert_eq!(modele["childShapes"][0]["properties"]["name"], "Z");
        assert_eq!(
            modele["childShapes"][0]["childShapes"][0]["properties"]["name"],
            "Z"
        );
        assert_eq!(
            modele["childShapes"][0]["childShapes"][1]["properties"]["name"],
            "A"
        );
        assert_eq!(modele["childShapes"][1]["properties"]["name"], "B");
    }

    #[test]
    fn renommage_sans_correspondance_ne_modifie_rien() {
        let mut modele = json!({"childShapes": [lane(json!("A"), vec![])]});
        let avant = modele.clone();
        assert_eq!(renommer_lanes(&mut modele, "X", "Z"), 0);
        assert_eq!(modele, avant);
    }

    #[test]
    fn renommage_preserve_les_proprietes_inconnues() {
        let mut modele = json!({"inconnu": {"k": [1, 2]}, "childShapes": [
            {"stencil": {"id": "Lane"}, "resourceId": "r1",
             "properties": {"name": "A", "autre": true}, "childShapes": []}
        ]});
        renommer_lanes(&mut modele, "A", "Z");
        assert_eq!(
            modele,
            json!({"inconnu": {"k": [1, 2]}, "childShapes": [
                {"stencil": {"id": "Lane"}, "resourceId": "r1",
                 "properties": {"name": "Z", "autre": true}, "childShapes": []}
            ]})
        );
    }

    #[test]
    fn renommages_sequentiels_en_chaine() {
        let mut modele =
            json!({"childShapes": [lane(json!("A"), vec![]), lane(json!("B"), vec![])]});
        assert_eq!(renommer_lanes(&mut modele, "A", "B"), 1);
        // La deuxième opération voit le résultat de la première (R09).
        assert_eq!(renommer_lanes(&mut modele, "B", "C"), 2);
    }

    // --- Synthèse (R02, R05)

    #[test]
    fn synthese_distingue_les_modeles_homonymes_par_chemin() {
        let resultats = [
            occ("a/model_1_.json", "Flux", "A"),
            occ("a/model_1_.json", "Flux", "A"),
            occ("b/model_1_.json", "Flux", "A"),
            occ("a/model_1_.json", "Flux", "B"),
        ];
        let synthese = synthetiser(&resultats);
        assert_eq!(synthese.keys().collect::<Vec<_>>(), ["A", "B"]);
        let a = &synthese["A"];
        assert_eq!(a.occurrences, 3);
        let par_modele: Vec<_> = a
            .occurrences_par_modele
            .iter()
            .map(|(chemin, pm)| (chemin.as_str(), pm.flux.as_str(), pm.occurrences))
            .collect();
        assert_eq!(
            par_modele,
            [
                ("a/model_1_.json", "Flux", 2),
                ("b/model_1_.json", "Flux", 1)
            ]
        );
    }

    #[test]
    fn synthese_nombre_de_flux_compte_les_noms_distincts() {
        // Deux modèles distincts portant le même nom de flux : un seul flux compté (R05).
        let resultats = [
            occ("a/model_1_.json", "Flux", "A"),
            occ("b/model_1_.json", "Flux", "A"),
        ];
        let synthese = synthetiser(&resultats);
        assert_eq!(synthese["A"].flux, ["Flux"]);
        assert_eq!(synthese["A"].occurrences_par_flux["Flux"], 2);
        assert_eq!(synthese["A"].occurrences_par_modele.len(), 2);
    }

    // --- Dry-run (R04, R05)

    #[test]
    fn dry_run_une_proposition_par_modele_et_nom() {
        let synthese = synthetiser(&[
            occ("a/model_1_.json", "Flux", "A"),
            occ("a/model_1_.json", "Flux", "A"),
            occ("b/model_1_.json", "Flux", "A"),
            occ("a/model_1_.json", "Flux", "B"),
        ]);
        let correspondances: IndexMap<String, String> = [("X", "Y"), ("B", "C"), ("A", "Z")]
            .into_iter()
            .map(|(a, n)| (a.to_string(), n.to_string()))
            .collect();
        let analyse = dry_run(&correspondances, &synthese);
        let lignes: Vec<_> = analyse
            .iter()
            .map(|m| {
                (
                    m.fichier_modele.as_str(),
                    m.nom_actuel.as_str(),
                    m.nouveau_nom.as_str(),
                    m.occurrences,
                )
            })
            .collect();
        assert_eq!(
            lignes,
            [
                ("a/model_1_.json", "B", "C", 1),
                ("a/model_1_.json", "A", "Z", 2),
                ("b/model_1_.json", "A", "Z", 1),
            ]
        );
    }

    #[test]
    fn dry_run_vide_si_aucun_ancien_nom_connu() {
        let synthese = synthetiser(&[occ("a/model_1_.json", "Flux", "A")]);
        let correspondances = IndexMap::from([("X".to_string(), "Y".to_string())]);
        assert!(dry_run(&correspondances, &synthese).is_empty());
    }

    // --- Relecture des décisions (R07, P02)

    #[test]
    fn proposition_incomplete_si_un_champ_manque() {
        let complete = || Ligne {
            fichier_modele: Some("a".into()),
            flux: Some("f".into()),
            nom_actuel: Some("A".into()),
            nouveau_nom: Some("Z".into()),
            occurrences: Some(1),
        };
        assert!(complete().proposition().is_some());
        let variantes: [fn(&mut Ligne); 5] = [
            |l| l.fichier_modele = None,
            |l| l.flux = None,
            |l| l.nom_actuel = None,
            |l| l.nouveau_nom = None,
            |l| l.occurrences = None,
        ];
        for retirer in variantes {
            let mut ligne = complete();
            retirer(&mut ligne);
            assert!(ligne.proposition().is_none());
        }
    }

    #[test]
    fn texte_convertit_toute_cellule_non_vide() {
        // caractérisation : défaut connu, voir P02 (coercition silencieuse des types).
        assert_eq!(texte(&Data::Empty), None);
        assert_eq!(texte(&Data::String(" a ".into())).as_deref(), Some(" a "));
        assert_eq!(texte(&Data::Int(123)).as_deref(), Some("123"));
        assert_eq!(texte(&Data::Float(123.0)).as_deref(), Some("123"));
        assert_eq!(texte(&Data::Bool(true)).as_deref(), Some("true"));
    }

    #[test]
    fn entier_accepte_les_nombres_entiers_non_negatifs() {
        assert_eq!(entier(&Data::Int(2)), Some(2));
        assert_eq!(entier(&Data::Float(2.0)), Some(2));
        assert_eq!(entier(&Data::Float(2.5)), None);
        assert_eq!(entier(&Data::Float(-1.0)), None);
        assert_eq!(entier(&Data::Int(-1)), None);
        assert_eq!(entier(&Data::String("2".into())), None);
        assert_eq!(entier(&Data::Bool(true)), None);
        assert_eq!(entier(&Data::Empty), None);
    }
}
