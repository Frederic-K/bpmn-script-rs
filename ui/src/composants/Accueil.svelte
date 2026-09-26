<script>
  import * as moteur from "../moteur.js";
  import Fichier from "./Fichier.svelte";

  let { executer, naviguerSelonEtape, occupe } = $props();

  let source = $state(null);
  let dossierParent = $state(null);

  async function choisirSource() {
    const chemin = await executer("Choix du fichier", moteur.choisirSgx);
    if (chemin) source = chemin;
  }

  async function choisirDossierParent() {
    const chemin = await executer("Choix du dossier", () =>
      moteur.choisirDossier("Dossier qui accueillera le traitement"),
    );
    if (chemin) dossierParent = chemin;
  }

  async function creer() {
    const reponse = await executer("Copie et inventaire du SGX", () => moteur.creerTraitement(source, dossierParent));
    if (reponse) naviguerSelonEtape();
  }

  async function reprendre() {
    const dossier = await executer("Choix du dossier", () => moteur.choisirDossier("Ouvrir un dossier de traitement"));
    if (!dossier) return;
    const reponse = await executer("Ouverture et vérification du traitement", () => moteur.ouvrirTraitement(dossier));
    if (reponse) naviguerSelonEtape();
  }
</script>

<h1 tabindex="-1">Harmoniser les noms de swimlanes d'un export Signavio</h1>
<p class="intro">
  Un traitement réunit un SGX, les classeurs Excel de correspondance et de décision, et les résultats. Il est
  enregistré dans un dossier que vous choisissez et peut être repris plus tard.
</p>

<div class="choix-grille">
  <section class="choix" aria-labelledby="titre-nouveau">
    <h2 id="titre-nouveau">Nouveau traitement</h2>
    <p class="petit">
      Choisissez un export .sgx et le dossier qui accueillera le traitement. Le fichier choisi n'est jamais modifié :
      l'application travaille sur une copie.
    </p>
    {#if source}
      <Fichier type="SGX" chemin={source}>
        <button class="bouton" type="button" disabled={occupe} onclick={choisirSource}>Changer</button>
      </Fichier>
    {:else}
      <div class="rangee">
        <button class="bouton" type="button" disabled={occupe} onclick={choisirSource}>Choisir le fichier SGX</button>
      </div>
    {/if}
    {#if dossierParent}
      <Fichier type="DOSSIER" chemin={dossierParent}>
        <button class="bouton" type="button" disabled={occupe} onclick={choisirDossierParent}>Changer</button>
      </Fichier>
    {:else}
      <div class="rangee">
        <button class="bouton" type="button" disabled={occupe} onclick={choisirDossierParent}>
          Choisir le dossier des traitements
        </button>
      </div>
    {/if}
    <div class="rangee">
      <button class="bouton principal" type="button" disabled={occupe || !source || !dossierParent} onclick={creer}>
        Inventorier les lanes
      </button>
    </div>
  </section>

  <section class="choix" aria-labelledby="titre-reprendre">
    <h2 id="titre-reprendre">Reprendre un traitement</h2>
    <p class="petit">
      Ouvrez le dossier d'un traitement existant. Son état est vérifié : copie de la source, classeurs lus et
      résultats.
    </p>
    <div class="rangee">
      <button class="bouton" type="button" disabled={occupe} onclick={reprendre}>Ouvrir un dossier de traitement</button>
    </div>
  </section>
</div>
