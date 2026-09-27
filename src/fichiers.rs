// Fichiers : empreintes, et publication sans jamais écraser un fichier existant.
//
// Un fichier est d'abord écrit complètement sous un nom temporaire (« .en-cours »
// ou « .copie-en-cours »), puis publié sous son nom final par un lien physique :
// le système refuse ce lien si le nom existe déjà, même s'il est apparu
// entre-temps, et le fichier final apparaît d'un coup, complet. Un arrêt brutal
// peut laisser le temporaire, jamais un fichier final incomplet. Un emplacement
// sans liens physiques (clé USB en FAT ou exFAT, certains partages réseau) est
// refusé : aucune publication de repli, moins sûre.

use std::fs::{self, File};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::{Erreur, Resultat};

// Empreinte SHA-256 en hexadécimal.
pub(crate) fn empreinte(contenu: &[u8]) -> String {
    Sha256::digest(contenu)
        .iter()
        .map(|octet| format!("{octet:02x}"))
        .collect()
}

pub(crate) fn empreinte_fichier(chemin: &Path) -> io::Result<String> {
    Ok(empreinte(&fs::read(chemin)?))
}

// Publie `temporaire` sous `destination`, puis retire le temporaire.
pub(crate) fn publier_sans_ecraser(temporaire: &Path, destination: &Path) -> io::Result<()> {
    let publication = fs::hard_link(temporaire, destination);
    let _ = fs::remove_file(temporaire);
    publication
}

// Message d'une publication impossible : la destination existe, ou
// l'emplacement ne permet pas de publier sans risque de fichier incomplet.
pub(crate) fn message_publication_impossible(destination: &Path, cause: &io::Error) -> String {
    if cause.kind() == ErrorKind::AlreadyExists {
        format!(
            "Un fichier existe déjà : {}. Choisissez un nouveau nom ; aucun fichier existant n'est remplacé.",
            destination.display()
        )
    } else {
        format!(
            "Publication impossible à cet emplacement : {} ({cause}). Choisissez un dossier sur un disque local (NTFS) ; les clés USB en FAT ou exFAT ne permettent pas une publication sûre.",
            destination.display()
        )
    }
}

// Copie un fichier du traitement vers l'emplacement choisi par l'utilisateur :
// copie complète dans « <destination>.copie-en-cours », vérification, puis
// publication sans écrasement. Refusée si la destination existe, même si
// l'utilisateur a confirmé le remplacement dans le dialogue. Si
// `empreinte_attendue` est fournie (SGX produit), la copie doit lui être
// identique : un fichier modifié depuis sa production n'est jamais copié.
pub fn copier_sans_ecraser(
    source: &Path,
    destination: &Path,
    empreinte_attendue: Option<&str>,
) -> Resultat<()> {
    let temporaire = nom_temporaire(destination, ".copie-en-cours");
    copier_exclusivement(source, &temporaire).map_err(|cause| {
        let message = if cause.kind() == ErrorKind::AlreadyExists {
            format!(
                "Une copie interrompue a laissé ce fichier : {}. Supprimez-le, puis recommencez.",
                temporaire.display()
            )
        } else {
            format!(
                "La copie n'a pas pu être enregistrée : {} ({cause})",
                destination.display()
            )
        };
        Erreur::nouvelle("copie_impossible", message)
    })?;
    if let Some(attendue) = empreinte_attendue {
        // Contrôle sur les octets de la copie elle-même, juste avant publication.
        if empreinte_fichier(&temporaire).ok().as_deref() != Some(attendue) {
            let _ = fs::remove_file(&temporaire);
            return Err(Erreur::nouvelle(
                "copie_refusee",
                format!(
                    "Ce fichier a été modifié depuis sa production et sa vérification : il n'est pas copié. {}",
                    source.display()
                ),
            ));
        }
    }
    publier_sans_ecraser(&temporaire, destination).map_err(|cause| {
        Erreur::nouvelle(
            "copie_impossible",
            message_publication_impossible(destination, &cause),
        )
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
// Utilisé uniquement pour écrire un temporaire, jamais un fichier final.
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

        let refus = copier_sans_ecraser(&source, &existant, None).unwrap_err();
        assert_eq!(refus.code, "copie_impossible");
        assert_eq!(fs::read(&existant).unwrap(), b"autre contenu");
        assert!(!nom_temporaire(&existant, ".copie-en-cours").exists());

        for alias in [
            source.clone(),
            racine.join("sorties/./export_modifie.sgx"),
            racine.join("sorties/../sorties/export_modifie.sgx"),
        ] {
            assert!(copier_sans_ecraser(&source, &alias, None).is_err());
            assert_eq!(fs::read(&source).unwrap(), b"sgx produit");
        }

        let copie = racine.join("copie.sgx");
        copier_sans_ecraser(&source, &copie, None).unwrap();
        assert_eq!(fs::read(&copie).unwrap(), b"sgx produit");
        assert!(!nom_temporaire(&copie, ".copie-en-cours").exists());

        // Dossier de destination absent : erreur, aucun fichier laissé.
        let impossible = racine.join("absent/copie.sgx");
        assert!(copier_sans_ecraser(&source, &impossible, None).is_err());
        assert!(!impossible.exists());

        // Erreur de lecture de la source (ici un dossier) : ni copie ni temporaire.
        let depuis_un_dossier = racine.join("depuis-un-dossier.sgx");
        assert!(copier_sans_ecraser(&racine.join("sorties"), &depuis_un_dossier, None).is_err());
        assert!(!depuis_un_dossier.exists());
        assert!(!nom_temporaire(&depuis_un_dossier, ".copie-en-cours").exists());

        // Temporaire laissé par une copie interrompue : jamais écrasé, signalé.
        let interrompue = racine.join("interrompue.sgx");
        let reste = nom_temporaire(&interrompue, ".copie-en-cours");
        fs::write(&reste, b"reste").unwrap();
        let refus = copier_sans_ecraser(&source, &interrompue, None).unwrap_err();
        assert!(refus.message.contains("Supprimez-le"));
        assert_eq!(fs::read(&reste).unwrap(), b"reste");
        assert!(!interrompue.exists());
        fs::remove_dir_all(&racine).unwrap();
    }

    // Revue F02 : un SGX modifié depuis sa production n'est jamais copié ; la
    // vérification porte sur la copie, avant publication.
    #[test]
    fn copie_refusee_si_l_empreinte_differe() {
        let racine = dossier_de_test("empreinte");
        let source = racine.join("sorties/export_modifie.sgx");
        fs::write(&source, b"sgx produit").unwrap();
        let attendue = empreinte(b"sgx produit");

        let copie = racine.join("copie.sgx");
        copier_sans_ecraser(&source, &copie, Some(&attendue)).unwrap();
        assert_eq!(fs::read(&copie).unwrap(), b"sgx produit");

        fs::write(&source, b"sgx altere").unwrap();
        let refusee = racine.join("refusee.sgx");
        let refus = copier_sans_ecraser(&source, &refusee, Some(&attendue)).unwrap_err();
        assert_eq!(refus.code, "copie_refusee");
        assert!(!refusee.exists());
        assert!(!nom_temporaire(&refusee, ".copie-en-cours").exists());
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
