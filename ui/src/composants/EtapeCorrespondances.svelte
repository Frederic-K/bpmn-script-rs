<script>
  import * as moteur from "../moteur.js";
  import { dateHeure, nomFichier } from "../format.js";
  import { textes } from "../textes.js";
  import Bouton from "./Bouton.svelte";
  import Fichier from "./Fichier.svelte";
  import Message from "./Message.svelte";

  let { etat, executer, naviguer, occupe } = $props();
  const texte = textes.correspondances;

  const bilan = $derived(etat.dernier_bilan);
  const lues = $derived(etat.correspondances_lues);
  const aLire = $derived(!etat.correspondances_adoptees || etat.correspondances_a_relire);
  // Action principale : ouvrir le classeur tant qu'il n'a été ni modifié ni lu, puis le lire une fois modifié.
  const aOuvrir = $derived(!etat.correspondances_adoptees && !etat.correspondances_a_relire);

  let lectureRefusee = $state(false);
  let copie = $state(null);

  async function enregistrerCopie() {
    const destination = await executer(textes.actions.copie, () =>
      moteur.enregistrerCopie(etat.edition_correspondances, texte.nomCopie(etat.nom)),
    );
    if (destination) copie = destination;
  }

  async function lire(fichier = null) {
    const reponse = await executer(textes.actions.lectureCorrespondances, () => moteur.lireCorrespondances(etat.revision, fichier));
    lectureRefusee = !reponse;
  }

  async function importer() {
    const fichier = await executer(textes.actions.choixClasseur, moteur.choisirClasseur);
    if (fichier) await lire(fichier);
  }

  async function preparerAnalyse() {
    const reponse = await executer(textes.actions.preparationAnalyse, () => moteur.preparerAnalyse(etat.revision));
    if (reponse) naviguer("analyse");
  }
</script>

<p class="max-w-[72ch] text-encre-2">{texte.consigne}</p>

<Fichier type="XLSX" chemin={etat.edition_correspondances}>
  <Bouton principal={aOuvrir} disabled={occupe} onclick={() => executer(textes.actions.ouvertureClasseur, () => moteur.ouvrirFichier(etat.edition_correspondances))}>
    {texte.ouvrir}
  </Bouton>
  <Bouton disabled={occupe} onclick={enregistrerCopie}>{texte.copier}</Bouton>
</Fichier>
<p class="max-w-[72ch] text-xs text-encre-2">{texte.copieSansEcrasement}</p>
{#if copie}
  <Message type="ok" etiquette={texte.etiquetteCopie} titre={texte.copieEnregistree(copie)} />
{/if}

{#if etat.correspondances_adoptees && etat.correspondances_a_relire}
  <Message type="alerte" etiquette={texte.etiquetteARelire} titre={texte.aRelire} />
{:else if aLire}
  <Message type="info" etiquette={texte.etiquetteInfo} titre={texte.fermerAvantLecture} />
{/if}

{#if lectureRefusee && lues}
  <Message type="alerte" etiquette={texte.etiquetteNonAdopte} titre={texte.nonAdopte(dateHeure(lues.horodatage))} />
{/if}

{#if etat.correspondances_adoptees && bilan}
  <Message type="ok" etiquette={texte.etiquetteLu} titre={texte.lus(bilan.correspondances_retenues)} />
  {#if lues}
    <Message
      type="info"
      etiquette={texte.etiquetteUtilise}
      titre={texte.lecture(dateHeure(lues.horodatage), nomFichier(lues.fichier), lues.classeur_du_traitement)}
    />
  {/if}
  {#if bilan.noms_inconnus.length > 0}
    <Message
      type="alerte"
      etiquette={texte.etiquetteAVerifier}
      titre={texte.nomsInconnus}
      details={bilan.noms_inconnus.map((nom) => `« ${nom} »`)}
    />
  {/if}
  {#if bilan.noms_en_doublon.length > 0}
    <Message
      type="alerte"
      etiquette={texte.etiquetteAVerifier}
      titre={texte.nomsEnDoublon}
      details={bilan.noms_en_doublon.map((nom) => `« ${nom} »`)}
    />
  {/if}
{/if}

<div class="flex flex-wrap items-center gap-2">
  <Bouton principal={etat.correspondances_a_relire} disabled={occupe} onclick={() => lire()}>
    {etat.correspondances_adoptees ? texte.relire : texte.lire}
  </Bouton>
  <Bouton disabled={occupe} onclick={importer}>{texte.importer}</Bouton>
  {#if etat.analyse_preparee}
    <Bouton principal disabled={occupe || aLire} onclick={() => naviguer("analyse")}>{texte.voirAnalyse}</Bouton>
  {:else if etat.correspondances_adoptees}
    <Bouton principal disabled={occupe || aLire} onclick={preparerAnalyse}>{texte.preparerAnalyse}</Bouton>
  {/if}
</div>
<p class="max-w-[72ch] text-xs text-encre-2">{texte.importerExplication}</p>
