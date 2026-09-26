<script>
  import * as moteur from "../moteur.js";
  import { dateHeure, STATUTS } from "../format.js";
  import Bouton from "./Bouton.svelte";
  import Chiffre from "./Chiffre.svelte";
  import Fichier from "./Fichier.svelte";
  import Message from "./Message.svelte";
  import Pastille from "./Pastille.svelte";

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

<h1 class="text-xl leading-snug font-semibold text-balance" tabindex="-1">Résultat</h1>

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
      <Bouton disabled={occupe} onclick={() => executer("Enregistrement de la copie", () => moteur.enregistrerCopie(courante.sgx))}>
        Enregistrer une copie
      </Bouton>
    {/if}
  </Fichier>
  <div class="grid grid-cols-[repeat(auto-fit,minmax(130px,1fr))] gap-2.5">
    <Chiffre valeur={courante.modeles_modifies} libelle="modèles modifiés" />
    <Chiffre valeur={courante.occurrences_modifiees} libelle="occurrences renommées" />
    <Chiffre valeur={dateHeure(courante.horodatage)} libelle="tentative {courante.numero}" />
  </div>
  <div class="flex flex-wrap items-center gap-2">
    <Bouton principal disabled={occupe || courante.etat_sgx === "Absent"} onclick={() => executer("Ouverture du dossier", () => moteur.afficherDansDossier(courante.sgx))}>
      Afficher le SGX dans le dossier
    </Bouton>
    <Bouton disabled={occupe} onclick={() => executer("Ouverture du rapport", () => moteur.ouvrirFichier(rapport(courante)))}>
      Ouvrir le rapport de contrôle
    </Bouton>
  </div>
  <Message type="info" etiquette="Étape suivante" titre="Importez ce fichier dans un emplacement Signavio de test et vérifiez les modèles concernés avant tout import en production." />
{:else if courante}
  <Message type="alerte" etiquette="Aucun SGX" titre="Aucun nouveau SGX n'a été créé : {CAUSES[courante.statut] ?? STATUTS[courante.statut]?.libelle}">
    <Bouton onclick={() => naviguer("decisions")}>Revenir aux décisions</Bouton>
    <Bouton disabled={occupe} onclick={() => executer("Ouverture du rapport", () => moteur.ouvrirFichier(rapport(courante)))}>
      Ouvrir le rapport de contrôle
    </Bouton>
  </Message>
{:else}
  <p>Aucun résultat pour les correspondances et décisions actuellement lues.</p>
{/if}

{#if precedentes.length > 0}
  <section class="grid content-start gap-3 rounded-lg border border-trait p-4.5" aria-labelledby="titre-precedents">
    <h2 id="titre-precedents" class="text-base font-semibold">Résultats précédents</h2>
    <p class="text-xs text-encre-2">Non produits à partir des correspondances et décisions actuellement lues.</p>
    {#each precedentes as tentative}
      <div class="grid gap-1 border-t border-trait pt-2.5 text-[13px]">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <strong>Tentative {tentative.numero} · {dateHeure(tentative.horodatage)}</strong>
          <Pastille couleur={STATUTS[tentative.statut]?.couleur}>{STATUTS[tentative.statut]?.libelle ?? tentative.statut}</Pastille>
        </div>
        {#if tentative.sgx}
          <span class="text-xs text-encre-2">{tentative.sgx}{tentative.etat_sgx !== "Disponible" ? ` — ${ETATS_FICHIER[tentative.etat_sgx]}` : ""}</span>
        {/if}
      </div>
    {/each}
  </section>
{/if}
