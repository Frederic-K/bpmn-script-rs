//! Lecture et écriture de l'archive SGX. La source n'est jamais modifiée.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use serde_json::Value;
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::regles::{Modification, Occurrence, trouver_lanes};
use crate::{Journal, Res};

/// Retourne l'unique fichier `.sgx` du dossier (extension sans casse).
pub(crate) fn trouver_sgx(dossier: &Path) -> Res<PathBuf> {
    let mut fichiers: Vec<PathBuf> = fs::read_dir(dossier)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entree| entree.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("sgx")))
        .collect();

    match fichiers.len() {
        0 => Err(format!("Aucun fichier SGX trouvé dans {}/", dossier.display()).into()),
        1 => Ok(fichiers.remove(0)),
        _ => Err(format!(
            "Plusieurs fichiers SGX trouvés dans {}/ : un seul fichier est attendu",
            dossier.display()
        )
        .into()),
    }
}

fn lire_json(archive: &mut ZipArchive<File>, chemin: &str) -> Res<Value> {
    let mut contenu = String::new();
    archive.by_name(chemin)?.read_to_string(&mut contenu)?;
    Ok(serde_json::from_str(&contenu)?)
}

/// Inventorie les lanes de tous les modèles `model_1_.json` de l'archive.
pub(crate) fn extraire_lanes(sgx: &Path, journal: &mut Journal<'_>) -> Res<Vec<Occurrence>> {
    let mut archive = ZipArchive::new(File::open(sgx)?)?;
    journal("[OK] Archive SGX ouverte");

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

/// Charge en mémoire, dans l'ordre de première citation, les modèles visés
/// par les modifications validées.
pub(crate) fn charger_modeles(
    sgx: &Path,
    validees: &[Modification],
) -> Res<IndexMap<String, Value>> {
    let mut archive = ZipArchive::new(File::open(sgx)?)?;
    let mut modeles = IndexMap::new();
    for m in validees {
        if !modeles.contains_key(&m.fichier_modele) {
            let modele = lire_json(&mut archive, &m.fichier_modele)?;
            modeles.insert(m.fichier_modele.clone(), modele);
        }
    }
    Ok(modeles)
}

/// Écrit une nouvelle archive : entrées inchangées copiées telles quelles,
/// modèles modifiés réécrits avec leur méthode de compression et leur date.
pub(crate) fn ecrire_sgx(
    source: &Path,
    destination: &Path,
    modeles: &IndexMap<String, Value>,
) -> Res<()> {
    let mut archive = ZipArchive::new(File::open(source)?)?;
    let mut sortie = ZipWriter::new(File::create(destination)?);

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
    Ok(())
}
