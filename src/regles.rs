// Règles métier en mémoire : inventaire des lanes, synthèse, correspondances,
// dry-run, contrôle des décisions, test des renommages et vérification du
// résultat. Aucun accès fichier.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::Journal;
use crate::excel::{COLONNES_DECISION, Cellule, LigneLue, lettre_colonne};

#[derive(Serialize)]
pub(crate) struct Occurrence {
    pub(crate) fichier_modele: String,
    pub(crate) flux: String,
    pub(crate) lane: String,
}

#[derive(Serialize, Default)]
pub(crate) struct Synthese {
    pub(crate) occurrences: u64,
    pub(crate) flux: Vec<String>,
    pub(crate) occurrences_par_flux: IndexMap<String, u64>,
    pub(crate) occurrences_par_modele: IndexMap<String, ParModele>,
}

#[derive(Serialize)]
pub(crate) struct ParModele {
    pub(crate) flux: String,
    pub(crate) occurrences: u64,
}

// Une proposition du dry-run (toutes les valeurs sont connues).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Modification {
    pub fichier_modele: String,
    pub flux: String,
    pub nom_actuel: String,
    pub nouveau_nom: String,
    pub occurrences: u64,
}

// Une ligne de décision relue. Un champ absent ou de type incorrect vaut None.
#[derive(Serialize)]
pub(crate) struct Ligne {
    pub(crate) fichier_modele: Option<String>,
    pub(crate) flux: Option<String>,
    pub(crate) nom_actuel: Option<String>,
    pub(crate) nouveau_nom: Option<String>,
    pub(crate) occurrences: Option<u64>,
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
pub(crate) struct Ignoree {
    #[serde(flatten)]
    ligne: Ligne,
    motif: String,
}

// Correspondances adoptées : ancien nom -> nouveau nom, et anciens noms saisis plusieurs fois.
pub(crate) struct Correspondances {
    pub(crate) retenues: IndexMap<String, String>,
    pub(crate) doublons: Vec<String>,
}

// Résultat du contrôle des décisions.
pub(crate) struct Controle {
    pub(crate) validees: Vec<Modification>,
    pub(crate) ignorees: Vec<Ignoree>,
    pub(crate) refusees: usize,
    pub(crate) en_attente: usize,
    pub(crate) propositions_sans_decision: usize,
    // Même proposition exacte à la fois OUI et NON : bloque toute production.
    pub(crate) contradictions: Vec<String>,
    // Lignes du rapport controle_validation.xlsx, dans l'ordre des colonnes.
    pub(crate) rapport: Vec<Vec<Value>>,
}

// ---------------------------------------------------------------- Lanes

fn est_lane(forme: &Value) -> bool {
    forme["stencil"]["id"] == "Lane"
}

// Nom d'une lane : absent = pas de nom ; présent mais non textuel = anomalie.
fn nom_de_lane(forme: &Value) -> Result<Option<&str>, &'static str> {
    match forme.get("properties") {
        None => Ok(None),
        Some(Value::Object(proprietes)) => match proprietes.get("name") {
            None => Ok(None),
            Some(Value::String(nom)) => Ok(Some(nom)),
            Some(_) => Err("properties.name : texte attendu"),
        },
        Some(_) => Err("properties : objet attendu"),
    }
}

// Ajoute à `lanes` les noms de lanes non vides (espaces périphériques retirés)
// de la forme et de ses enfants. Toute structure mal typée est ajoutée à
// `anomalies` avec son emplacement ; elle doit bloquer le traitement.
pub(crate) fn trouver_lanes(
    forme: &Value,
    emplacement: &str,
    lanes: &mut Vec<String>,
    anomalies: &mut Vec<String>,
) {
    let Some(objet) = forme.as_object() else {
        anomalies.push(format!("{emplacement} : objet attendu"));
        return;
    };
    if est_lane(forme) {
        match nom_de_lane(forme) {
            Ok(Some(nom)) if !nom.trim().is_empty() => lanes.push(nom.trim().to_string()),
            Ok(_) => {}
            Err(motif) => anomalies.push(format!("{emplacement}.{motif}")),
        }
    }
    match objet.get("childShapes") {
        None => {}
        Some(Value::Array(enfants)) => {
            for (index, enfant) in enfants.iter().enumerate() {
                let emplacement_enfant = format!("{emplacement}.childShapes[{index}]");
                trouver_lanes(enfant, &emplacement_enfant, lanes, anomalies);
            }
        }
        Some(_) => anomalies.push(format!("{emplacement}.childShapes : tableau attendu")),
    }
}

// Enfants d'une forme, sans créer de clé : l'indexation mutable de serde_json
// (forme["childShapes"]) insérerait "childShapes": null dans chaque forme sans
// enfant, ce qui altérerait le modèle réécrit.
fn enfants_modifiables(forme: &mut Value) -> impl Iterator<Item = &mut Value> {
    forme
        .get_mut("childShapes")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
}

fn renommer_lanes(forme: &mut Value, nom_actuel: &str, nouveau_nom: &str) -> u64 {
    let mut nombre = 0;
    if est_lane(forme) && forme["properties"]["name"].as_str().map(str::trim) == Some(nom_actuel) {
        forme["properties"]["name"] = nouveau_nom.into();
        nombre += 1;
    }
    for enfant in enfants_modifiables(forme) {
        nombre += renommer_lanes(enfant, nom_actuel, nouveau_nom);
    }
    nombre
}

// ---------------------------------------------------------------- Inventaire et dry-run

pub(crate) fn synthetiser(occurrences: &[Occurrence]) -> IndexMap<String, Synthese> {
    let mut synthese: IndexMap<String, Synthese> = IndexMap::new();
    for occurrence in occurrences {
        let synthese_lane = synthese.entry(occurrence.lane.clone()).or_default();
        synthese_lane.occurrences += 1;
        if !synthese_lane.flux.contains(&occurrence.flux) {
            synthese_lane.flux.push(occurrence.flux.clone());
        }
        *synthese_lane
            .occurrences_par_flux
            .entry(occurrence.flux.clone())
            .or_default() += 1;
        synthese_lane
            .occurrences_par_modele
            .entry(occurrence.fichier_modele.clone())
            .or_insert(ParModele {
                flux: occurrence.flux.clone(),
                occurrences: 0,
            })
            .occurrences += 1;
    }
    synthese
}

// Contrat d'import des correspondances (P02). Une ligne sans nouveau nom (vide
// ou espaces) n'est pas une demande. Sur une ligne avec demande, les deux noms
// doivent être du texte ; toute formule est refusée. Le nouveau nom est nettoyé
// des espaces périphériques, l'ancien nom est lu tel quel ; la dernière ligne
// l'emporte pour un même ancien nom. Retourne les diagnostics en cas de refus.
pub(crate) fn lire_correspondances(lignes: &[LigneLue]) -> Result<Correspondances, Vec<String>> {
    let mut correspondances = Correspondances {
        retenues: IndexMap::new(),
        doublons: Vec::new(),
    };
    let mut diagnostics = Vec::new();
    let diagnostic = |numero: u32, colonne: usize, attendu: &str, cellule: &Cellule| {
        format!(
            "Correspondance, ligne {numero}, colonne {} ({}) : {attendu} (valeur lue : {})",
            lettre_colonne(colonne),
            ["Nom actuel", "Nouveau nom"][colonne],
            cellule.valeur_lue()
        )
    };

    for ligne in lignes {
        let (ancien, nouveau) = (&ligne.cellules[0], &ligne.cellules[1]);
        let mut formule = false;
        for (colonne, cellule) in [ancien, nouveau].into_iter().enumerate() {
            if matches!(cellule, Cellule::Formule(_)) {
                formule = true;
                diagnostics.push(diagnostic(
                    ligne.numero,
                    colonne,
                    "formule refusée, saisir le texte directement",
                    cellule,
                ));
            }
        }
        if formule {
            continue;
        }
        let nouveau = match nouveau {
            Cellule::Vide => continue,
            Cellule::Texte(texte) if texte.trim().is_empty() => continue,
            Cellule::Texte(texte) => texte.trim(),
            autre => {
                diagnostics.push(diagnostic(ligne.numero, 1, "texte attendu", autre));
                continue;
            }
        };
        let Cellule::Texte(ancien) = ancien else {
            diagnostics.push(diagnostic(ligne.numero, 0, "texte attendu", ancien));
            continue;
        };
        if correspondances.retenues.contains_key(ancien)
            && !correspondances.doublons.contains(ancien)
        {
            correspondances.doublons.push(ancien.clone());
        }
        correspondances
            .retenues
            .insert(ancien.clone(), nouveau.to_string());
    }

    if diagnostics.is_empty() {
        Ok(correspondances)
    } else {
        Err(diagnostics)
    }
}

pub(crate) fn dry_run(
    correspondances: &IndexMap<String, String>,
    synthese: &IndexMap<String, Synthese>,
) -> Vec<Modification> {
    let mut analyse = Vec::new();
    for (nom_actuel, nouveau_nom) in correspondances {
        let Some(infos) = synthese.get(nom_actuel) else {
            continue;
        };
        for (fichier_modele, par_modele) in &infos.occurrences_par_modele {
            analyse.push(Modification {
                fichier_modele: fichier_modele.clone(),
                flux: par_modele.flux.clone(),
                nom_actuel: nom_actuel.clone(),
                nouveau_nom: nouveau_nom.clone(),
                occurrences: par_modele.occurrences,
            });
        }
    }
    analyse
}

// ---------------------------------------------------------------- Décisions

// Champ texte d'une décision : seul un texte saisi est accepté, sans conversion.
fn texte_strict(cellule: &Cellule) -> Result<String, &'static str> {
    match cellule {
        Cellule::Texte(texte) => Ok(texte.clone()),
        _ => Err("texte attendu"),
    }
}

// Occurrences : nombre entier positif ou nul (1.0 accepté), jamais un texte.
fn entier_strict(cellule: &Cellule) -> Result<u64, &'static str> {
    match cellule {
        Cellule::Nombre(nombre)
            if *nombre >= 0.0 && nombre.fract() == 0.0 && *nombre <= 9_007_199_254_740_992.0 =>
        {
            Ok(*nombre as u64)
        }
        _ => Err("nombre entier positif attendu"),
    }
}

// Compare les décisions relues au dry-run (R07, R08, P02). Seul un OUI dont les
// cinq champs sont valides et identiques à une proposition est admis. Un OUI
// non conforme est ignoré avec son motif sans bloquer les autres lignes.
// Une proposition qui reçoit à la fois OUI et NON est une contradiction : elle
// bloque la production, sans faire gagner l'une des deux lignes.
pub(crate) fn controler_decisions(
    analyse: &[Modification],
    lignes: &[LigneLue],
    journal: &mut Journal<'_>,
) -> Controle {
    let mut controle = Controle {
        validees: Vec::new(),
        ignorees: Vec::new(),
        refusees: 0,
        en_attente: 0,
        propositions_sans_decision: 0,
        contradictions: Vec::new(),
        rapport: Vec::new(),
    };
    let mut propositions_repondues = vec![false; analyse.len()];
    // Numéro de ligne et proposition de chaque OUI admis et de chaque NON.
    let mut lignes_oui = Vec::new();
    let mut lignes_non = Vec::new();

    for ligne_lue in lignes {
        let cellules = &ligne_lue.cellules;
        let champ = |colonne: usize| -> Result<String, String> {
            texte_strict(&cellules[colonne])
                .map_err(|attendu| motif_cellule(colonne, attendu, &cellules[colonne]))
        };
        let flux = champ(0);
        let nom_actuel = champ(1);
        let nouveau_nom = champ(2);
        let occurrences =
            entier_strict(&cellules[3]).map_err(|attendu| motif_cellule(3, attendu, &cellules[3]));
        let fichier_modele = champ(5);

        let ligne = Ligne {
            fichier_modele: fichier_modele.clone().ok(),
            flux: flux.clone().ok(),
            nom_actuel: nom_actuel.clone().ok(),
            nouveau_nom: nouveau_nom.clone().ok(),
            occurrences: occurrences.clone().ok(),
        };
        if let Some(proposition) = ligne.proposition() {
            for (index, attendue) in analyse.iter().enumerate() {
                if *attendue == proposition {
                    propositions_repondues[index] = true;
                }
            }
        }
        let attendu = analyse.iter().find(|modification| {
            Some(&modification.fichier_modele) == ligne.fichier_modele.as_ref()
                && Some(&modification.flux) == ligne.flux.as_ref()
                && Some(&modification.nom_actuel) == ligne.nom_actuel.as_ref()
                && Some(modification.occurrences) == ligne.occurrences
        });

        let validation = match &cellules[4] {
            Cellule::Vide => Ok(None),
            Cellule::Texte(texte) => Ok(Some(texte.trim().to_uppercase())),
            autre => Err(motif_cellule(4, "texte OUI ou NON attendu", autre)),
        };

        let (resultat, motif) = match &validation {
            Err(motif) => ("EN ATTENTE", motif.clone()),
            Ok(Some(valeur)) if valeur == "OUI" => {
                let premier_champ_invalide = [&fichier_modele, &flux, &nom_actuel, &nouveau_nom]
                    .into_iter()
                    .find_map(|champ| champ.clone().err())
                    .or(occurrences.clone().err());
                match (premier_champ_invalide, ligne.proposition()) {
                    (Some(motif), _) => ("IGNORÉE", motif),
                    (None, Some(proposition)) if analyse.contains(&proposition) => {
                        lignes_oui.push((ligne_lue.numero, proposition.clone()));
                        controle.validees.push(proposition);
                        ("VALIDÉE", "Conforme au dry-run".to_string())
                    }
                    _ => ("IGNORÉE", "Ne correspond plus au dry-run".to_string()),
                }
            }
            Ok(Some(valeur)) if valeur == "NON" => {
                if let Some(proposition) = ligne.proposition() {
                    lignes_non.push((ligne_lue.numero, proposition));
                }
                ("NON VALIDÉE", "Refus humain".to_string())
            }
            Ok(Some(valeur)) if !valeur.is_empty() => (
                "EN ATTENTE",
                format!("Validation « {valeur} » non reconnue : OUI ou NON attendu"),
            ),
            Ok(_) => ("EN ATTENTE", "Aucune validation renseignée".to_string()),
        };
        if resultat == "IGNORÉE" {
            journal(&format!(
                "[ATTENTION] Ligne {} validée mais non conforme, modification ignorée : {motif}",
                ligne_lue.numero
            ));
        }

        controle.rapport.push(vec![
            json!(ligne_lue.numero),
            cellules[0].pour_rapport(),
            cellules[1].pour_rapport(),
            cellules[2].pour_rapport(),
            json!(attendu.map(|modification| &modification.nouveau_nom)),
            cellules[3].pour_rapport(),
            match &validation {
                Ok(valeur) => json!(valeur),
                Err(_) => cellules[4].pour_rapport(),
            },
            json!(resultat),
            json!(motif),
            cellules[5].pour_rapport(),
        ]);
        match resultat {
            "IGNORÉE" => controle.ignorees.push(Ignoree { ligne, motif }),
            "NON VALIDÉE" => controle.refusees += 1,
            "EN ATTENTE" => controle.en_attente += 1,
            _ => {}
        }
    }

    // Contradictions : chaque ligne concernée, avec les lignes opposées.
    let mut lignes_opposees: IndexMap<u32, Vec<u32>> = IndexMap::new();
    let mut propositions_contradictoires = Vec::new();
    for (ligne_oui, proposition) in &lignes_oui {
        for (ligne_non, refusee) in &lignes_non {
            if proposition == refusee {
                controle.contradictions.push(format!(
                    "Ligne {ligne_oui} (OUI) et ligne {ligne_non} (NON) : décisions contraires pour « {} » → « {} » ({})",
                    proposition.nom_actuel,
                    proposition.nouveau_nom,
                    proposition.fichier_modele
                ));
                lignes_opposees
                    .entry(*ligne_oui)
                    .or_default()
                    .push(*ligne_non);
                lignes_opposees
                    .entry(*ligne_non)
                    .or_default()
                    .push(*ligne_oui);
                propositions_contradictoires.push(proposition.clone());
            }
        }
    }
    for contradiction in &controle.contradictions {
        journal(&format!("[ERREUR] {contradiction}"));
    }
    // Une proposition contradictoire n'est ni admise ni refusée : le rapport le
    // dit sur ses lignes, pour rester compréhensible hors de l'application.
    controle
        .validees
        .retain(|proposition| !propositions_contradictoires.contains(proposition));
    for ligne in &mut controle.rapport {
        let numero = ligne[0].as_u64().unwrap_or_default() as u32;
        if let Some(opposees) = lignes_opposees.get(&numero) {
            if ligne[7] == "NON VALIDÉE" {
                controle.refusees -= 1;
            }
            let opposees: Vec<String> = opposees.iter().map(u32::to_string).collect();
            ligne[7] = json!("CONTRADICTOIRE");
            ligne[8] = json!(format!(
                "OUI et NON pour la même proposition (ligne {}) : aucun SGX tant que la contradiction n'est pas résolue",
                opposees.join(", ")
            ));
        }
    }

    controle.propositions_sans_decision = propositions_repondues
        .iter()
        .filter(|repondue| !**repondue)
        .count();
    controle
}

fn motif_cellule(colonne: usize, attendu: &str, cellule: &Cellule) -> String {
    format!(
        "{} : {attendu} (valeur lue : {})",
        COLONNES_DECISION[colonne],
        cellule.valeur_lue()
    )
}

// ---------------------------------------------------------------- Renommages

// Applique séquentiellement les modifications validées aux modèles chargés et
// recompte chaque opération (R09). Retourne les divergences (vide si conforme).
pub(crate) fn tester_renommages(
    modeles: &mut IndexMap<String, Value>,
    validees: &[Modification],
    journal: &mut Journal<'_>,
) -> Vec<String> {
    let mut divergences = Vec::new();
    for modification in validees {
        let modele = modeles
            .get_mut(&modification.fichier_modele)
            .expect("modèle chargé avant le test des renommages");
        let nombre = renommer_lanes(modele, &modification.nom_actuel, &modification.nouveau_nom);
        if nombre == modification.occurrences {
            journal(&format!(
                "[OK] Test en mémoire conforme : {nombre} modification(s)"
            ));
        } else {
            journal(&format!(
                "[ATTENTION] Test en mémoire non conforme : {nombre} trouvée(s), {} attendue(s)",
                modification.occurrences
            ));
            divergences.push(format!(
                "{} : « {} » -> « {} » : {nombre} occurrence(s) trouvée(s), {} attendue(s)",
                modification.fichier_modele,
                modification.nom_actuel,
                modification.nouveau_nom,
                modification.occurrences
            ));
        }
    }
    divergences
}

// Vérifie qu'un modèle relu dans le SGX produit ne diffère de sa source que par
// les renommages validés pour ce modèle, avec le nombre d'occurrences attendu.
// Contrôle indépendant du test séquentiel : les renommages sont appliqués en
// une passe sur une copie de la source, puis comparés au modèle produit.
pub(crate) fn verifier_modele(
    fichier_modele: &str,
    source: &Value,
    produit: &Value,
    validees: &[Modification],
) -> Vec<String> {
    let mut ecarts = Vec::new();
    let mut renommages: IndexMap<&str, (&str, u64)> = IndexMap::new();
    for modification in validees
        .iter()
        .filter(|modification| modification.fichier_modele == fichier_modele)
    {
        if renommages
            .insert(
                &modification.nom_actuel,
                (&modification.nouveau_nom, modification.occurrences),
            )
            .is_some()
        {
            ecarts.push(format!(
                "{fichier_modele} : « {} » renommé plusieurs fois",
                modification.nom_actuel
            ));
        }
    }

    let mut attendu = source.clone();
    let mut comptes: IndexMap<&str, u64> = IndexMap::new();
    appliquer_renommages(&mut attendu, &renommages, &mut comptes);

    for (nom_actuel, (nouveau_nom, occurrences)) in &renommages {
        let compte = comptes.get(nom_actuel).copied().unwrap_or(0);
        if compte != *occurrences {
            ecarts.push(format!(
                "{fichier_modele} : « {nom_actuel} » -> « {nouveau_nom} » : {compte} occurrence(s) dans la source, {occurrences} attendue(s)"
            ));
        }
    }
    if attendu != *produit {
        ecarts.push(format!(
            "{fichier_modele} : le contenu produit diffère de la source au-delà des renommages validés"
        ));
    }
    ecarts
}

fn appliquer_renommages<'a>(
    forme: &mut Value,
    renommages: &IndexMap<&'a str, (&'a str, u64)>,
    comptes: &mut IndexMap<&'a str, u64>,
) {
    if est_lane(forme)
        && let Some(nom) = forme["properties"]["name"].as_str()
        && let Some((nom_actuel, (nouveau_nom, _))) = renommages.get_key_value(nom.trim())
    {
        forme["properties"]["name"] = (*nouveau_nom).into();
        *comptes.entry(nom_actuel).or_default() += 1;
    }
    for enfant in enfants_modifiables(forme) {
        appliquer_renommages(enfant, renommages, comptes);
    }
}

// ---------------------------------------------------------------- Tests

// Tests des règles. Les tests marqués « caractérisation » figent un comportement
// historique conservé en V1 ; les autres décrivent le contrat V1.
#[cfg(test)]
mod tests {
    use super::*;

    fn lane(nom: Value, enfants: Vec<Value>) -> Value {
        json!({"stencil": {"id": "Lane"}, "properties": {"name": nom}, "childShapes": enfants})
    }

    fn tache(nom: &str) -> Value {
        json!({"stencil": {"id": "Task"}, "properties": {"name": nom}, "childShapes": []})
    }

    fn inventorier(modele: &Value) -> (Vec<String>, Vec<String>) {
        let mut lanes = Vec::new();
        let mut anomalies = Vec::new();
        trouver_lanes(modele, "racine", &mut lanes, &mut anomalies);
        (lanes, anomalies)
    }

    fn lanes_de(modele: &Value) -> Vec<String> {
        let (lanes, anomalies) = inventorier(modele);
        assert!(anomalies.is_empty(), "{anomalies:?}");
        lanes
    }

    fn occurrence(fichier_modele: &str, flux: &str, lane: &str) -> Occurrence {
        Occurrence {
            fichier_modele: fichier_modele.into(),
            flux: flux.into(),
            lane: lane.into(),
        }
    }

    fn texte(valeur: &str) -> Cellule {
        Cellule::Texte(valeur.into())
    }

    fn ligne_lue(numero: u32, cellules: Vec<Cellule>) -> LigneLue {
        LigneLue { numero, cellules }
    }

    fn modification(
        fichier_modele: &str,
        nom_actuel: &str,
        nouveau_nom: &str,
        occurrences: u64,
    ) -> Modification {
        Modification {
            fichier_modele: fichier_modele.into(),
            flux: "Flux".into(),
            nom_actuel: nom_actuel.into(),
            nouveau_nom: nouveau_nom.into(),
            occurrences,
        }
    }

    // --- Inventaire (R03, P03)

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
    fn inventaire_champs_optionnels_absents_traites_comme_vides() {
        let modele = json!({"childShapes": [
            json!({"stencil": {"id": "Lane"}}),
            json!({"stencil": {"id": "Lane"}, "properties": {}}),
            json!({"stencil": {"id": "Task"}}),
        ]});
        assert!(lanes_de(&modele).is_empty());
        assert!(lanes_de(&json!({})).is_empty());
    }

    #[test]
    fn inventaire_signale_les_noms_non_textuels() {
        let modele = json!({"childShapes": [
            lane(Value::Null, vec![lane(json!("Enfant"), vec![])]),
            lane(json!(123), vec![]),
            json!({"stencil": {"id": "Lane"}, "properties": "x"}),
        ]});
        let (lanes, anomalies) = inventorier(&modele);
        assert_eq!(lanes, ["Enfant"]);
        assert_eq!(
            anomalies,
            [
                "racine.childShapes[0].properties.name : texte attendu",
                "racine.childShapes[1].properties.name : texte attendu",
                "racine.childShapes[2].properties : objet attendu",
            ]
        );
    }

    #[test]
    fn inventaire_signale_les_structures_mal_typees() {
        let (_, anomalies) = inventorier(&json!({"childShapes": 7}));
        assert_eq!(anomalies, ["racine.childShapes : tableau attendu"]);
        let (_, anomalies) = inventorier(&json!({"childShapes": [1, null]}));
        assert_eq!(
            anomalies,
            [
                "racine.childShapes[0] : objet attendu",
                "racine.childShapes[1] : objet attendu"
            ]
        );
        let (_, anomalies) = inventorier(&json!([]));
        assert_eq!(anomalies, ["racine : objet attendu"]);
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
    fn renommage_n_ajoute_aucune_cle() {
        // Défaut hérité corrigé : les formes sans childShapes restaient
        // modifiées avec "childShapes": null.
        let mut modele = json!({"childShapes": [
            {"stencil": {"id": "Lane"}, "properties": {"name": "A"}},
            {"stencil": {"id": "Task"}}
        ]});
        renommer_lanes(&mut modele, "A", "Z");
        assert_eq!(
            modele,
            json!({"childShapes": [
                {"stencil": {"id": "Lane"}, "properties": {"name": "Z"}},
                {"stencil": {"id": "Task"}}
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

    #[test]
    fn test_des_renommages_signale_les_divergences() {
        let mut modeles = IndexMap::from([(
            "m".to_string(),
            json!({"childShapes": [lane(json!("A"), vec![]), lane(json!("B"), vec![])]}),
        )]);
        let validees = [
            modification("m", "A", "B", 1),
            modification("m", "B", "C", 1),
        ];
        let divergences = tester_renommages(&mut modeles, &validees, &mut |_| {});
        assert_eq!(
            divergences,
            ["m : « B » -> « C » : 2 occurrence(s) trouvée(s), 1 attendue(s)"]
        );
    }

    // --- Vérification du résultat

    #[test]
    fn verification_accepte_les_seuls_renommages_valides() {
        let source = json!({"x": 1, "childShapes": [lane(json!(" A "), vec![lane(json!("A"), vec![])]), lane(json!("B"), vec![])]});
        let validees = [modification("m", "A", "Z", 2)];
        let mut produit = source.clone();
        renommer_lanes(&mut produit, "A", "Z");
        assert!(verifier_modele("m", &source, &produit, &validees).is_empty());
    }

    #[test]
    fn verification_detecte_toute_autre_difference() {
        let source =
            json!({"x": 1, "childShapes": [lane(json!("A"), vec![]), lane(json!("B"), vec![])]});
        let validees = [modification("m", "A", "Z", 1)];

        let mut lane_en_trop = source.clone();
        renommer_lanes(&mut lane_en_trop, "A", "Z");
        renommer_lanes(&mut lane_en_trop, "B", "Z");
        let mut propriete_modifiee = source.clone();
        renommer_lanes(&mut propriete_modifiee, "A", "Z");
        propriete_modifiee["x"] = json!(2);
        let mut coquille = source.clone();
        renommer_lanes(&mut coquille, "A", "Zz");

        for produit in [lane_en_trop, propriete_modifiee, coquille, source.clone()] {
            assert_eq!(
                verifier_modele("m", &source, &produit, &validees),
                ["m : le contenu produit diffère de la source au-delà des renommages validés"]
            );
        }
        let mauvais_compte = [modification("m", "A", "Z", 3)];
        let mut produit = source.clone();
        renommer_lanes(&mut produit, "A", "Z");
        assert_eq!(
            verifier_modele("m", &source, &produit, &mauvais_compte),
            ["m : « A » -> « Z » : 1 occurrence(s) dans la source, 3 attendue(s)"]
        );
    }

    // --- Synthèse (R02, R05)

    #[test]
    fn synthese_distingue_les_modeles_homonymes_par_chemin() {
        let occurrences = [
            occurrence("a/model_1_.json", "Flux", "A"),
            occurrence("a/model_1_.json", "Flux", "A"),
            occurrence("b/model_1_.json", "Flux", "A"),
            occurrence("a/model_1_.json", "Flux", "B"),
        ];
        let synthese = synthetiser(&occurrences);
        assert_eq!(synthese.keys().collect::<Vec<_>>(), ["A", "B"]);
        let lane_a = &synthese["A"];
        assert_eq!(lane_a.occurrences, 3);
        let par_modele: Vec<_> = lane_a
            .occurrences_par_modele
            .iter()
            .map(|(chemin, par_modele)| {
                (
                    chemin.as_str(),
                    par_modele.flux.as_str(),
                    par_modele.occurrences,
                )
            })
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
        let occurrences = [
            occurrence("a/model_1_.json", "Flux", "A"),
            occurrence("b/model_1_.json", "Flux", "A"),
        ];
        let synthese = synthetiser(&occurrences);
        assert_eq!(synthese["A"].flux, ["Flux"]);
        assert_eq!(synthese["A"].occurrences_par_flux["Flux"], 2);
        assert_eq!(synthese["A"].occurrences_par_modele.len(), 2);
    }

    // --- Correspondances (R04, P02)

    #[test]
    fn correspondances_retenues_selon_r04() {
        let lignes = [
            ligne_lue(2, vec![texte("A"), texte("Y")]),
            ligne_lue(3, vec![texte("X"), texte("Inconnu")]),
            ligne_lue(4, vec![texte("B"), Cellule::Vide]),
            ligne_lue(5, vec![texte("B"), texte("   ")]),
            ligne_lue(6, vec![Cellule::Nombre(1.0), Cellule::Vide]),
            ligne_lue(7, vec![texte("A"), texte(" Z ")]),
            ligne_lue(8, vec![texte(" C "), texte("D")]),
        ];
        let correspondances = lire_correspondances(&lignes).unwrap();
        assert_eq!(
            correspondances.retenues,
            IndexMap::from([
                ("A".to_string(), "Z".to_string()),
                ("X".to_string(), "Inconnu".to_string()),
                (" C ".to_string(), "D".to_string()),
            ])
        );
        assert_eq!(correspondances.doublons, ["A"]);
    }

    #[test]
    fn correspondances_refusent_les_cellules_non_textuelles() {
        let lignes = [
            ligne_lue(2, vec![Cellule::Nombre(123.0), texte("Z")]),
            ligne_lue(3, vec![texte("A"), Cellule::Nombre(123.0)]),
            ligne_lue(4, vec![texte("B"), Cellule::Formule("=\"Z\"".into())]),
            ligne_lue(5, vec![Cellule::Formule("=A1".into()), Cellule::Vide]),
            ligne_lue(6, vec![Cellule::Vide, texte("Z")]),
            ligne_lue(7, vec![texte("C"), Cellule::Autre("booléen true".into())]),
            ligne_lue(8, vec![texte("D"), texte("E")]),
        ];
        assert_eq!(
            lire_correspondances(&lignes).err().unwrap(),
            [
                "Correspondance, ligne 2, colonne A (Nom actuel) : texte attendu (valeur lue : nombre 123)",
                "Correspondance, ligne 3, colonne B (Nouveau nom) : texte attendu (valeur lue : nombre 123)",
                "Correspondance, ligne 4, colonne B (Nouveau nom) : formule refusée, saisir le texte directement (valeur lue : formule =\"Z\")",
                "Correspondance, ligne 5, colonne A (Nom actuel) : formule refusée, saisir le texte directement (valeur lue : formule =A1)",
                "Correspondance, ligne 6, colonne A (Nom actuel) : texte attendu (valeur lue : cellule vide)",
                "Correspondance, ligne 7, colonne B (Nouveau nom) : texte attendu (valeur lue : booléen true)",
            ]
        );
    }

    // --- Dry-run (R04, R05)

    #[test]
    fn dry_run_une_proposition_par_modele_et_nom() {
        let synthese = synthetiser(&[
            occurrence("a/model_1_.json", "Flux", "A"),
            occurrence("a/model_1_.json", "Flux", "A"),
            occurrence("b/model_1_.json", "Flux", "A"),
            occurrence("a/model_1_.json", "Flux", "B"),
        ]);
        let correspondances: IndexMap<String, String> = [("X", "Y"), ("B", "C"), ("A", "Z")]
            .into_iter()
            .map(|(ancien, nouveau)| (ancien.to_string(), nouveau.to_string()))
            .collect();
        let analyse = dry_run(&correspondances, &synthese);
        let lignes: Vec<_> = analyse
            .iter()
            .map(|modification| {
                (
                    modification.fichier_modele.as_str(),
                    modification.nom_actuel.as_str(),
                    modification.nouveau_nom.as_str(),
                    modification.occurrences,
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
        let synthese = synthetiser(&[occurrence("a/model_1_.json", "Flux", "A")]);
        let correspondances = IndexMap::from([("X".to_string(), "Y".to_string())]);
        assert!(dry_run(&correspondances, &synthese).is_empty());
    }

    // --- Décisions (R07, R08, P02)

    fn decision(numero: u32, occurrences: Cellule, validation: Cellule) -> LigneLue {
        ligne_lue(
            numero,
            vec![
                texte("Flux"),
                texte("A"),
                texte("Z"),
                occurrences,
                validation,
                texte("m"),
            ],
        )
    }

    fn resultats(controle: &Controle) -> Vec<(String, String)> {
        controle
            .rapport
            .iter()
            .map(|ligne| {
                (
                    ligne[7].as_str().unwrap().to_string(),
                    ligne[8].as_str().unwrap().to_string(),
                )
            })
            .collect()
    }

    #[test]
    fn decisions_types_stricts() {
        let analyse = [modification("m", "A", "Z", 2)];
        let lignes = [
            decision(2, Cellule::Nombre(2.0), texte(" oui ")),
            decision(3, texte("2"), texte("OUI")),
            decision(4, Cellule::Nombre(2.5), texte("OUI")),
            decision(5, Cellule::Nombre(-2.0), texte("OUI")),
            decision(6, Cellule::Autre("booléen true".into()), texte("OUI")),
            decision(7, Cellule::Nombre(2.0), Cellule::Formule("=\"OUI\"".into())),
            decision(
                8,
                Cellule::Nombre(2.0),
                Cellule::Autre("booléen true".into()),
            ),
            decision(9, Cellule::Nombre(2.0), texte("peut-être")),
            // NON sur une autre proposition (3 occurrences) : sur la même, ce
            // serait une contradiction avec la ligne 2, hors du sujet de ce test.
            decision(10, Cellule::Nombre(3.0), texte("NON")),
            decision(11, Cellule::Nombre(2.0), Cellule::Vide),
        ];
        let controle = controler_decisions(&analyse, &lignes, &mut |_| {});
        assert_eq!(controle.validees, [modification("m", "A", "Z", 2)]);
        assert_eq!(
            resultats(&controle),
            [
                ("VALIDÉE", "Conforme au dry-run"),
                (
                    "IGNORÉE",
                    "Occurrences : nombre entier positif attendu (valeur lue : « 2 »)"
                ),
                (
                    "IGNORÉE",
                    "Occurrences : nombre entier positif attendu (valeur lue : nombre 2.5)"
                ),
                (
                    "IGNORÉE",
                    "Occurrences : nombre entier positif attendu (valeur lue : nombre -2)"
                ),
                (
                    "IGNORÉE",
                    "Occurrences : nombre entier positif attendu (valeur lue : booléen true)"
                ),
                (
                    "EN ATTENTE",
                    "Validation : texte OUI ou NON attendu (valeur lue : formule =\"OUI\")"
                ),
                (
                    "EN ATTENTE",
                    "Validation : texte OUI ou NON attendu (valeur lue : booléen true)"
                ),
                (
                    "EN ATTENTE",
                    "Validation « PEUT-ÊTRE » non reconnue : OUI ou NON attendu"
                ),
                ("NON VALIDÉE", "Refus humain"),
                ("EN ATTENTE", "Aucune validation renseignée"),
            ]
            .map(|(resultat, motif)| (resultat.to_string(), motif.to_string()))
        );
        assert_eq!(controle.ignorees.len(), 4);
        assert_eq!(controle.refusees, 1);
        assert_eq!(controle.en_attente, 4);
        assert_eq!(controle.propositions_sans_decision, 0);
    }

    #[test]
    fn decisions_champ_texte_non_textuel_ignore() {
        let analyse = [modification("m", "123", "Z", 1)];
        let lignes = [ligne_lue(
            2,
            vec![
                texte("Flux"),
                Cellule::Nombre(123.0),
                texte("Z"),
                Cellule::Nombre(1.0),
                texte("OUI"),
                texte("m"),
            ],
        )];
        let controle = controler_decisions(&analyse, &lignes, &mut |_| {});
        assert!(controle.validees.is_empty());
        assert_eq!(
            resultats(&controle),
            [(
                "IGNORÉE".to_string(),
                "Nom actuel : texte attendu (valeur lue : nombre 123)".to_string()
            )]
        );
    }

    #[test]
    fn decisions_propositions_sans_reponse_comptees() {
        let analyse = [
            modification("m", "A", "Z", 2),
            modification("n", "A", "Z", 1),
        ];
        let lignes = [decision(2, Cellule::Nombre(2.0), texte("NON"))];
        let controle = controler_decisions(&analyse, &lignes, &mut |_| {});
        assert_eq!(controle.propositions_sans_decision, 1);
    }

    // OUI et NON sur la même proposition, dans les deux ordres : contradiction
    // signalée avec ses lignes. OUI répété : pas une contradiction (R09 garde le
    // doublon, que le recomptage bloque ensuite).
    #[test]
    fn decisions_contradictoires_signalees() {
        let analyse = [modification("m", "A", "Z", 2)];
        for (premiere, deuxieme) in [("OUI", "NON"), ("NON", "OUI")] {
            let lignes = [
                decision(2, Cellule::Nombre(2.0), texte(premiere)),
                decision(3, Cellule::Nombre(2.0), texte(deuxieme)),
            ];
            let controle = controler_decisions(&analyse, &lignes, &mut |_| {});
            let (ligne_oui, ligne_non) = if premiere == "OUI" { (2, 3) } else { (3, 2) };
            assert_eq!(
                controle.contradictions,
                [format!(
                    "Ligne {ligne_oui} (OUI) et ligne {ligne_non} (NON) : décisions contraires pour « A » → « Z » (m)"
                )]
            );
            // Ni admise ni refusée ; le rapport le dit sur les deux lignes.
            assert!(controle.validees.is_empty());
            assert_eq!(controle.refusees, 0);
            let suffixe = " : aucun SGX tant que la contradiction n'est pas résolue";
            assert_eq!(
                resultats(&controle),
                [
                    (
                        "CONTRADICTOIRE".to_string(),
                        format!("OUI et NON pour la même proposition (ligne 3){suffixe}")
                    ),
                    (
                        "CONTRADICTOIRE".to_string(),
                        format!("OUI et NON pour la même proposition (ligne 2){suffixe}")
                    ),
                ]
            );
        }

        let lignes = [
            decision(2, Cellule::Nombre(2.0), texte("OUI")),
            decision(3, Cellule::Nombre(2.0), texte("OUI")),
        ];
        let controle = controler_decisions(&analyse, &lignes, &mut |_| {});
        assert!(controle.contradictions.is_empty());
        assert_eq!(controle.validees.len(), 2);

        // Un NON sur une autre proposition n'est pas une contradiction.
        let analyse = [
            modification("m", "A", "Z", 2),
            modification("n", "A", "Z", 2),
        ];
        let mut autre = decision(3, Cellule::Nombre(2.0), texte("NON"));
        autre.cellules[5] = texte("n");
        let lignes = [decision(2, Cellule::Nombre(2.0), texte("OUI")), autre];
        let controle = controler_decisions(&analyse, &lignes, &mut |_| {});
        assert!(controle.contradictions.is_empty());
    }
}
