<script>
  import { textes } from "../textes.js";
  import Bouton from "./Bouton.svelte";
  import Chiffre from "./Chiffre.svelte";
  import Message from "./Message.svelte";

  // Aperçu en compteurs ; le détail ligne par ligne est dans le classeur de décision.
  // Le classeur d'analyse n'est pas proposé à l'ouverture : sa colonne Validation
  // serait confondue avec le classeur de décision.
  let { etat, naviguer, occupe } = $props();
  const texte = textes.analyse;

  // Tant que l'analyse n'est pas préparée, la liste vide ne signifie pas « aucun changement ».
  const propositions = $derived(etat.propositions);
  const aActualiser = $derived(etat.correspondances_a_relire);
  const occurrencesVisees = $derived(propositions.reduce((total, proposition) => total + proposition.occurrences, 0));
  const modelesConcernes = $derived(new Set(propositions.map((proposition) => proposition.fichier_modele)).size);
</script>

{#if !etat.analyse_preparee}
  <Message type="info" etiquette={texte.etiquetteNonPreparee} titre={texte.nonPreparee}>
    <Bouton onclick={() => naviguer("correspondances")}>{texte.retourCorrespondances}</Bouton>
  </Message>
{:else}
  {#if aActualiser}
    <Message type="alerte" etiquette={texte.etiquetteAActualiser} titre={texte.aActualiser}>
      <Bouton onclick={() => naviguer("correspondances")}>{texte.retourCorrespondances}</Bouton>
    </Message>
  {/if}

  {#if propositions.length === 0}
    <Message type="info" etiquette={texte.etiquetteResultat} titre={texte.aucunChangement}>
      <Bouton onclick={() => naviguer("correspondances")}>{texte.retourCorrespondances}</Bouton>
    </Message>
  {:else}
    <p class="max-w-[72ch] text-encre-2">{texte.apercu}</p>

    <div
      class="grid grid-cols-[repeat(auto-fit,minmax(130px,1fr))] gap-2.5 {aActualiser ? 'opacity-60' : ''}"
      data-analyse={aActualiser ? "a-actualiser" : "actuelle"}
    >
      <Chiffre valeur={propositions.length} libelle={texte.propositions} />
      <Chiffre valeur={occurrencesVisees} libelle={texte.occurrences} />
      <Chiffre valeur={modelesConcernes} libelle={texte.modeles} />
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <Bouton principal disabled={occupe || aActualiser} onclick={() => naviguer("decisions")}>{texte.suivant}</Bouton>
    </div>
  {/if}
{/if}
