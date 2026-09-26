<script>
  import * as moteur from "../moteur.js";
  import { textes } from "../textes.js";
  import Bouton from "./Bouton.svelte";
  import Fichier from "./Fichier.svelte";
  import Message from "./Message.svelte";

  let { etat, executer, naviguer, occupe } = $props();
  const texte = textes.correspondances;

  const bilan = $derived(etat.dernier_bilan);
  const aLire = $derived(!etat.correspondances_adoptees || etat.correspondances_a_relire);

  function lire(fichier = null) {
    return executer(textes.actions.lectureCorrespondances, () => moteur.lireCorrespondances(etat.revision, fichier));
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

<h1 class="text-xl leading-snug font-semibold text-balance" tabindex="-1">{texte.titre}</h1>
<p class="max-w-[72ch] text-encre-2">{texte.consigne}</p>

<Fichier type="XLSX" chemin={etat.edition_correspondances}>
  <Bouton disabled={occupe} onclick={() => executer(textes.actions.ouvertureClasseur, () => moteur.ouvrirFichier(etat.edition_correspondances))}>
    {texte.ouvrir}
  </Bouton>
</Fichier>

{#if etat.correspondances_adoptees && etat.correspondances_a_relire}
  <Message type="alerte" etiquette={texte.etiquetteARelire} titre={texte.aRelire} />
{:else if aLire}
  <Message type="info" etiquette={texte.etiquetteInfo} titre={texte.fermerAvantLecture} />
{/if}

{#if etat.correspondances_adoptees && bilan}
  <Message type="ok" etiquette={texte.etiquetteLu} titre={texte.lus(bilan.correspondances_retenues)} />
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
      type="info"
      etiquette={texte.etiquetteInformation}
      titre={texte.nomsEnDoublon}
      details={bilan.noms_en_doublon.map((nom) => `« ${nom} »`)}
    />
  {/if}
{/if}

<div class="flex flex-wrap items-center gap-2">
  <Bouton principal={aLire} disabled={occupe} onclick={() => lire()}>
    {etat.correspondances_adoptees ? texte.relire : texte.lire}
  </Bouton>
  <Bouton disabled={occupe} onclick={importer}>{texte.importer}</Bouton>
  {#if etat.analyse_preparee}
    <Bouton principal disabled={occupe || aLire} onclick={() => naviguer("analyse")}>{texte.voirAnalyse}</Bouton>
  {:else if etat.correspondances_adoptees}
    <Bouton principal disabled={occupe || aLire} onclick={preparerAnalyse}>{texte.preparerAnalyse}</Bouton>
  {/if}
</div>
