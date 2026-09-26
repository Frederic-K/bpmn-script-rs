// Adaptateur Tauri : une commande par opération du moteur. Aucune règle métier
// ici ; le moteur refait tous les contrôles, quelle que soit l'interface.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use bpmn_script_rs::{Bilan, Erreur, Etat, Traitement};
use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

// Traitement ouvert dans la fenêtre. Le verrou n'est jamais attendu : une
// demande reçue pendant une opération est refusée (double clic, deux actions).
#[derive(Default)]
struct TraitementOuvert(Mutex<Option<Traitement>>);

#[derive(Serialize, Debug)]
struct ErreurInterface {
    code: String,
    message: String,
    details: Vec<String>,
}

impl From<Erreur> for ErreurInterface {
    fn from(erreur: Erreur) -> ErreurInterface {
        ErreurInterface {
            code: erreur.code.to_string(),
            message: erreur.message,
            details: erreur.details,
        }
    }
}

fn erreur(code: &str, message: impl Into<String>) -> ErreurInterface {
    ErreurInterface {
        code: code.to_string(),
        message: message.into(),
        details: Vec::new(),
    }
}

type Reponse<T> = Result<T, ErreurInterface>;

#[derive(Serialize)]
struct Resultat {
    etat: Etat,
    bilan: Option<Bilan>,
}

// Exécute `operation` sur le traitement ouvert, puis retourne son nouvel état.
fn avec_traitement(
    ouvert: &TraitementOuvert,
    operation: impl FnOnce(&mut Traitement) -> Result<Option<Bilan>, Erreur>,
) -> Reponse<Resultat> {
    let mut garde = ouvert.0.try_lock().map_err(|_| {
        erreur(
            "operation_en_cours",
            "Une opération est déjà en cours : attendez qu'elle se termine.",
        )
    })?;
    let traitement = garde
        .as_mut()
        .ok_or_else(|| erreur("aucun_traitement", "Aucun traitement n'est ouvert."))?;
    let bilan = operation(traitement)?;
    Ok(Resultat {
        etat: traitement.etat()?,
        bilan,
    })
}

// Remplace le traitement ouvert (l'ancien est fermé et son verrou libéré).
fn remplacer(
    ouvert: &TraitementOuvert,
    ouvrir: impl FnOnce() -> Result<(Traitement, Option<Bilan>), Erreur>,
) -> Reponse<Resultat> {
    let mut garde = ouvert.0.try_lock().map_err(|_| {
        erreur(
            "operation_en_cours",
            "Une opération est déjà en cours : attendez qu'elle se termine.",
        )
    })?;
    // Fermé avant d'ouvrir : rouvrir le même dossier ne doit pas buter sur son propre verrou.
    *garde = None;
    let (traitement, bilan) = ouvrir()?;
    let etat = traitement.etat()?;
    *garde = Some(traitement);
    Ok(Resultat { etat, bilan })
}

fn journal(message: &str) {
    println!("{message}");
}

// ---------------------------------------------------------------- Traitement

#[tauri::command]
async fn etat(ouvert: State<'_, TraitementOuvert>) -> Reponse<Option<Etat>> {
    let garde = ouvert
        .0
        .try_lock()
        .map_err(|_| erreur("operation_en_cours", "Une opération est déjà en cours."))?;
    match garde.as_ref() {
        Some(traitement) => Ok(Some(traitement.etat()?)),
        None => Ok(None),
    }
}

#[tauri::command]
async fn creer_traitement(
    ouvert: State<'_, TraitementOuvert>,
    source: PathBuf,
    dossier_parent: PathBuf,
) -> Reponse<Resultat> {
    remplacer(&ouvert, || {
        let (traitement, bilan) = Traitement::creer(&source, &dossier_parent, &mut journal)?;
        Ok((traitement, Some(bilan)))
    })
}

#[tauri::command]
async fn ouvrir_traitement(
    ouvert: State<'_, TraitementOuvert>,
    dossier: PathBuf,
) -> Reponse<Resultat> {
    remplacer(&ouvert, || Ok((Traitement::ouvrir(&dossier)?, None)))
}

#[tauri::command]
async fn fermer_traitement(ouvert: State<'_, TraitementOuvert>) -> Reponse<()> {
    let mut garde = ouvert
        .0
        .try_lock()
        .map_err(|_| erreur("operation_en_cours", "Une opération est déjà en cours."))?;
    *garde = None;
    Ok(())
}

#[tauri::command]
async fn lire_correspondances(
    ouvert: State<'_, TraitementOuvert>,
    revision: u64,
    fichier: Option<PathBuf>,
) -> Reponse<Resultat> {
    avec_traitement(&ouvert, |traitement| {
        traitement
            .adopter_correspondances(revision, fichier.as_deref(), &mut journal)
            .map(Some)
    })
}

#[tauri::command]
async fn preparer_analyse(ouvert: State<'_, TraitementOuvert>, revision: u64) -> Reponse<Resultat> {
    avec_traitement(&ouvert, |traitement| {
        traitement
            .preparer_analyse(revision, &mut journal)
            .map(Some)
    })
}

#[tauri::command]
async fn lire_decisions(
    ouvert: State<'_, TraitementOuvert>,
    revision: u64,
    fichier: Option<PathBuf>,
) -> Reponse<Resultat> {
    avec_traitement(&ouvert, |traitement| {
        traitement
            .adopter_decisions(revision, fichier.as_deref(), &mut journal)
            .map(Some)
    })
}

#[tauri::command]
async fn produire(ouvert: State<'_, TraitementOuvert>, revision: u64) -> Reponse<Resultat> {
    avec_traitement(&ouvert, |traitement| {
        traitement.produire(revision, &mut journal).map(Some)
    })
}

// ---------------------------------------------------------------- Fichiers

// Les dialogues bloquants s'exécutent hors du fil principal (commandes async).

// Les sélecteurs s'ouvrent dans Documents (à défaut le dossier personnel)
// plutôt que dans « Récents », où la saisie d'un chemin devient une recherche.
fn dialogue(app: &AppHandle, titre: &str) -> tauri_plugin_dialog::FileDialogBuilder<tauri::Wry> {
    let depart = app
        .path()
        .document_dir()
        .ok()
        .filter(|dossier| dossier.is_dir())
        .or_else(|| app.path().home_dir().ok());
    let dialogue = app.dialog().file().set_title(titre);
    match depart {
        Some(dossier) => dialogue.set_directory(dossier),
        None => dialogue,
    }
}

#[tauri::command]
async fn choisir_sgx(app: AppHandle) -> Reponse<Option<PathBuf>> {
    let choix = dialogue(&app, "Choisir un export Signavio")
        .add_filter("Export Signavio (.sgx)", &["sgx"])
        .blocking_pick_file();
    Ok(choix.and_then(|chemin| chemin.into_path().ok()))
}

#[tauri::command]
async fn choisir_dossier(app: AppHandle, titre: String) -> Reponse<Option<PathBuf>> {
    let choix = dialogue(&app, &titre).blocking_pick_folder();
    Ok(choix.and_then(|chemin| chemin.into_path().ok()))
}

#[tauri::command]
async fn choisir_classeur(app: AppHandle) -> Reponse<Option<PathBuf>> {
    let choix = dialogue(&app, "Choisir un classeur Excel")
        .add_filter("Classeur Excel (.xlsx)", &["xlsx"])
        .blocking_pick_file();
    Ok(choix.and_then(|chemin| chemin.into_path().ok()))
}

// Seuls les fichiers du traitement ouvert peuvent être ouverts, affichés ou copiés.
fn fichier_du_traitement(ouvert: &TraitementOuvert, chemin: &Path) -> Reponse<PathBuf> {
    let garde = ouvert
        .0
        .try_lock()
        .map_err(|_| erreur("operation_en_cours", "Une opération est déjà en cours."))?;
    let traitement = garde
        .as_ref()
        .ok_or_else(|| erreur("aucun_traitement", "Aucun traitement n'est ouvert."))?;
    verifier_appartenance(traitement.dossier(), chemin)
}

fn verifier_appartenance(dossier: &Path, chemin: &Path) -> Reponse<PathBuf> {
    let introuvable = || {
        erreur(
            "fichier_absent",
            format!("Fichier introuvable : {}", chemin.display()),
        )
    };
    // Comparaison sur les chemins canoniques ; le chemin d'origine est retourné,
    // car la forme canonique Windows (\\?\C:\...) n'est pas comprise par
    // toutes les applications (Excel, Explorateur).
    let dossier_canonique = dossier.canonicalize().map_err(|_| introuvable())?;
    let chemin_canonique = chemin.canonicalize().map_err(|_| introuvable())?;
    if !chemin_canonique.starts_with(&dossier_canonique) {
        return Err(erreur(
            "fichier_refuse",
            "Seuls les fichiers du traitement ouvert peuvent être ouverts.",
        ));
    }
    Ok(chemin.to_path_buf())
}

#[tauri::command]
async fn ouvrir_fichier(
    app: AppHandle,
    ouvert: State<'_, TraitementOuvert>,
    chemin: PathBuf,
) -> Reponse<()> {
    let chemin = fichier_du_traitement(&ouvert, &chemin)?;
    app.opener().open_path(chemin.to_string_lossy(), None::<&str>).map_err(|_| {
        erreur(
            "ouverture_impossible",
            format!(
                "Aucune application n'a pu ouvrir ce fichier. Utilisez « Afficher dans le dossier » : {}",
                chemin.display()
            ),
        )
    })
}

#[tauri::command]
async fn afficher_dans_dossier(
    app: AppHandle,
    ouvert: State<'_, TraitementOuvert>,
    chemin: PathBuf,
) -> Reponse<()> {
    let chemin = fichier_du_traitement(&ouvert, &chemin)?;
    app.opener().reveal_item_in_dir(&chemin).map_err(|_| {
        erreur(
            "ouverture_impossible",
            format!("Impossible d'afficher le dossier : {}", chemin.display()),
        )
    })
}

// Enregistre une copie d'un fichier du traitement (classeur pour un arbitre,
// SGX produit) à l'emplacement choisi. L'original reste dans le traitement.
#[tauri::command]
async fn enregistrer_copie(
    app: AppHandle,
    ouvert: State<'_, TraitementOuvert>,
    chemin: PathBuf,
) -> Reponse<Option<PathBuf>> {
    let chemin = fichier_du_traitement(&ouvert, &chemin)?;
    let nom = chemin
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let Some(destination) = dialogue(&app, "Enregistrer une copie")
        .set_file_name(&nom)
        .blocking_save_file()
        .and_then(|choix| choix.into_path().ok())
    else {
        return Ok(None);
    };
    copier_sans_ecraser(&chemin, &destination)?;
    Ok(Some(destination))
}

// La destination est créée exclusivement : un fichier existant n'est jamais
// remplacé, même si l'utilisateur a confirmé le remplacement dans le dialogue.
// Cela couvre aussi la copie vers le fichier lui-même, quel que soit le chemin
// utilisé pour le désigner. Une copie interrompue est supprimée.
fn copier_sans_ecraser(source: &Path, destination: &Path) -> Reponse<()> {
    let echec = |cause: std::io::Error| {
        erreur(
            "copie_impossible",
            format!(
                "La copie n'a pas pu être enregistrée : {} ({cause})",
                destination.display()
            ),
        )
    };
    let mut lecture = std::fs::File::open(source).map_err(echec)?;
    let mut copie = match std::fs::File::create_new(destination) {
        Ok(copie) => copie,
        Err(cause) if cause.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(erreur(
                "copie_impossible",
                format!(
                    "Un fichier existe déjà à cet emplacement : {}. Choisissez un autre nom ; aucun fichier n'est remplacé.",
                    destination.display()
                ),
            ));
        }
        Err(cause) => return Err(echec(cause)),
    };
    let resultat = std::io::copy(&mut lecture, &mut copie).and_then(|_| copie.sync_all());
    drop(copie);
    if let Err(cause) = resultat {
        let _ = std::fs::remove_file(destination);
        return Err(echec(cause));
    }
    Ok(())
}

pub fn lancer() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(TraitementOuvert::default())
        .invoke_handler(tauri::generate_handler![
            etat,
            creer_traitement,
            ouvrir_traitement,
            fermer_traitement,
            lire_correspondances,
            preparer_analyse,
            lire_decisions,
            produire,
            choisir_sgx,
            choisir_dossier,
            choisir_classeur,
            ouvrir_fichier,
            afficher_dans_dossier,
            enregistrer_copie,
        ])
        .run(tauri::generate_context!())
        .expect("impossible de démarrer BPMN-Script");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seuls_les_fichiers_du_traitement_sont_acceptes() {
        let racine = std::env::temp_dir().join(format!("bpmn-app-{}", std::process::id()));
        let traitement = racine.join("traitement");
        std::fs::create_dir_all(traitement.join("edition")).unwrap();
        std::fs::write(traitement.join("edition/classeur.xlsx"), b"x").unwrap();
        std::fs::write(racine.join("dehors.xlsx"), b"x").unwrap();

        assert!(
            verifier_appartenance(&traitement, &traitement.join("edition/classeur.xlsx")).is_ok()
        );
        let refus = verifier_appartenance(&traitement, &racine.join("dehors.xlsx")).unwrap_err();
        assert_eq!(refus.code, "fichier_refuse");
        let refus =
            verifier_appartenance(&traitement, &traitement.join("edition/../../dehors.xlsx"))
                .unwrap_err();
        assert_eq!(refus.code, "fichier_refuse");
        let refus =
            verifier_appartenance(&traitement, &traitement.join("absent.xlsx")).unwrap_err();
        assert_eq!(refus.code, "fichier_absent");
        std::fs::remove_dir_all(&racine).unwrap();
    }

    // Revue D01 : la copie ne remplace jamais un fichier, y compris la source
    // désignée par un autre chemin.
    #[test]
    fn copie_sans_ecrasement() {
        let racine = std::env::temp_dir().join(format!("bpmn-copie-{}", std::process::id()));
        std::fs::create_dir_all(racine.join("sorties")).unwrap();
        let source = racine.join("sorties/export_modifie.sgx");
        let existant = racine.join("existant.sgx");
        std::fs::write(&source, b"sgx produit").unwrap();
        std::fs::write(&existant, b"autre contenu").unwrap();

        let refus = copier_sans_ecraser(&source, &existant).unwrap_err();
        assert_eq!(refus.code, "copie_impossible");
        assert_eq!(std::fs::read(&existant).unwrap(), b"autre contenu");

        for alias in [
            source.clone(),
            racine.join("sorties/./export_modifie.sgx"),
            racine.join("sorties/../sorties/export_modifie.sgx"),
        ] {
            assert_eq!(
                copier_sans_ecraser(&source, &alias).unwrap_err().code,
                "copie_impossible"
            );
            assert_eq!(std::fs::read(&source).unwrap(), b"sgx produit");
        }

        let copie = racine.join("copie.sgx");
        copier_sans_ecraser(&source, &copie).unwrap();
        assert_eq!(std::fs::read(&copie).unwrap(), b"sgx produit");

        // Dossier de destination absent : erreur, et aucun fichier laissé.
        let impossible = racine.join("absent/copie.sgx");
        assert!(copier_sans_ecraser(&source, &impossible).is_err());
        assert!(!impossible.exists());
        std::fs::remove_dir_all(&racine).unwrap();
    }
}
