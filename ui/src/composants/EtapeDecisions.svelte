<script>
  import * as moteur from "../moteur.js";
  import { textes } from "../textes.js";
  import Bouton from "./Bouton.svelte";
  import Fichier from "./Fichier.svelte";
  import Message from "./Message.svelte";
  import Pastille from "./Pastille.svelte";
  import Tableau from "./Tableau.svelte";

  let { etat, executer, naviguer, occupe } = $props();
  const texte = textes.decisions;

  const bilan = $derived(etat.dernier_bilan);
  const aLire = $derived(!etat.decisions_adoptees || etat.decisions_a_relire);
  // Un classeur modifié sans être relu : les compteurs sont ceux du dernier contrôle, pas du contenu actuel.
  const aActualiser = $derived(etat.decisions_a_relire || etat.correspondances_a_relire);
  // Le moteur refait tous ces contrôles à la génération ; le bouton ne fait que les refléter.
  const generationPossible = $derived(
    etat.decisions_adoptees &&
      !aActualiser &&
      (bilan?.statut === "ProductionPossible" || bilan?.statut === "SgxProduit"),
  );

  function lire(fichier = null) {
    return executer(textes.actions.lectureDecisions, () => moteur.lireDecisions(etat.revision, fichier));
  }

  async function importer() {
    const fichier = await executer(textes.actions.choixClasseur, moteur.choisirClasseur);
    if (fichier) await lire(fichier);
  }

  async function generer() {
    const reponse = await executer(textes.actions.generation, () => moteur.produire(etat.revision));
    if (reponse) naviguer("resultat");
  }
</script>

<h1 class="text-xl leading-snug font-semibold text-balance" tabindex="-1">{texte.titre}</h1>
<p class="max-w-[72ch] text-encre-2">{texte.consigne}</p>

{#if etat.edition_decisions}
  <Fichier type="XLSX" chemin={etat.edition_decisions}>
    <Bouton disabled={occupe} onclick={() => executer(textes.actions.ouvertureClasseur, () => moteur.ouvrirFichier(etat.edition_decisions))}>
      {texte.ouvrir}
    </Bouton>
  </Fichier>
{/if}

{#if etat.decisions_adoptees && etat.decisions_a_relire}
  <Message type="alerte" etiquette={texte.etiquetteARelire} titre={texte.aRelire} />
{:else if etat.correspondances_a_relire}
  <Message type="alerte" etiquette={texte.etiquetteARelire} titre={texte.correspondancesARelire} />
{:else if aLire}
  <Message type="info" etiquette={texte.etiquetteInfo} titre={texte.fermerAvantLecture} />
{/if}

<div class="flex flex-wrap items-center gap-2">
  <Bouton principal={aLire} disabled={occupe} onclick={() => lire()}>
    {etat.decisions_adoptees ? texte.relire : texte.lire}
  </Bouton>
  <Bouton disabled={occupe} onclick={importer}>{texte.importer}</Bouton>
</div>

{#if etat.decisions_adoptees && bilan}
  <section class="grid content-start gap-3" aria-labelledby="titre-controle" data-controle={aActualiser ? "a-actualiser" : "actuel"}>
    <h2 id="titre-controle" class="text-base font-semibold {aActualiser ? 'text-alerte' : ''}">
      {aActualiser ? texte.dernierControleAActualiser : texte.dernierControle}
    </h2>
    <div
      class="grid grid-cols-[repeat(auto-fit,minmax(112px,1fr))] gap-2.5
        [&>div]:grid [&>div]:content-start [&>div]:gap-1 [&>div]:rounded-md [&>div]:border [&>div]:border-trait [&>div]:px-3 [&>div]:py-2.5
        [&_b]:text-[22px] [&_b]:leading-tight [&_b]:tabular-nums [&_small]:text-xs [&_small]:text-encre-2
        {aActualiser ? 'opacity-60' : ''}"
    >
      <div><Pastille couleur="ok">{texte.admises}</Pastille><b>{bilan.lignes_admises}</b><small>{texte.occurrencesPrevues(bilan.occurrences_admises)}</small></div>
      <div><Pastille>{texte.refusees}</Pastille><b>{bilan.lignes_refusees}</b><small>{texte.refuseesDetail}</small></div>
      <div><Pastille couleur="alerte">{texte.ignorees}</Pastille><b>{bilan.lignes_ignorees}</b><small>{texte.ignoreesDetail}</small></div>
      <div><Pastille>{texte.enAttente}</Pastille><b>{bilan.lignes_en_attente}</b><small>{texte.enAttenteDetail}</small></div>
      <div><Pastille>{texte.sansDecision}</Pastille><b>{bilan.propositions_sans_decision}</b><small>{texte.sansDecisionDetail}</small></div>
    </div>

    {#if bilan.lignes_a_examiner.length > 0}
      <Tableau legende={texte.legende}>
        <thead>
          <tr>
            <th scope="col" class="text-right">{texte.colonneLigne}</th>
            <th scope="col">{texte.colonneNomActuel}</th>
            <th scope="col">{texte.colonneNouveauNom}</th>
            <th scope="col">{texte.colonneResultat}</th>
            <th scope="col">{texte.colonneMotif}</th>
          </tr>
        </thead>
        <tbody>
          {#each bilan.lignes_a_examiner as ligne}
            <tr>
              <td class="text-right tabular-nums">{ligne.ligne}</td>
              <td class="whitespace-pre-wrap">{ligne.nom_actuel}</td>
              <td class="font-semibold whitespace-pre-wrap">{ligne.nouveau_nom}</td>
              <td><Pastille couleur={ligne.resultat === "IGNORÉE" ? "alerte" : "neutre"}>{ligne.resultat}</Pastille></td>
              <td>{ligne.motif}</td>
            </tr>
          {/each}
        </tbody>
      </Tableau>
    {/if}

    {#if bilan.statut === "ControleBloquant"}
      <Message type="erreur" etiquette={texte.etiquetteBloquant} titre={texte.bloquant} details={bilan.divergences} />
    {:else if bilan.statut === "AucuneDecisionAdmissible"}
      <Message type="alerte" etiquette={texte.etiquetteAucunSgx} titre={texte.aucuneAdmissible} />
    {:else if generationPossible}
      <Message type="ok" etiquette={texte.etiquetteControle} titre={texte.controle} />
    {/if}
  </section>

  <div class="flex flex-wrap items-center gap-2">
    <Bouton principal disabled={occupe || !generationPossible} onclick={generer}>{texte.generer}</Bouton>
    <Bouton disabled={occupe} onclick={() => executer(textes.actions.ouvertureRapport, () => moteur.ouvrirFichier(etat.fichier_controle))}>
      {texte.ouvrirRapport}
    </Bouton>
  </div>
{/if}
