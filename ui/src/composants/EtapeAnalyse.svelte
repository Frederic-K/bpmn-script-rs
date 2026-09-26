<script>
  import * as moteur from "../moteur.js";
  import Bouton from "./Bouton.svelte";
  import Chiffre from "./Chiffre.svelte";
  import Message from "./Message.svelte";
  import Tableau from "./Tableau.svelte";

  let { etat, executer, naviguer, occupe } = $props();

  const propositions = $derived(etat.propositions);
  const occurrencesVisees = $derived(propositions.reduce((total, proposition) => total + proposition.occurrences, 0));
  const modelesConcernes = $derived(new Set(propositions.map((proposition) => proposition.fichier_modele)).size);

  let copie = $state(null);

  async function enregistrerCopie() {
    const destination = await executer("Enregistrement de la copie", () => moteur.enregistrerCopie(etat.fichier_analyse));
    if (destination) copie = destination;
  }
</script>

<h1 class="text-xl leading-snug font-semibold text-balance" tabindex="-1">Analyse</h1>

{#if propositions.length === 0}
  <Message type="info" etiquette="Résultat" titre="Aucun changement proposé. Vérifiez les nouveaux noms renseignés : aucun nom actuel ne correspond à l'inventaire.">
    <Bouton onclick={() => naviguer("correspondances")}>Revenir aux correspondances</Bouton>
  </Message>
{:else}
  <p class="max-w-[72ch] text-encre-2">Aperçu en lecture seule des changements proposés. Rien n'est encore modifié.</p>

  <div class="grid grid-cols-[repeat(auto-fit,minmax(130px,1fr))] gap-2.5">
    <Chiffre valeur={propositions.length} libelle="propositions" />
    <Chiffre valeur={occurrencesVisees} libelle="occurrences visées" />
    <Chiffre valeur={modelesConcernes} libelle="modèles concernés" />
  </div>

  <Tableau legende="Une ligne par modèle et par nom ; des flux homonymes restent distincts par leur fichier modèle.">
    <thead>
      <tr>
        <th scope="col">Flux</th>
        <th scope="col">Nom actuel</th>
        <th scope="col">Nouveau nom</th>
        <th scope="col" class="text-right">Occ.</th>
        <th scope="col">Fichier modèle</th>
      </tr>
    </thead>
    <tbody>
      {#each propositions as proposition}
        <tr>
          <td>{proposition.flux}</td>
          <td class="whitespace-pre-wrap">{proposition.nom_actuel}</td>
          <td class="font-semibold whitespace-pre-wrap">{proposition.nouveau_nom}</td>
          <td class="text-right tabular-nums">{proposition.occurrences}</td>
          <td class="min-w-[220px] font-mono text-xs text-encre-2 [overflow-wrap:anywhere]">{proposition.fichier_modele}</td>
        </tr>
      {/each}
    </tbody>
  </Tableau>

  {#if copie}
    <Message type="ok" etiquette="Copie" titre="Copie enregistrée pour arbitrage : {copie}" />
  {/if}

  <div class="flex flex-wrap items-center gap-2">
    <Bouton principal disabled={occupe} onclick={() => naviguer("decisions")}>Passer aux décisions</Bouton>
    <Bouton disabled={occupe} onclick={() => executer("Ouverture de l'analyse", () => moteur.ouvrirFichier(etat.fichier_analyse))}>
      Ouvrir l'analyse
    </Bouton>
    <Bouton disabled={occupe} onclick={enregistrerCopie}>Enregistrer une copie pour arbitrage</Bouton>
  </div>
{/if}
