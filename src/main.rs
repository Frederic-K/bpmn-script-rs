//! bpmn-script-rs — CLI historique.
//!
//! À lancer depuis le dossier qui contient `input/`, `work/` et `output/` :
//! inventaire -> correspondance humaine -> dry-run -> validation humaine -> SGX modifié.

use std::path::Path;

use bpmn_script_rs::{Chemins, executer};

fn main() {
    let chemins = Chemins::historiques(Path::new(""));
    if let Err(e) = executer(&chemins, &mut |message| println!("{message}")) {
        eprintln!("[ERREUR] {e}");
        std::process::exit(1);
    }
}
