<script>
  import { tick } from "svelte";
  import * as moteur from "./moteur.js";
  import { textes } from "./textes.js";
  import Accueil from "./composants/Accueil.svelte";
  import Bouton from "./composants/Bouton.svelte";
  import Etapes from "./composants/Etapes.svelte";
  import BilanTraitement from "./composants/BilanTraitement.svelte";
  import Message from "./composants/Message.svelte";
  import EtapeSource from "./composants/EtapeSource.svelte";
  import EtapeCorrespondances from "./composants/EtapeCorrespondances.svelte";
  import EtapeAnalyse from "./composants/EtapeAnalyse.svelte";
  import EtapeDecisions from "./composants/EtapeDecisions.svelte";
  import EtapeResultat from "./composants/EtapeResultat.svelte";

  // État du traitement tel que retourné par le moteur : seule source de vérité.
  let etat = $state(null);
  let ecran = $state("source");
  // Libellé de l'action en cours ; vide quand aucune action n'est en cours.
  let occupe = $state("");
  let erreur = $state(null);
  let theme = $state(document.documentElement.dataset.theme);
  let contenu;

  const ECRAN_DE_L_ETAPE = {
    InventairePret: "source",
    CorrespondancesPretes: "correspondances",
    AnalysePrete: "analyse",
    DecisionsControlees: "decisions",
    ResultatProduit: "resultat",
  };

  // Exécute une action du moteur, une seule à la fois. En cas de succès, retourne
  // sa réponse et reprend l'état qu'elle contient ; en cas d'échec, affiche
  // l'erreur et retourne undefined.
  async function executer(libelle, action) {
    if (occupe) return undefined;
    occupe = libelle;
    erreur = null;
    try {
      const reponse = await action();
      if (reponse?.etat) etat = reponse.etat;
      return reponse;
    } catch (cause) {
      erreur = typeof cause === "string" ? { code: "technique", message: cause, details: [] } : cause;
      return undefined;
    } finally {
      occupe = "";
    }
  }

  async function naviguer(nouvelEcran) {
    ecran = nouvelEcran;
    erreur = null;
    await tick();
    contenu?.querySelector("h1")?.focus();
  }

  function naviguerSelonEtape() {
    naviguer(ECRAN_DE_L_ETAPE[etat.etape]);
  }

  async function fermer() {
    if ((await executer(textes.actions.fermeture, moteur.fermerTraitement)) !== undefined) {
      etat = null;
      naviguer("source");
    }
  }

  function basculerTheme() {
    theme = theme === "dark" ? "light" : "dark";
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("theme", theme);
  }

  // Au retour dans l'application (après Excel), l'état est relu pour signaler
  // un classeur modifié. Une lecture impossible à ce moment est sans conséquence.
  async function actualiser() {
    if (!etat || occupe) return;
    try {
      etat = (await moteur.etat()) ?? etat;
    } catch {
      // Une opération vient de démarrer : son résultat mettra l'état à jour.
    }
  }
</script>

<svelte:window onfocus={actualiser} />

<div class="grid h-full grid-rows-[auto_1fr]">
  <header class="flex flex-wrap items-center gap-x-4 gap-y-2 border-b border-trait bg-surface px-4 py-2.5">
    <span class="font-semibold">{textes.application.nom}</span>
    {#if etat}
      <span class="text-encre-2 wrap-anywhere">· {etat.nom}</span>
    {/if}
    <span class="flex-1"></span>
    {#if etat}
      <Bouton disabled={!!occupe} onclick={() => executer(textes.actions.ouvertureDossier, () => moteur.afficherDansDossier(etat.dossier))}>
        {textes.application.afficherDossier}
      </Bouton>
      <Bouton disabled={!!occupe} onclick={fermer}>{textes.application.fermer}</Bouton>
    {/if}
    <Bouton aria-pressed={theme === "dark"} onclick={basculerTheme}>{textes.application.themeSombre}</Bouton>
  </header>

  <div
    class="grid min-h-0 max-[980px]:grid-cols-1 max-[980px]:overflow-y-auto
      {etat ? 'grid-cols-[212px_minmax(0,1fr)_268px]' : 'grid-cols-1'}"
  >
    {#if etat}
      <Etapes {etat} {ecran} {naviguer} />
    {/if}

    <main
      class="grid min-w-0 content-start gap-4.5 overflow-y-auto px-6.5 pt-5.5 pb-8
        max-[980px]:overflow-visible max-[980px]:px-4 max-[980px]:pt-4.5"
      bind:this={contenu}
    >
      {#if erreur}
        <Message type="erreur" etiquette={textes.application.etiquetteErreur} titre={erreur.message} details={erreur.details} />
      {/if}

      {#if !etat}
        <Accueil {executer} {naviguerSelonEtape} occupe={!!occupe} />
      {:else if ecran === "source"}
        <EtapeSource {etat} {naviguer} />
      {:else if ecran === "correspondances"}
        <EtapeCorrespondances {etat} {executer} {naviguer} occupe={!!occupe} />
      {:else if ecran === "analyse"}
        <EtapeAnalyse {etat} {naviguer} occupe={!!occupe} />
      {:else if ecran === "decisions"}
        <EtapeDecisions {etat} {executer} {naviguer} occupe={!!occupe} />
      {:else}
        <EtapeResultat {etat} {executer} {naviguer} occupe={!!occupe} />
      {/if}
    </main>

    {#if etat}
      <BilanTraitement {etat} />
    {/if}
  </div>
</div>

<div id="action-en-cours" role="status" aria-live="polite">
  {#if occupe}
    <span class="fixed right-4 bottom-4 rounded-md bg-encre px-3.5 py-2 font-semibold text-fond shadow-lg">{occupe}…</span>
  {/if}
</div>
