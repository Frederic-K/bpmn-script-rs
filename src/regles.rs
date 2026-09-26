//! Règles métier en mémoire : inventaire des lanes, synthèse, dry-run,
//! contrôle des décisions et test des renommages. Aucun accès fichier.

use indexmap::IndexMap;
use serde::Serialize;
use serde_json::{Value, json};

use crate::Journal;

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

/// Une proposition du dry-run (toutes les valeurs sont connues).
#[derive(Serialize, Clone, PartialEq)]
pub(crate) struct Modification {
    pub(crate) fichier_modele: String,
    pub(crate) flux: String,
    pub(crate) nom_actuel: String,
    pub(crate) nouveau_nom: String,
    pub(crate) occurrences: u64,
}

/// Une ligne relue dans l'Excel de validation (les cellules peuvent être vides).
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

/// Une ligne de décision relue : proposition et valeur brute de « Validation ».
pub(crate) struct Decision {
    pub(crate) ligne: Ligne,
    pub(crate) validation: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct Ignoree {
    #[serde(flatten)]
    ligne: Ligne,
    motif: &'static str,
}

/// Résultat du contrôle des décisions.
pub(crate) struct Controle {
    pub(crate) validees: Vec<Modification>,
    pub(crate) ignorees: Vec<Ignoree>,
    /// Lignes du rapport `controle_validation.xlsx`, dans l'ordre des colonnes.
    pub(crate) rapport: Vec<Vec<Value>>,
}

// ---------------------------------------------------------------- Lanes

fn est_lane(shape: &Value) -> bool {
    shape["stencil"]["id"] == "Lane"
}

pub(crate) fn trouver_lanes(shape: &Value, lanes: &mut Vec<String>) {
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

// ---------------------------------------------------------------- Traitements

pub(crate) fn synthetiser(resultats: &[Occurrence]) -> IndexMap<String, Synthese> {
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

/// Retient les correspondances relues (ancien nom, nouveau nom) : nouveau nom
/// nettoyé, vide ignoré, dernière valeur retenue pour un même ancien nom.
pub(crate) fn retenir_correspondances(
    lignes: Vec<(Option<String>, Option<String>)>,
) -> IndexMap<String, String> {
    let mut correspondances = IndexMap::new();
    for ligne in lignes {
        let (Some(actuel), Some(nouveau)) = ligne else {
            continue;
        };
        let nouveau = nouveau.trim();
        if !nouveau.is_empty() {
            correspondances.insert(actuel, nouveau.to_string());
        }
    }
    correspondances
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

/// Compare les décisions relues au dry-run et retourne les modifications
/// validées conformes, les OUI ignorés et les lignes du rapport de contrôle.
pub(crate) fn controler_decisions(
    analyse: &[Modification],
    decisions: Vec<Decision>,
    journal: &mut Journal<'_>,
) -> Controle {
    let mut validees = Vec::new();
    let mut ignorees = Vec::new();
    let mut rapport = Vec::new();

    for (i, Decision { ligne, validation }) in decisions.into_iter().enumerate() {
        let validation = validation.map(|v| v.trim().to_uppercase());

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
                    journal(
                        "[ATTENTION] Une ligne validée ne correspond plus au dry-run : modification ignorée",
                    );
                    ("IGNORÉE", "Ne correspond plus au dry-run")
                }
            },
            Some("NON") => ("NON VALIDÉE", "Refus humain"),
            _ => ("EN ATTENTE", "Aucune validation renseignée"),
        };

        rapport.push(vec![
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

    Controle {
        validees,
        ignorees,
        rapport,
    }
}

/// Applique séquentiellement les modifications validées aux modèles chargés
/// et recompte chaque opération. Retourne `true` si tous les décomptes sont
/// conformes. Chaque modèle cité doit être présent dans `modeles`.
pub(crate) fn tester_renommages(
    modeles: &mut IndexMap<String, Value>,
    validees: &[Modification],
    journal: &mut Journal<'_>,
) -> bool {
    let mut conformes = true;
    for m in validees {
        let modele = modeles
            .get_mut(&m.fichier_modele)
            .expect("modèle chargé avant le test des renommages");
        let n = renommer_lanes(modele, &m.nom_actuel, &m.nouveau_nom);
        if n == m.occurrences {
            journal(&format!(
                "[OK] Test en mémoire conforme : {n} modification(s)"
            ));
        } else {
            conformes = false;
            journal(&format!(
                "[ATTENTION] Test en mémoire non conforme : {n} trouvée(s), {} attendue(s)",
                m.occurrences
            ));
        }
    }
    conformes
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
}
