//! Lecture des Excel de saisie et écriture des Excel produits.
//! Aucune décision métier : les valeurs relues sont transmises à `regles`.

use std::path::Path;

use calamine::{Data, Reader, Xlsx, open_workbook};
use indexmap::IndexMap;
use rust_xlsxwriter::{DataValidation, Format, FormatAlign, FormatPattern, Workbook};
use serde_json::{Value, json};

use crate::Res;
use crate::regles::{Decision, Ligne, Modification, Synthese};

pub(crate) fn ecrire_inventaire(chemin: &Path, synthese: &IndexMap<String, Synthese>) -> Res<()> {
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
        chemin,
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

pub(crate) fn ecrire_analyse(chemin: &Path, analyse: &[Modification]) -> Res<()> {
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

pub(crate) fn ecrire_controle(chemin: &Path, rapport: &[Vec<Value>]) -> Res<()> {
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

/// Écrit une feuille unique : en-tête en gras, volet figé, filtre automatique,
/// colonnes à retour à la ligne, et éventuellement une colonne de saisie
/// surlignée (avec liste OUI/NON si demandé).
fn ecrire_excel(
    chemin: &Path,
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

/// Relit les couples (Nom actuel, Nouveau nom) de la feuille « Correspondance ».
pub(crate) fn lire_correspondances(chemin: &Path) -> Res<Vec<(Option<String>, Option<String>)>> {
    Ok(lire_feuille(chemin, "Correspondance", 2)?
        .iter()
        .map(|ligne| (texte(&ligne[0]), texte(&ligne[1])))
        .collect())
}

/// Relit les lignes de la feuille « Analyse » de l'Excel de validation.
pub(crate) fn lire_decisions(chemin: &Path) -> Res<Vec<Decision>> {
    Ok(lire_feuille(chemin, "Analyse", 6)?
        .iter()
        .map(|c| Decision {
            ligne: Ligne {
                flux: texte(&c[0]),
                nom_actuel: texte(&c[1]),
                nouveau_nom: texte(&c[2]),
                occurrences: entier(&c[3]),
                fichier_modele: texte(&c[5]),
            },
            validation: texte(&c[4]),
        })
        .collect())
}

/// Lit les lignes de données (à partir de la ligne 2) d'une feuille.
fn lire_feuille(chemin: &Path, feuille: &str, colonnes: u32) -> Res<Vec<Vec<Data>>> {
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

// ---------------------------------------------------------------- Tests

/// Tests de caractérisation (lot M1), voir `regles.rs`.
#[cfg(test)]
mod tests {
    use super::*;

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
