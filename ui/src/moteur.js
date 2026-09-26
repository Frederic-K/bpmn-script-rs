// Appels au moteur Rust (commandes de src-tauri/src/lib.rs).
// Une opération qui modifie le traitement retourne { etat, bilan }.
// Une erreur rejetée est un objet { code, message, details }.

import { invoke } from "@tauri-apps/api/core";

export const etat = () => invoke("etat");

export const creerTraitement = (source, dossierParent) =>
  invoke("creer_traitement", { source, dossierParent });

export const ouvrirTraitement = (dossier) => invoke("ouvrir_traitement", { dossier });

export const fermerTraitement = () => invoke("fermer_traitement");

// fichier = null : lit le classeur d'édition du traitement.
export const lireCorrespondances = (revision, fichier = null) =>
  invoke("lire_correspondances", { revision, fichier });

export const preparerAnalyse = (revision) => invoke("preparer_analyse", { revision });

export const lireDecisions = (revision, fichier = null) =>
  invoke("lire_decisions", { revision, fichier });

export const produire = (revision) => invoke("produire", { revision });

// Dialogues : retournent un chemin, ou null si l'utilisateur annule.
export const choisirSgx = () => invoke("choisir_sgx");

export const choisirDossier = (titre) => invoke("choisir_dossier", { titre });

export const choisirClasseur = () => invoke("choisir_classeur");

// Fichiers du traitement ouvert uniquement (contrôlé côté Rust).
export const ouvrirFichier = (chemin) => invoke("ouvrir_fichier", { chemin });

export const afficherDansDossier = (chemin) => invoke("afficher_dans_dossier", { chemin });

export const enregistrerCopie = (chemin) => invoke("enregistrer_copie", { chemin });
