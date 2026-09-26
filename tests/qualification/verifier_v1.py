"""Attentes V1 des 30 scénarios du pilote de qualification.

Le paquet docs/dossier-claude/references/qualification-rust-reproductible.zip
compare le binaire Rust au programme Python d'origine. Son check_qualification.py
décrit le comportement historique ; ce script décrit le contrat V1
(docs/m3-contrats-v1.md). Aucune donnée de référence n'est modifiée.

    python qualify_rust.py --root <paquet décompressé> --binary <bpmn-script-rs>
    python tests/qualification/verifier_v1.py <paquet décompressé>/rust-qualification-results.json
"""

import json
import sys

# Code de sortie V1 : 0 étape terminée, 1 erreur bloquante, 2 décisions fournies sans SGX.
CODES = {
    0: {"inventory", "dry_run", "accepted", "mixed", "normalized_yes", "empty_mapping",
        "unknown_mapping", "duplicate_mapping", "missing_output", "partial", "same_name",
        "zip_attributes"},
    1: {"no_source", "multiple_sources", "invalid_zip", "numeric_old", "numeric_new",
        "formula_new", "null_name", "bad_children"},
    2: {"refused", "pending", "stale_name", "stale_count", "stale_path", "stale_flux",
        "duplicate_validation", "chain_conflict", "duplicate_different", "old_output"},
}

PRODUIT_UN_SGX = {"accepted", "mixed", "normalized_yes", "partial", "same_name", "zip_attributes"}

# Écarts de contenu avec Python, tous volontaires (P02, P04, P05) ; tout autre
# écart est une régression.
ECARTS_AVEC_PYTHON = {
    "empty_mapping": "P05 : analyse vide, Python échoue",
    "unknown_mapping": "P05 : analyse vide, Python échoue",
    "missing_output": "P05 : dossier de sortie créé, Python échoue",
    "numeric_old": "P02 : classeur refusé",
    "numeric_new": "P02 : classeur refusé",
    "formula_new": "P02 : classeur refusé",
    "pending": "P02 : motif « valeur non reconnue » explicite",
}

resultats = {resultat["scenario"]: resultat for resultat in json.load(open(sys.argv[1], encoding="utf-8"))}
echecs = []


def verifier(condition, message):
    if not condition:
        echecs.append(message)


attendus = set().union(*CODES.values())
verifier(attendus <= resultats.keys(), f"scénarios manquants : {sorted(attendus - resultats.keys())}")

for code, scenarios in CODES.items():
    for nom in scenarios & resultats.keys():
        rust = resultats[nom]["rust"]
        verifier(rust["code"] == code, f"{nom} : code {rust['code']}, attendu {code}")
        verifier(rust["source_intact"], f"{nom} : source modifiée")
        nouveau_sgx = bool(rust["sgx_after"]) and rust["sgx_after"] != rust["sgx_before"]
        verifier(nouveau_sgx == (nom in PRODUIT_UN_SGX), f"{nom} : SGX produit = {nouveau_sgx}")
        ecart = bool(resultats[nom]["semantic_differences"])
        verifier(ecart == (nom in ECARTS_AVEC_PYTHON), f"{nom} : écart avec Python = {ecart} {resultats[nom]['semantic_differences']}")

controle = resultats["zip_attributes"]["rust"]["archive_check"]
verifier(controle["untouched_contents_preserved"], "zip_attributes : entrées non modifiées altérées")
verifier(controle["archive_comment_preserved"], "zip_attributes : commentaire d'archive perdu (P06)")
entree_modifiee = next(ecart for ecart in controle["attribute_changes"] if ecart["entry"] == "dossier/a/model_1_.json")
verifier(
    not {"extra", "comment", "date_time", "compress_type"} & set(entree_modifiee["changed_attributes"]),
    f"zip_attributes : attributs perdus sur l'entrée modifiée {entree_modifiee['changed_attributes']} (P06)",
)
verifier(
    resultats["old_output"]["rust"]["sgx_before"] == resultats["old_output"]["rust"]["sgx_after"],
    "old_output : ancienne sortie modifiée",
)

if echecs:
    print("Attentes V1 non respectées :")
    for echec in echecs:
        print(" -", echec)
    sys.exit(1)
print(f"Attentes V1 respectées : {len(attendus)} scénarios.")
