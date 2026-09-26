<script>
  import * as moteur from "../moteur.js";
  import { dateHeure, STATUTS } from "../format.js";
  import Fichier from "./Fichier.svelte";
  import Message from "./Message.svelte";

  let { etat, executer, naviguer, occupe } = $props();

  // La tentative courante porte sur les entrées actuellement lues ; les autres sont des résultats précédents.
  const courante = $derived(etat.tentatives.findLast((tentative) => tentative.courante));
  const precedentes = $derived(etat.tentatives.filter((tentative) => tentative !== courante).reverse());

  const ETATS_FICHIER = {
    Absent: "Ce fichier a été supprimé depuis sa production.",
    Modifie: "Ce fichier a été modifié depuis sa production : il ne correspond plus au résultat vérifié.",
  };

  const CAUSES = {
    AucuneDecisionAdmissible: "aucune décision admissible (lignes refusées, en attente ou non conformes).",
    ControleBloquant: "le recomptage des occurrences ne correspond pas aux décisions.",
  };

  function rapport(tentative) {
    return `${tentative.dossier}/controle_validation.xlsx`;
  }
</script>

<h1 tabindex="-1">Résultat</h1>

{#each etat.tentatives_interrompues as dossier}
  <Message type="alerte" etiquette="Interrompue" titre="Une production a été interrompue avant d'être enregistrée. Son contenu n'est pas confirmé et n'est jamais présenté comme un résultat :" details={[dossier]} />
{/each}

{#if courante?.sgx}
  {#if courante.etat_sgx === "Disponible"}
    <Message type="ok" etiquette="Produit" titre="SGX produit et vérifié : relu après écriture, seules les lanes validées diffèrent de la source." />
  {:else}
    <Message type="erreur" etiquette="Attention" titre={ETATS_FICHIER[courante.etat_sgx]} />
  {/if}
  <Fichier type="SGX" chemin={courante.sgx}>
    {#if courante.etat_sgx === "Disponible"}
      <button class="bouton" type="button" disabled={occupe} onclick={() => executer("Enregistrement de la copie", () => moteur.enregistrerCopie(courante.sgx))}>
        Enregistrer une copie
      </button>
    {/if}
  </Fichier>
  <div class="chiffres">
    <div class="chiffre"><b>{courante.modeles_modifies}</b><span>modèles modifiés</span></div>
    <div class="chiffre"><b>{courante.occurrences_modifiees}</b><span>occurrences renommées</span></div>
    <div class="chiffre"><b>{dateHeure(courante.horodatage)}</b><span>tentative {courante.numero}</span></div>
  </div>
  <div class="rangee">
    <button class="bouton principal" type="button" disabled={occupe || courante.etat_sgx === "Absent"} onclick={() => executer("Ouverture du dossier", () => moteur.afficherDansDossier(courante.sgx))}>
      Afficher le SGX dans le dossier
    </button>
    <button class="bouton" type="button" disabled={occupe} onclick={() => executer("Ouverture du rapport", () => moteur.ouvrirFichier(rapport(courante)))}>
      Ouvrir le rapport de contrôle
    </button>
  </div>
  <Message type="info" etiquette="Étape suivante" titre="Importez ce fichier dans un emplacement Signavio de test et vérifiez les modèles concernés avant tout import en production." />
{:else if courante}
  <Message type="alerte" etiquette="Aucun SGX" titre="Aucun nouveau SGX n'a été créé : {CAUSES[courante.statut] ?? STATUTS[courante.statut]?.libelle}">
    <button class="bouton" type="button" onclick={() => naviguer("decisions")}>Revenir aux décisions</button>
    <button class="bouton" type="button" disabled={occupe} onclick={() => executer("Ouverture du rapport", () => moteur.ouvrirFichier(rapport(courante)))}>
      Ouvrir le rapport de contrôle
    </button>
  </Message>
{:else}
  <p>Aucun résultat pour les correspondances et décisions actuellement lues.</p>
{/if}

{#if precedentes.length > 0}
  <section class="choix" aria-labelledby="titre-precedents">
    <h2 id="titre-precedents">Résultats précédents</h2>
    <p class="petit">Non produits à partir des correspondances et décisions actuellement lues.</p>
    {#each precedentes as tentative}
      <div class="tentative">
        <div class="ligne">
          <strong>Tentative {tentative.numero} · {dateHeure(tentative.horodatage)}</strong>
          <span class="pastille {STATUTS[tentative.statut]?.couleur ?? 'neutre'}">{STATUTS[tentative.statut]?.libelle ?? tentative.statut}</span>
        </div>
        {#if tentative.sgx}
          <span class="petit">{tentative.sgx}{tentative.etat_sgx !== "Disponible" ? ` — ${ETATS_FICHIER[tentative.etat_sgx]}` : ""}</span>
        {/if}
      </div>
    {/each}
  </section>
{/if}
