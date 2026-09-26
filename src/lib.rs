// bpmn-script-rs : moteur d'harmonisation des noms de swimlanes d'un SGX.
//
// Workflow : inventaire -> correspondance humaine -> dry-run -> validation
// humaine -> SGX modifié. Point d'entrée historique : workflow::executer.

mod excel;
mod regles;
mod sgx;
pub mod traitement;
pub mod workflow;

use std::fmt;

pub use regles::Modification;
pub use traitement::{Etape, Etat, EtatFichier, EtatTentative, Traitement};
pub use workflow::{Bilan, Chemins, LigneExaminee, Statut, executer};

// Erreur bloquante : code stable (pour l'interface), message lisible et
// détails éventuels (une ligne par cellule ou par modèle en anomalie).
#[derive(Debug)]
pub struct Erreur {
    pub code: &'static str,
    pub message: String,
    pub details: Vec<String>,
}

impl Erreur {
    pub(crate) fn nouvelle(code: &'static str, message: impl Into<String>) -> Erreur {
        Erreur {
            code,
            message: message.into(),
            details: Vec::new(),
        }
    }

    pub(crate) fn avec_details(mut self, details: Vec<String>) -> Erreur {
        self.details = details;
        self
    }
}

impl fmt::Display for Erreur {
    fn fmt(&self, sortie: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(sortie, "{}", self.message)?;
        for detail in &self.details {
            write!(sortie, "\n  - {detail}")?;
        }
        Ok(())
    }
}

// Toute erreur technique non qualifiée (lecture, écriture, format) devient une
// erreur « technique ». Les erreurs attendues passent par Contexte ou nouvelle.
impl<E: std::error::Error> From<E> for Erreur {
    fn from(erreur: E) -> Erreur {
        Erreur::nouvelle("technique", erreur.to_string())
    }
}

// Ajoute un code et un message métier à une erreur technique.
pub(crate) trait Contexte<T> {
    fn contexte(self, code: &'static str, message: &str) -> Resultat<T>;
}

impl<T, E: std::error::Error> Contexte<T> for Result<T, E> {
    fn contexte(self, code: &'static str, message: &str) -> Resultat<T> {
        self.map_err(|erreur| Erreur::nouvelle(code, format!("{message} ({erreur})")))
    }
}

pub type Resultat<T> = Result<T, Erreur>;

// Reçoit les messages de progression ([OK] ..., [ATTENTION] ..., [INFO] ...).
pub type Journal<'a> = dyn FnMut(&str) + 'a;
