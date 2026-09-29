// Lecture des Excel de saisie et écriture des Excel produits.
// La lecture restitue les cellules sans conversion ; les règles d'acceptation
// sont dans regles.rs.

use std::path::Path;

use calamine::{Data, Reader, Xlsx, open_workbook};
use indexmap::IndexMap;
use rust_xlsxwriter::{DataValidation, Format, FormatAlign, FormatPattern, Workbook};
use serde_json::{Value, json};

use crate::regles::{Modification, Synthese, repetitions_dans_un_modele};
use crate::{Contexte, Erreur, Resultat};

pub(crate) const COLONNES_CORRESPONDANCE: [&str; 2] = ["Nom actuel", "Nouveau nom"];
pub(crate) const COLONNES_DECISION: [&str; 6] = [
    "Flux",
    "Nom actuel",
    "Nouveau nom",
    "Occurrences",
    "Validation",
    "Fichier modèle",
];

// Contenu d'une cellule tel qu'il est enregistré dans le classeur.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Cellule {
    Vide,
    Texte(String),
    Nombre(f64),
    // Formule, avec ou sans valeur calculée : jamais évaluée ni acceptée.
    Formule(String),
    // Booléen, date, durée ou erreur Excel, avec sa valeur lisible.
    Autre(String),
}

impl Cellule {
    // Valeur lisible pour les diagnostics et le rapport de contrôle.
    pub(crate) fn valeur_lue(&self) -> String {
        match self {
            Cellule::Vide => "cellule vide".to_string(),
            Cellule::Texte(texte) => format!("« {texte} »"),
            Cellule::Nombre(nombre) => format!("nombre {nombre}"),
            Cellule::Formule(formule) => format!("formule {formule}"),
            Cellule::Autre(valeur) => valeur.clone(),
        }
    }

    // Valeur à recopier dans le rapport : texte brut, formule en texte (non exécutable).
    pub(crate) fn pour_rapport(&self) -> Value {
        match self {
            Cellule::Vide => Value::Null,
            Cellule::Texte(texte) => json!(texte),
            Cellule::Nombre(nombre) => json!(nombre),
            Cellule::Formule(formule) => json!(formule),
            Cellule::Autre(valeur) => json!(valeur),
        }
    }

    fn est_vide(&self) -> bool {
        matches!(self, Cellule::Vide) || matches!(self, Cellule::Texte(texte) if texte.is_empty())
    }
}

// Une ligne de données : numéro de ligne Excel (1 = en-tête) et cellules des colonnes attendues.
pub(crate) struct LigneLue {
    pub(crate) numero: u32,
    pub(crate) cellules: Vec<Cellule>,
}

// Lit une feuille dont la première ligne porte exactement les en-têtes attendus
// (espaces périphériques ignorés). Les colonnes suivantes sont ignorées, les
// lignes entièrement vides aussi.
pub(crate) fn lire_feuille(
    chemin: &Path,
    feuille: &str,
    entetes_attendus: &[&str],
) -> Resultat<Vec<LigneLue>> {
    let illisible = format!(
        "Impossible de lire ce classeur. Enregistrez-le, fermez-le puis réessayez : {}",
        chemin.display()
    );
    let mut classeur: Xlsx<_> = open_workbook(chemin).contexte("classeur_illisible", &illisible)?;

    let feuilles = classeur.sheet_names();
    if !feuilles.iter().any(|nom| nom == feuille) {
        return Err(Erreur::nouvelle(
            "classeur_format",
            format!(
                "Ce classeur ne correspond pas au format attendu : feuille « {feuille} » absente (feuilles trouvées : {}).",
                feuilles.join(", ")
            ),
        ));
    }
    let valeurs = classeur
        .worksheet_range(feuille)
        .contexte("classeur_illisible", &illisible)?;
    let formules = classeur
        .worksheet_formula(feuille)
        .contexte("classeur_illisible", &illisible)?;

    let cellule = |ligne: u32, colonne: u32| -> Cellule {
        if let Some(formule) = formules.get_value((ligne, colonne))
            && !formule.is_empty()
        {
            return Cellule::Formule(format!("={formule}"));
        }
        match valeurs.get_value((ligne, colonne)) {
            None | Some(Data::Empty) => Cellule::Vide,
            Some(Data::String(texte)) => Cellule::Texte(texte.clone()),
            Some(Data::Float(nombre)) => Cellule::Nombre(*nombre),
            Some(Data::Int(nombre)) => Cellule::Nombre(*nombre as f64),
            Some(Data::Bool(valeur)) => Cellule::Autre(format!("booléen {valeur}")),
            Some(Data::Error(erreur)) => Cellule::Autre(format!("erreur Excel {erreur}")),
            Some(autre) => Cellule::Autre(format!("date ou durée {autre}")),
        }
    };

    let ecarts: Vec<String> = entetes_attendus
        .iter()
        .enumerate()
        .filter_map(|(colonne, attendu)| {
            let lu = cellule(0, colonne as u32);
            let conforme = matches!(&lu, Cellule::Texte(texte) if texte.trim() == *attendu);
            (!conforme).then(|| {
                format!(
                    "Colonne {} : « {attendu} » attendu, {} trouvé",
                    lettre_colonne(colonne),
                    lu.valeur_lue()
                )
            })
        })
        .collect();
    if !ecarts.is_empty() {
        return Err(Erreur::nouvelle(
            "classeur_format",
            format!(
                "Ce classeur ne correspond pas au format attendu : en-têtes de la feuille « {feuille} » incorrects."
            ),
        )
        .avec_details(ecarts));
    }

    let derniere_ligne = valeurs
        .end()
        .map(|(ligne, _)| ligne)
        .max(formules.end().map(|(ligne, _)| ligne))
        .unwrap_or(0);

    Ok((1..=derniere_ligne)
        .map(|ligne| LigneLue {
            numero: ligne + 1,
            cellules: (0..entetes_attendus.len() as u32)
                .map(|colonne| cellule(ligne, colonne))
                .collect(),
        })
        .filter(|ligne| !ligne.cellules.iter().all(Cellule::est_vide))
        .collect())
}

pub(crate) fn lettre_colonne(index: usize) -> char {
    (b'A' + index as u8) as char
}

// ---------------------------------------------------------------- Écriture

pub(crate) fn ecrire_inventaire(
    chemin: &Path,
    synthese: &IndexMap<String, Synthese>,
) -> Resultat<()> {
    let mut lanes: Vec<_> = synthese.iter().collect();
    lanes.sort_by(|a, b| a.0.cmp(b.0));

    let lignes: Vec<Vec<Value>> = lanes
        .into_iter()
        .map(|(lane, infos)| {
            vec![
                json!(lane),
                Value::Null,
                json!(infos.occurrences),
                json!(infos.flux.len()),
                json!(infos.flux.join(", ")),
                // Une entrée par ligne dans la cellule.
                json!(repetitions_dans_un_modele(infos).join("\n")),
            ]
        })
        .collect();

    ecrire_excel(
        chemin,
        "Correspondance",
        &[
            ("Nom actuel", 35.0),
            ("Nouveau nom", 35.0),
            ("Occurrences", 15.0),
            ("Nombre de flux", 18.0),
            ("Flux concernés", 90.0),
            ("Répétitions dans un même modèle", 90.0),
        ],
        &lignes,
        &[4, 5],
        Some((1, false)),
    )
}

pub(crate) fn ecrire_analyse(chemin: &Path, analyse: &[Modification]) -> Resultat<()> {
    let lignes: Vec<Vec<Value>> = analyse
        .iter()
        .map(|modification| {
            vec![
                json!(modification.flux),
                json!(modification.nom_actuel),
                json!(modification.nouveau_nom),
                json!(modification.occurrences),
                Value::Null,
                json!(modification.fichier_modele),
            ]
        })
        .collect();

    ecrire_excel(
        chemin,
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

pub(crate) fn ecrire_controle(chemin: &Path, rapport: &[Vec<Value>]) -> Resultat<()> {
    ecrire_excel(
        chemin,
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
        rapport,
        &[1, 8, 9],
        None,
    )
}

// Écrit une feuille unique : en-tête en gras, volet figé, filtre automatique,
// colonnes à retour à la ligne, et éventuellement une colonne de saisie
// surlignée (avec liste OUI/NON si demandé).
fn ecrire_excel(
    chemin: &Path,
    feuille: &str,
    colonnes: &[(&str, f64)],
    lignes: &[Vec<Value>],
    colonnes_retour_ligne: &[u16],
    colonne_saisie: Option<(u16, bool)>,
) -> Resultat<()> {
    let format_entete = Format::new().set_bold();
    let format_retour_ligne = Format::new().set_text_wrap().set_align(FormatAlign::Top);
    let format_saisie = Format::new()
        .set_pattern(FormatPattern::Solid)
        .set_background_color(0xFFF2CC);

    let mut classeur = Workbook::new();
    let feuille_excel = classeur.add_worksheet().set_name(feuille)?;

    for (colonne, (titre, largeur)) in colonnes.iter().enumerate() {
        feuille_excel.write_string_with_format(0, colonne as u16, *titre, &format_entete)?;
        feuille_excel.set_column_width(colonne as u16, *largeur)?;
    }

    for (index_ligne, ligne) in lignes.iter().enumerate() {
        let numero_ligne = index_ligne as u32 + 1;
        for (colonne, valeur) in ligne.iter().enumerate() {
            let colonne = colonne as u16;
            let format = if colonne_saisie.is_some_and(|(saisie, _)| saisie == colonne) {
                &format_saisie
            } else if colonnes_retour_ligne.contains(&colonne) {
                &format_retour_ligne
            } else {
                &Format::default()
            };
            match valeur {
                Value::Null => feuille_excel.write_blank(numero_ligne, colonne, format)?,
                Value::Number(nombre) => feuille_excel.write_number_with_format(
                    numero_ligne,
                    colonne,
                    nombre.as_f64().unwrap_or_default(),
                    format,
                )?,
                // Écrit comme texte : une valeur commençant par « = » ne devient pas une formule.
                Value::String(texte) => {
                    feuille_excel.write_string_with_format(numero_ligne, colonne, texte, format)?
                }
                autre => feuille_excel.write_string_with_format(
                    numero_ligne,
                    colonne,
                    autre.to_string(),
                    format,
                )?,
            };
        }
    }

    let derniere_ligne = lignes.len() as u32;
    feuille_excel.set_freeze_panes(1, 0)?;
    feuille_excel.autofilter(0, 0, derniere_ligne, colonnes.len() as u16 - 1)?;

    if let Some((colonne, true)) = colonne_saisie
        && derniere_ligne > 0
    {
        let oui_non = DataValidation::new().allow_list_strings(&["OUI", "NON"])?;
        feuille_excel.add_data_validation(1, colonne, derniere_ligne, colonne, &oui_non)?;
    }

    classeur.save(chemin)?;
    Ok(())
}
