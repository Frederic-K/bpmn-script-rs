// Pas de console supplémentaire sous Windows pour l'application compilée.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    bpmn_script_app::lancer();
}
