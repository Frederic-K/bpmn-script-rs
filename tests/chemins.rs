//! Tests de la bibliothèque (lot M2) : dossiers passés explicitement,
//! indépendants du répertoire courant, messages reçus par le journal.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use bpmn_script_rs::{Chemins, executer};
use serde_json::json;
use zip::{ZipWriter, write::SimpleFileOptions};

struct Temp(PathBuf);

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp(nom: &str) -> Temp {
    let dossier = std::env::temp_dir().join(format!("bpmn-m2-{}-{nom}", std::process::id()));
    let _ = fs::remove_dir_all(&dossier);
    fs::create_dir_all(&dossier).unwrap();
    Temp(dossier)
}

fn ecrire_sgx(chemin: &PathBuf) {
    let mut zip = ZipWriter::new(File::create(chemin).unwrap());
    let options = SimpleFileOptions::default();
    let modele = json!({"childShapes": [
        {"stencil": {"id": "Lane"}, "properties": {"name": "A"}, "childShapes": []}
    ]});
    zip.start_file("m/model_1_.json", options).unwrap();
    zip.write_all(modele.to_string().as_bytes()).unwrap();
    zip.start_file("m/model_meta.json", options).unwrap();
    zip.write_all(br#"{"name": "Flux"}"#).unwrap();
    zip.finish().unwrap();
}

#[test]
fn dossiers_explicites_et_separes() {
    let t = temp("explicites");
    let chemins = Chemins {
        input: t.0.join("sources/export"),
        work: t.0.join("saisies"),
        output: t.0.join("resultats/lot 1"),
    };
    fs::create_dir_all(&chemins.input).unwrap();
    fs::create_dir_all(&chemins.work).unwrap();
    ecrire_sgx(&chemins.input.join("export.sgx"));

    let mut messages = Vec::new();
    executer(&chemins, &mut |m| messages.push(m.to_string())).unwrap();

    // Dossier de sortie imbriqué créé ; rien écrit ailleurs.
    let synthese: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(chemins.output.join("synthese.json")).unwrap())
            .unwrap();
    assert_eq!(synthese["A"]["occurrences"], 1);
    assert!(chemins.output.join("inventaire_swimlanes.xlsx").exists());
    assert_eq!(fs::read_dir(&chemins.work).unwrap().count(), 0);
    assert_eq!(fs::read_dir(&chemins.input).unwrap().count(), 1);

    assert_eq!(messages[0], "[OK] Fichier SGX sélectionné : export.sgx");
    assert_eq!(
        messages.last().unwrap(),
        "[INFO] Aucun fichier de correspondance trouvé : la préparation des modifications est ignorée"
    );
    assert!(messages.contains(&format!(
        "[OK] Excel généré : {}/inventaire_swimlanes.xlsx",
        chemins.output.display()
    )));
}

#[test]
fn erreur_nomme_le_dossier_source() {
    let t = temp("sans-source");
    let chemins = Chemins::historiques(&t.0);
    let erreur = executer(&chemins, &mut |_| {}).unwrap_err().to_string();
    assert_eq!(
        erreur,
        format!("Aucun fichier SGX trouvé dans {}/", chemins.input.display())
    );
    assert!(!chemins.output.exists());
}
