// Lecture et écriture de l'archive SGX. La source n'est jamais modifiée ; le
// SGX produit est écrit dans un fichier temporaire, relu, vérifié, puis publié
// sous son nom final sans jamais écraser un fichier existant.

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use serde_json::Value;
use zip::write::FullFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::regles::{Modification, Occurrence, trouver_lanes, verifier_modele};
use crate::{Contexte, Erreur, Journal, Resultat};

const SUFFIXE_MODELE: &str = "model_1_.json";
const SUFFIXE_METADONNEES: &str = "model_meta.json";
// Champ supplémentaire ZIP64 : recalculé par la bibliothèque, jamais recopié.
const CHAMP_ZIP64: u16 = 0x0001;

// Retourne l'unique fichier .sgx du dossier (extension sans casse).
pub(crate) fn trouver_sgx(dossier: &Path) -> Resultat<PathBuf> {
    let mut fichiers: Vec<PathBuf> = fs::read_dir(dossier)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entree| entree.path())
        .filter(|chemin| {
            chemin
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("sgx"))
        })
        .collect();

    match fichiers.len() {
        0 => Err(Erreur::nouvelle(
            "source_introuvable",
            format!("Aucun fichier SGX trouvé dans {}/", dossier.display()),
        )),
        1 => Ok(fichiers.remove(0)),
        _ => Err(Erreur::nouvelle(
            "sources_multiples",
            format!(
                "Plusieurs fichiers SGX trouvés dans {}/ : un seul fichier est attendu",
                dossier.display()
            ),
        )),
    }
}

fn ouvrir(sgx: &Path) -> Resultat<ZipArchive<File>> {
    let message = format!(
        "Ce fichier n'est pas une archive SGX lisible : {}",
        sgx.display()
    );
    let fichier = File::open(sgx).contexte("archive_illisible", &message)?;
    ZipArchive::new(fichier).contexte("archive_illisible", &message)
}

fn lire_json(archive: &mut ZipArchive<File>, chemin: &str) -> Result<Value, String> {
    let mut contenu = Vec::new();
    archive
        .by_name(chemin)
        .map_err(|erreur| format!("entrée illisible ({erreur})"))?
        .read_to_end(&mut contenu)
        .map_err(|erreur| format!("entrée illisible ({erreur})"))?;
    serde_json::from_slice(&contenu).map_err(|erreur| format!("JSON invalide ({erreur})"))
}

// Nom du flux lu dans les métadonnées du modèle : absent = vide, non textuel = anomalie.
fn lire_flux(archive: &mut ZipArchive<File>, fichier_metadonnees: &str) -> Result<String, String> {
    let metadonnees = lire_json(archive, fichier_metadonnees)?;
    let Some(objet) = metadonnees.as_object() else {
        return Err("objet JSON attendu".to_string());
    };
    match objet.get("name") {
        None => Ok(String::new()),
        Some(Value::String(nom)) => Ok(nom.clone()),
        Some(_) => Err("name : texte attendu".to_string()),
    }
}

// Inventorie les lanes de tous les modèles de l'archive (R02, R03). Toute
// anomalie (JSON illisible, métadonnées absentes, structure mal typée, entrée
// en double) bloque l'inventaire (P03) : aucun résultat partiel.
// Retourne les occurrences et le nombre de modèles reconnus.
pub(crate) fn extraire_lanes(
    sgx: &Path,
    journal: &mut Journal<'_>,
) -> Resultat<(Vec<Occurrence>, usize)> {
    let mut archive = ouvrir(sgx)?;
    journal("[OK] Archive SGX ouverte");

    let noms: Vec<String> = (0..archive.len())
        .filter_map(|index| archive.name_for_index(index).map(str::to_string))
        .collect();
    let mut anomalies = Vec::new();
    let mut deja_vus = HashSet::new();
    for nom in &noms {
        if !deja_vus.insert(nom) {
            anomalies.push(format!(
                "{nom} : entrée présente plusieurs fois dans l'archive"
            ));
        }
    }

    let fichiers_modeles: Vec<&String> = noms
        .iter()
        .filter(|nom| nom.ends_with(SUFFIXE_MODELE))
        .collect();
    if fichiers_modeles.is_empty() {
        return Err(Erreur::nouvelle(
            "aucun_modele",
            "Aucun modèle compatible trouvé dans ce SGX : sélectionnez un autre export.",
        ));
    }

    let modeles_reconnus = fichiers_modeles.len();
    let mut occurrences = Vec::new();
    for fichier_modele in fichiers_modeles {
        let fichier_metadonnees = fichier_modele.replace(SUFFIXE_MODELE, SUFFIXE_METADONNEES);
        let flux = match lire_flux(&mut archive, &fichier_metadonnees) {
            Ok(flux) => flux,
            Err(motif) => {
                anomalies.push(format!("{fichier_metadonnees} : {motif}"));
                continue;
            }
        };
        let modele = match lire_json(&mut archive, fichier_modele) {
            Ok(modele) => modele,
            Err(motif) => {
                anomalies.push(format!("{fichier_modele} : {motif}"));
                continue;
            }
        };
        let mut lanes = Vec::new();
        let mut anomalies_modele = Vec::new();
        trouver_lanes(&modele, "racine", &mut lanes, &mut anomalies_modele);
        anomalies.extend(
            anomalies_modele
                .into_iter()
                .map(|anomalie| format!("{fichier_modele} : {anomalie}")),
        );
        for lane in lanes {
            occurrences.push(Occurrence {
                fichier_modele: fichier_modele.clone(),
                flux: flux.clone(),
                lane,
            });
        }
    }

    if !anomalies.is_empty() {
        return Err(Erreur::nouvelle(
            "modeles_invalides",
            format!(
                "Le SGX contient {} anomalie(s) : aucun traitement n'est possible tant qu'elles ne sont pas corrigées.",
                anomalies.len()
            ),
        )
        .avec_details(anomalies));
    }
    Ok((occurrences, modeles_reconnus))
}

// Charge en mémoire, dans l'ordre de première citation, les modèles visés
// par les modifications validées.
pub(crate) fn charger_modeles(
    sgx: &Path,
    validees: &[Modification],
) -> Resultat<IndexMap<String, Value>> {
    let mut archive = ouvrir(sgx)?;
    let mut modeles = IndexMap::new();
    for modification in validees {
        if !modeles.contains_key(&modification.fichier_modele) {
            let modele =
                lire_json(&mut archive, &modification.fichier_modele).map_err(|motif| {
                    Erreur::nouvelle(
                        "modeles_invalides",
                        format!("{} : {motif}", modification.fichier_modele),
                    )
                })?;
            modeles.insert(modification.fichier_modele.clone(), modele);
        }
    }
    Ok(modeles)
}

// Découpe le champ supplémentaire brut d'une entrée en (identifiant, données).
fn champs_supplementaires(brut: &[u8]) -> Vec<(u16, &[u8])> {
    let mut champs = Vec::new();
    let mut position = 0;
    while position + 4 <= brut.len() {
        let identifiant = u16::from_le_bytes([brut[position], brut[position + 1]]);
        let taille = u16::from_le_bytes([brut[position + 2], brut[position + 3]]) as usize;
        let debut = position + 4;
        let Some(donnees) = brut.get(debut..debut + taille) else {
            break;
        };
        champs.push((identifiant, donnees));
        position = debut + taille;
    }
    champs
}

// Produit le SGX modifié. Retourne les transformations d'attributs ZIP qui
// n'ont pas pu être conservées (P06). En cas d'échec, aucun fichier final
// n'est créé et le temporaire est supprimé.
pub(crate) fn produire_sgx(
    source: &Path,
    destination: &Path,
    modeles: &IndexMap<String, Value>,
    validees: &[Modification],
) -> Resultat<Vec<String>> {
    let mut nom_temporaire = destination.file_name().unwrap_or_default().to_os_string();
    nom_temporaire.push(".en-cours");
    let temporaire = destination.with_file_name(nom_temporaire);

    let resultat = ecrire_sgx(source, &temporaire, modeles).and_then(|transformations| {
        let ecarts = verifier_sgx(source, &temporaire, validees)?;
        if !ecarts.is_empty() {
            return Err(Erreur::nouvelle(
                "verification_echouee",
                "Le SGX produit ne correspond pas exactement aux modifications validées : il n'a pas été publié.",
            )
            .avec_details(ecarts));
        }
        if destination.exists() {
            return Err(Erreur::nouvelle(
                "sortie_existante",
                format!("Un fichier existe déjà : {}", destination.display()),
            ));
        }
        fs::rename(&temporaire, destination).contexte(
            "ecriture_impossible",
            &format!("Le SGX n'a pas pu être créé : {}", destination.display()),
        )?;
        Ok(transformations)
    });
    if resultat.is_err() {
        let _ = fs::remove_file(&temporaire);
    }
    resultat
}

// Écrit la nouvelle archive : entrées inchangées recopiées sans recompression,
// modèles modifiés réécrits avec leur méthode de compression, leur date, leurs
// permissions, leur commentaire et leurs champs supplémentaires (P06).
fn ecrire_sgx(
    source: &Path,
    destination: &Path,
    modeles: &IndexMap<String, Value>,
) -> Resultat<Vec<String>> {
    let mut archive = ouvrir(source)?;
    let message = format!("Le SGX n'a pas pu être créé : {}", destination.display());
    let fichier = File::create(destination).contexte("ecriture_impossible", &message)?;
    let mut sortie = ZipWriter::new(fichier);
    let mut transformations = Vec::new();

    for index in 0..archive.len() {
        let entree = archive.by_index_raw(index)?;
        let Some(modele) = modeles.get(entree.name()) else {
            sortie.raw_copy_file(entree)?;
            continue;
        };
        let mut options = FullFileOptions::default()
            .compression_method(entree.compression())
            .last_modified_time(entree.last_modified().unwrap_or_default());
        if let Some(mode) = entree.unix_mode() {
            options = options.unix_permissions(mode);
        }
        if !entree.comment().is_empty() {
            options = options.with_file_comment(entree.comment());
        }
        for (identifiant, donnees) in
            champs_supplementaires(entree.extra_data().unwrap_or_default())
        {
            if identifiant != CHAMP_ZIP64
                && options.add_extra_data(identifiant, donnees, false).is_err()
            {
                transformations.push(format!(
                    "{} : champ supplémentaire 0x{identifiant:04x} non conservé",
                    entree.name()
                ));
            }
        }
        let nom = entree.name().to_string();
        drop(entree);
        sortie.start_file(nom, options)?;
        sortie.write_all(serde_json::to_string(modele)?.as_bytes())?;
    }
    sortie.set_raw_comment(archive.comment().into())?;
    let fichier = sortie.finish()?;
    fichier
        .sync_all()
        .contexte("ecriture_impossible", &message)?;
    Ok(transformations)
}

// Relit le SGX produit et le compare à la source : mêmes entrées dans le même
// ordre, entrées non modifiées identiques octet pour octet après décompression
// (sommes de contrôle vérifiées), modèles modifiés conformes aux seuls
// renommages validés. Retourne les écarts (vide si conforme).
fn verifier_sgx(source: &Path, produit: &Path, validees: &[Modification]) -> Resultat<Vec<String>> {
    let mut archive_source = ouvrir(source)?;
    let mut archive_produite = ouvrir(produit)?;
    let modeles_modifies: HashSet<&str> = validees
        .iter()
        .map(|modification| modification.fichier_modele.as_str())
        .collect();

    let noms_source: Vec<String> = (0..archive_source.len())
        .filter_map(|index| archive_source.name_for_index(index).map(str::to_string))
        .collect();
    let noms_produits: Vec<String> = (0..archive_produite.len())
        .filter_map(|index| archive_produite.name_for_index(index).map(str::to_string))
        .collect();
    if noms_source != noms_produits {
        return Ok(vec![
            "la liste ou l'ordre des entrées diffère de la source".to_string(),
        ]);
    }

    let mut ecarts = Vec::new();
    for (index, nom) in noms_source.iter().enumerate() {
        let contenu_source = lire_entree(&mut archive_source, index)?;
        let contenu_produit = match lire_entree(&mut archive_produite, index) {
            Ok(contenu) => contenu,
            Err(erreur) => {
                ecarts.push(format!(
                    "{nom} : entrée illisible dans le SGX produit ({erreur})"
                ));
                continue;
            }
        };
        if modeles_modifies.contains(nom.as_str()) {
            match (
                serde_json::from_slice::<Value>(&contenu_source),
                serde_json::from_slice::<Value>(&contenu_produit),
            ) {
                (Ok(modele_source), Ok(modele_produit)) => ecarts.extend(verifier_modele(
                    nom,
                    &modele_source,
                    &modele_produit,
                    validees,
                )),
                _ => ecarts.push(format!("{nom} : JSON illisible après écriture")),
            }
        } else if contenu_source != contenu_produit {
            ecarts.push(format!(
                "{nom} : contenu modifié alors qu'aucun renommage n'est validé"
            ));
        }
    }
    for fichier_modele in &modeles_modifies {
        if !noms_source.iter().any(|nom| nom == fichier_modele) {
            ecarts.push(format!(
                "{fichier_modele} : modèle validé absent de la source"
            ));
        }
    }
    Ok(ecarts)
}

fn lire_entree(archive: &mut ZipArchive<File>, index: usize) -> Resultat<Vec<u8>> {
    let mut contenu = Vec::new();
    archive.by_index(index)?.read_to_end(&mut contenu)?;
    Ok(contenu)
}

// ---------------------------------------------------------------- Tests

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn modification(nom_actuel: &str, nouveau_nom: &str, occurrences: u64) -> Modification {
        Modification {
            fichier_modele: "m/model_1_.json".into(),
            flux: "Flux".into(),
            nom_actuel: nom_actuel.into(),
            nouveau_nom: nouveau_nom.into(),
            occurrences,
        }
    }

    fn preparer(nom: &str) -> (PathBuf, PathBuf) {
        let dossier = std::env::temp_dir().join(format!("bpmn-sgx-{}-{nom}", std::process::id()));
        let _ = fs::remove_dir_all(&dossier);
        fs::create_dir_all(&dossier).unwrap();
        let source = dossier.join("source.sgx");
        let mut zip = ZipWriter::new(File::create(&source).unwrap());
        let modele =
            json!({"childShapes": [{"stencil": {"id": "Lane"}, "properties": {"name": "A"}}]});
        zip.start_file("m/model_1_.json", FullFileOptions::default())
            .unwrap();
        zip.write_all(modele.to_string().as_bytes()).unwrap();
        zip.finish().unwrap();
        (dossier, source)
    }

    fn modele_renomme(nom: &str) -> IndexMap<String, Value> {
        IndexMap::from([(
            "m/model_1_.json".to_string(),
            json!({"childShapes": [{"stencil": {"id": "Lane"}, "properties": {"name": nom}}]}),
        )])
    }

    #[test]
    fn coquille_detectee_et_sgx_non_publie() {
        let (dossier, source) = preparer("coquille");
        let destination = dossier.join("sortie.sgx");
        // Le modèle à écrire ne correspond pas à la décision validée (A -> Z).
        let erreur = produire_sgx(
            &source,
            &destination,
            &modele_renomme("Zz"),
            &[modification("A", "Z", 1)],
        )
        .unwrap_err();
        assert_eq!(erreur.code, "verification_echouee");
        assert_eq!(
            erreur.details,
            [
                "m/model_1_.json : le contenu produit diffère de la source au-delà des renommages validés"
            ]
        );
        let restants: Vec<_> = fs::read_dir(&dossier)
            .unwrap()
            .map(|entree| entree.unwrap().file_name())
            .collect();
        assert_eq!(restants, ["source.sgx"]);
        fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn sortie_existante_jamais_ecrasee() {
        let (dossier, source) = preparer("existante");
        let destination = dossier.join("sortie.sgx");
        fs::write(&destination, b"ANCIEN").unwrap();
        let erreur = produire_sgx(
            &source,
            &destination,
            &modele_renomme("Z"),
            &[modification("A", "Z", 1)],
        )
        .unwrap_err();
        assert_eq!(erreur.code, "sortie_existante");
        assert_eq!(fs::read(&destination).unwrap(), b"ANCIEN");
        assert!(!dossier.join("sortie.sgx.en-cours").exists());
        fs::remove_dir_all(&dossier).unwrap();
    }

    #[test]
    fn production_conforme_publiee() {
        let (dossier, source) = preparer("conforme");
        let destination = dossier.join("sortie.sgx");
        let transformations = produire_sgx(
            &source,
            &destination,
            &modele_renomme("Z"),
            &[modification("A", "Z", 1)],
        )
        .unwrap();
        assert!(transformations.is_empty());
        assert!(destination.exists());
        assert!(!dossier.join("sortie.sgx.en-cours").exists());
        fs::remove_dir_all(&dossier).unwrap();
    }
}
