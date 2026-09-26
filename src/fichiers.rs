// Publication de fichiers sans jamais écraser un fichier existant.
//
// Un fichier est d'abord écrit complètement sous un nom temporaire, puis publié
// sous son nom final par une opération qui échoue si ce nom existe déjà, même
// s'il est apparu entre-temps. Un arrêt brutal peut laisser le temporaire
// (nom en « .en-cours » ou « .copie-en-cours »), jamais un fichier final incomplet.

use std::fs::{self, File};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use crate::{Erreur, Resultat};

// Publie `temporaire` sous `destination`, puis retire le temporaire. Le lien
// physique est refusé par le système si la destination existe. Sur un système
// de fichiers sans liens physiques (clé USB en FAT ou exFAT), repli sur une
// copie exclusive vérifiée ; seul ce repli peut laisser un fichier final
// incomplet en cas d'arrêt brutal pendant la copie.
pub(crate) fn publier_sans_ecraser(temporaire: &Path, destination: &Path) -> io::Result<()> {
    let publication = match fs::hard_link(temporaire, destination) {
        Err(erreur) if erreur.kind() != ErrorKind::AlreadyExists => {
            copier_exclusivement(temporaire, destination)
        }
        autre => autre,
    };
    let _ = fs::remove_file(temporaire);
    publication
}

// Copie un fichier du traitement vers l'emplacement choisi par l'utilisateur :
// copie complète dans « <destination>.copie-en-cours », puis publication sans
// écrasement. Refusée si la destination existe, même si l'utilisateur a
// confirmé le remplacement dans le dialogue.
pub fn copier_sans_ecraser(source: &Path, destination: &Path) -> Resultat<()> {
    let temporaire = nom_temporaire(destination, ".copie-en-cours");
    let echec = |cause: io::Error| {
        Erreur::nouvelle(
            "copie_impossible",
            format!(
                "La copie n'a pas pu être enregistrée : {} ({cause})",
                destination.display()
            ),
        )
    };
    copier_exclusivement(source, &temporaire).map_err(|cause| {
        if cause.kind() == ErrorKind::AlreadyExists {
            Erreur::nouvelle(
                "copie_impossible",
                format!(
                    "Une copie interrompue a laissé ce fichier : {}. Supprimez-le, puis recommencez.",
                    temporaire.display()
                ),
            )
        } else {
            echec(cause)
        }
    })?;
    publier_sans_ecraser(&temporaire, destination).map_err(|cause| {
        if cause.kind() == ErrorKind::AlreadyExists {
            Erreur::nouvelle(
                "copie_impossible",
                format!(
                    "Un fichier existe déjà : {}. Choisissez un nouveau nom ; aucun fichier existant n'est remplacé.",
                    destination.display()
                ),
            )
        } else {
            echec(cause)
        }
    })
}

// « export.sgx » -> « export.sgx<suffixe> », dans le même dossier.
pub(crate) fn nom_temporaire(destination: &Path, suffixe: &str) -> PathBuf {
    let mut nom = destination.file_name().unwrap_or_default().to_os_string();
    nom.push(suffixe);
    destination.with_file_name(nom)
}

// Crée `destination` (refusé si elle existe), y copie `source`, synchronise et
// relit pour comparer. En cas d'échec après la création, le fichier créé est retiré.
fn copier_exclusivement(source: &Path, destination: &Path) -> io::Result<()> {
    let mut copie = File::create_new(destination)?;
    let resultat = remplir(source, &mut copie, destination);
    drop(copie);
    if resultat.is_err() {
        let _ = fs::remove_file(destination);
    }
    resultat
}

fn remplir(source: &Path, copie: &mut File, destination: &Path) -> io::Result<()> {
    io::copy(&mut File::open(source)?, copie)?;
    copie.sync_all()?;
    if fs::read(source)? != fs::read(destination)? {
        return Err(io::Error::other("la copie relue diffère de l'original"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dossier_de_test(nom: &str) -> PathBuf {
        let dossier =
            std::env::temp_dir().join(format!("bpmn-fichiers-{nom}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dossier);
        fs::create_dir_all(dossier.join("sorties")).unwrap();
        dossier
    }

    // Revue D01 et R1 : aucune copie ne remplace un fichier, y compris la
    // source désignée par un autre chemin ; aucun temporaire laissé.
    #[test]
    fn copie_sans_ecrasement() {
        let racine = dossier_de_test("copie");
        let source = racine.join("sorties/export_modifie.sgx");
        let existant = racine.join("existant.sgx");
        fs::write(&source, b"sgx produit").unwrap();
        fs::write(&existant, b"autre contenu").unwrap();

        let refus = copier_sans_ecraser(&source, &existant).unwrap_err();
        assert_eq!(refus.code, "copie_impossible");
        assert_eq!(fs::read(&existant).unwrap(), b"autre contenu");
        assert!(!nom_temporaire(&existant, ".copie-en-cours").exists());

        for alias in [
            source.clone(),
            racine.join("sorties/./export_modifie.sgx"),
            racine.join("sorties/../sorties/export_modifie.sgx"),
        ] {
            assert!(copier_sans_ecraser(&source, &alias).is_err());
            assert_eq!(fs::read(&source).unwrap(), b"sgx produit");
        }

        let copie = racine.join("copie.sgx");
        copier_sans_ecraser(&source, &copie).unwrap();
        assert_eq!(fs::read(&copie).unwrap(), b"sgx produit");
        assert!(!nom_temporaire(&copie, ".copie-en-cours").exists());

        // Dossier de destination absent : erreur, aucun fichier laissé.
        let impossible = racine.join("absent/copie.sgx");
        assert!(copier_sans_ecraser(&source, &impossible).is_err());
        assert!(!impossible.exists());

        // Erreur de lecture de la source (ici un dossier) : ni copie ni temporaire.
        let depuis_un_dossier = racine.join("depuis-un-dossier.sgx");
        assert!(copier_sans_ecraser(&racine.join("sorties"), &depuis_un_dossier).is_err());
        assert!(!depuis_un_dossier.exists());
        assert!(!nom_temporaire(&depuis_un_dossier, ".copie-en-cours").exists());

        // Temporaire laissé par une copie interrompue : jamais écrasé, signalé.
        let interrompue = racine.join("interrompue.sgx");
        let reste = nom_temporaire(&interrompue, ".copie-en-cours");
        fs::write(&reste, b"reste").unwrap();
        let refus = copier_sans_ecraser(&source, &interrompue).unwrap_err();
        assert!(refus.message.contains("Supprimez-le"));
        assert_eq!(fs::read(&reste).unwrap(), b"reste");
        assert!(!interrompue.exists());
        fs::remove_dir_all(&racine).unwrap();
    }

    // Destination apparue entre l'écriture du temporaire et la publication :
    // elle est conservée, le temporaire est retiré.
    #[test]
    fn publication_refusee_si_la_destination_est_apparue() {
        let racine = dossier_de_test("publication");
        let temporaire = racine.join("sortie.sgx.en-cours");
        let destination = racine.join("sortie.sgx");
        fs::write(&temporaire, b"nouveau").unwrap();
        fs::write(&destination, b"concurrent").unwrap();

        let erreur = publier_sans_ecraser(&temporaire, &destination).unwrap_err();
        assert_eq!(erreur.kind(), ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&destination).unwrap(), b"concurrent");
        assert!(!temporaire.exists());

        fs::write(&temporaire, b"nouveau").unwrap();
        let libre = racine.join("libre.sgx");
        publier_sans_ecraser(&temporaire, &libre).unwrap();
        assert_eq!(fs::read(&libre).unwrap(), b"nouveau");
        assert!(!temporaire.exists());
        fs::remove_dir_all(&racine).unwrap();
    }
}
