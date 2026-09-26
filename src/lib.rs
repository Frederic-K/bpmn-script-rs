//! bpmn-script-rs — moteur d'harmonisation des noms de swimlanes d'un SGX.
//!
//! Workflow : inventaire -> correspondance humaine -> dry-run -> validation
//! humaine -> SGX modifié. Le point d'entrée est [`workflow::executer`].

mod excel;
mod regles;
mod sgx;
pub mod workflow;

pub use workflow::{Chemins, executer};

/// Résultat d'une opération ; l'erreur porte un message lisible.
pub type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// Reçoit les messages de progression (`[OK] ...`, `[ATTENTION] ...`, `[INFO] ...`).
pub type Journal<'a> = dyn FnMut(&str) + 'a;
