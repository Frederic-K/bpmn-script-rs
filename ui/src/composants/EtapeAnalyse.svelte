<script>
  import { textes } from "../textes.js";
  import Bouton from "./Bouton.svelte";
  import Chiffre from "./Chiffre.svelte";
  import Message from "./Message.svelte";
  import Tableau from "./Tableau.svelte";

  // Aperçu en lecture seule. Le classeur d'analyse n'est pas proposé à
  // l'ouverture : sa colonne Validation serait confondue avec le classeur de
  // décision. Sa copie sans réponse s'enregistre depuis l'étape Décisions.
  let { etat, naviguer, occupe } = $props();
  const texte = textes.analyse;

  // Tant que l'analyse n'est pas préparée, la liste vide ne signifie pas « aucun changement ».
  const propositions = $derived(etat.propositions);
  const aActualiser = $derived(etat.correspondances_a_relire);
  const occurrencesVisees = $derived(propositions.reduce((total, proposition) => total + proposition.occurrences, 0));
  const modelesConcernes = $derived(new Set(propositions.map((proposition) => proposition.fichier_modele)).size);
</script>

<h1 class="text-xl leading-snug font-semibold text-balance" tabindex="-1">{texte.titre}</h1>

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

    <div class="grid gap-4.5 {aActualiser ? 'opacity-60' : ''}" data-analyse={aActualiser ? "a-actualiser" : "actuelle"}>
      <div class="grid grid-cols-[repeat(auto-fit,minmax(130px,1fr))] gap-2.5">
        <Chiffre valeur={propositions.length} libelle={texte.propositions} />
        <Chiffre valeur={occurrencesVisees} libelle={texte.occurrences} />
        <Chiffre valeur={modelesConcernes} libelle={texte.modeles} />
      </div>

      <Tableau legende={texte.legende}>
        <thead>
          <tr>
            <th scope="col">{texte.colonneFlux}</th>
            <th scope="col">{texte.colonneNomActuel}</th>
            <th scope="col">{texte.colonneNouveauNom}</th>
            <th scope="col" class="text-right">{texte.colonneOccurrences}</th>
            <th scope="col">{texte.colonneFichier}</th>
          </tr>
        </thead>
        <tbody>
          {#each propositions as proposition}
            <tr>
              <td>{proposition.flux}</td>
              <td class="whitespace-pre-wrap">{proposition.nom_actuel}</td>
              <td class="font-semibold whitespace-pre-wrap">{proposition.nouveau_nom}</td>
              <td class="text-right tabular-nums">{proposition.occurrences}</td>
              <td class="min-w-[220px] font-mono text-xs text-encre-2 wrap-anywhere">{proposition.fichier_modele}</td>
            </tr>
          {/each}
        </tbody>
      </Tableau>
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <Bouton principal disabled={occupe || aActualiser} onclick={() => naviguer("decisions")}>{texte.suivant}</Bouton>
    </div>
  {/if}
{/if}
