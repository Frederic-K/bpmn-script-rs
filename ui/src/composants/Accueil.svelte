<script>
  import * as moteur from "../moteur.js";
  import { textes } from "../textes.js";
  import Bouton from "./Bouton.svelte";
  import Fichier from "./Fichier.svelte";

  let { executer, naviguerSelonEtape, occupe } = $props();
  const texte = textes.accueil;

  let source = $state(null);
  let dossierParent = $state(null);

  async function choisirSource() {
    const chemin = await executer(textes.actions.choixFichier, moteur.choisirSgx);
    if (chemin) source = chemin;
  }

  async function choisirDossierParent() {
    const chemin = await executer(textes.actions.choixDossier, () => moteur.choisirDossier(texte.titreDialogueDossier));
    if (chemin) dossierParent = chemin;
  }

  async function creer() {
    const reponse = await executer(textes.actions.creation, () => moteur.creerTraitement(source, dossierParent));
    if (reponse) naviguerSelonEtape();
  }

  async function reprendre() {
    const dossier = await executer(textes.actions.choixDossier, () => moteur.choisirDossier(texte.titreDialogueTraitement));
    if (!dossier) return;
    const reponse = await executer(textes.actions.ouverture, () => moteur.ouvrirTraitement(dossier));
    if (reponse) naviguerSelonEtape();
  }
</script>

<h1 class="text-xl leading-snug font-semibold text-balance" tabindex="-1">{texte.titre}</h1>
<p class="max-w-[72ch] text-encre-2">{texte.introduction}</p>

<div class="grid max-w-215 grid-cols-[repeat(auto-fit,minmax(280px,1fr))] gap-4">
  <section class="grid content-start gap-3 rounded-lg border border-trait p-4.5" aria-labelledby="titre-nouveau">
    <h2 id="titre-nouveau" class="text-base font-semibold">{texte.nouveauTitre}</h2>
    <p class="text-xs text-encre-2">{texte.nouveauExplication}</p>
    {#if source}
      <Fichier type="SGX" chemin={source}>
        <Bouton disabled={occupe} onclick={choisirSource}>{texte.changer}</Bouton>
      </Fichier>
    {:else}
      <div><Bouton disabled={occupe} onclick={choisirSource}>{texte.choisirSgx}</Bouton></div>
    {/if}
    {#if dossierParent}
      <Fichier type="DOSSIER" chemin={dossierParent}>
        <Bouton disabled={occupe} onclick={choisirDossierParent}>{texte.changer}</Bouton>
      </Fichier>
    {:else}
      <div><Bouton disabled={occupe} onclick={choisirDossierParent}>{texte.choisirDossier}</Bouton></div>
    {/if}
    <div>
      <Bouton principal disabled={occupe || !source || !dossierParent} onclick={creer}>{texte.inventorier}</Bouton>
    </div>
  </section>

  <section class="grid content-start gap-3 rounded-lg border border-trait p-4.5" aria-labelledby="titre-reprendre">
    <h2 id="titre-reprendre" class="text-base font-semibold">{texte.reprendreTitre}</h2>
    <p class="text-xs text-encre-2">{texte.reprendreExplication}</p>
    <div><Bouton disabled={occupe} onclick={reprendre}>{texte.ouvrirTraitement}</Bouton></div>
  </section>
</div>
