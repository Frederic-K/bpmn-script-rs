<script>
  import * as moteur from "../moteur.js";
  import { dateHeure, nomFichier } from "../format.js";
  import { textes } from "../textes.js";
  import Bouton from "./Bouton.svelte";
  import Fichier from "./Fichier.svelte";
  import Message from "./Message.svelte";
  import Pastille from "./Pastille.svelte";

  // Deux voies, un seul classeur utilisé : le classeur de décision du
  // traitement. Un retour importé le remplace entièrement (pas de fusion).
  let { etat, executer, naviguer, occupe } = $props();
  const texte = textes.decisions;

  const bilan = $derived(etat.dernier_bilan);
  const lues = $derived(etat.decisions_lues);
  // Un classeur modifié sans être relu : les compteurs sont ceux du dernier contrôle, pas du contenu actuel.
  const aActualiser = $derived(etat.decisions_a_relire || etat.correspondances_a_relire);
  // Le moteur refait tous ces contrôles à la génération ; le bouton ne fait que les refléter.
  const generationPossible = $derived(
    etat.decisions_adoptees &&
      !aActualiser &&
      (bilan?.statut === "ProductionPossible" || bilan?.statut === "SgxProduit"),
  );

  let lectureRefusee = $state(false);
  let copie = $state(null);

  async function lire(fichier = null) {
    const reponse = await executer(textes.actions.lectureDecisions, () => moteur.lireDecisions(etat.revision, fichier));
    lectureRefusee = !reponse;
  }

  async function importer() {
    const fichier = await executer(textes.actions.choixClasseur, moteur.choisirClasseur);
    if (fichier) await lire(fichier);
  }

  async function enregistrerCopie() {
    const destination = await executer(textes.actions.copie, () => moteur.enregistrerCopie(etat.fichier_analyse, texte.nomCopie(etat.nom)));
    if (destination) copie = destination;
  }

  async function generer() {
    const reponse = await executer(textes.actions.generation, () => moteur.produire(etat.revision));
    if (reponse) naviguer("resultat");
  }
</script>

<h1 class="text-xl leading-snug font-semibold text-balance" tabindex="-1">{texte.titre}</h1>
<p class="max-w-[72ch] text-encre-2">{texte.consigne}</p>

{#if etat.correspondances_a_relire}
  <Message type="alerte" etiquette={texte.etiquetteARelire} titre={texte.correspondancesARelire} />
{:else if etat.decisions_a_relire}
  <Message type="alerte" etiquette={texte.etiquetteARelire} titre={texte.aRelire} />
{/if}

<div class="grid grid-cols-[repeat(auto-fit,minmax(300px,1fr))] gap-4">
  <section class="grid content-start gap-3 rounded-lg border border-trait p-4.5" aria-labelledby="titre-local">
    <h2 id="titre-local" class="text-base font-semibold">{texte.localTitre}</h2>
    <p class="text-[13px] text-encre-2">{texte.localConsigne}</p>
    {#if etat.edition_decisions}
      <Fichier type="XLSX" chemin={etat.edition_decisions}>
        <Bouton disabled={occupe} onclick={() => executer(textes.actions.ouvertureClasseur, () => moteur.ouvrirFichier(etat.edition_decisions))}>
          {texte.ouvrir}
        </Bouton>
      </Fichier>
    {/if}
    <div><Bouton principal={!etat.decisions_adoptees || etat.decisions_a_relire} disabled={occupe} onclick={() => lire()}>{texte.lire}</Bouton></div>
  </section>

  <section class="grid content-start gap-3 rounded-lg border border-trait p-4.5" aria-labelledby="titre-externe">
    <h2 id="titre-externe" class="text-base font-semibold">{texte.externeTitre}</h2>
    <p class="text-[13px] text-encre-2">{texte.externeConsigne}</p>
    <div><Bouton disabled={occupe || !etat.fichier_analyse} onclick={enregistrerCopie}>{texte.copier}</Bouton></div>
    <p class="text-xs text-encre-2">{texte.copieSansEcrasement}</p>
    {#if copie}
      <Message type="ok" etiquette={texte.etiquetteCopie} titre={texte.copieEnregistree(copie)} />
    {/if}
    <p class="text-[13px] text-encre-2">{texte.importerConsigne}</p>
    <div><Bouton disabled={occupe} onclick={importer}>{texte.importer}</Bouton></div>
  </section>
</div>

{#if lectureRefusee && lues}
  <Message type="alerte" etiquette={texte.etiquetteNonAdopte} titre={texte.nonAdopte(dateHeure(lues.horodatage))} />
{/if}

{#if etat.decisions_adoptees && bilan}
  {#if lues}
    <Message
      type="info"
      etiquette={texte.etiquetteUtilisees}
      titre={texte.lecture(dateHeure(lues.horodatage), nomFichier(lues.fichier), lues.classeur_du_traitement)}
    />
  {/if}

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

    {#if bilan.lignes_ignorees > 0}
      <Message type="alerte" etiquette={texte.etiquetteIgnores} titre={texte.ouiIgnores(bilan.lignes_ignorees)} />
    {/if}

    {#if bilan.contradictions?.length > 0}
      <Message type="erreur" etiquette={texte.etiquetteBloquant} titre={texte.contradictions} details={bilan.contradictions} />
    {:else if bilan.statut === "ControleBloquant"}
      <Message type="erreur" etiquette={texte.etiquetteBloquant} titre={texte.bloquant} details={bilan.divergences} />
    {:else if bilan.statut === "AucuneDecisionAdmissible"}
      <Message type="alerte" etiquette={texte.etiquetteAucunSgx} titre={texte.aucuneAdmissible} />
    {:else if generationPossible}
      <Message type="ok" etiquette={texte.etiquetteControle} titre={texte.controle} />
    {/if}
  </section>

  {#if generationPossible}
    <p class="max-w-[72ch] font-semibold">{texte.seraApplique(bilan.lignes_admises, bilan.occurrences_admises)}</p>
  {/if}
  <div class="flex flex-wrap items-center gap-2">
    <Bouton principal disabled={occupe || !generationPossible} onclick={generer}>{texte.generer}</Bouton>
    <Bouton disabled={occupe} onclick={() => executer(textes.actions.ouvertureRapport, () => moteur.ouvrirFichier(etat.fichier_controle))}>
      {texte.ouvrirRapport}
    </Bouton>
  </div>
{/if}
