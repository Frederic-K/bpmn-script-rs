<script>
  import { tick } from "svelte";
  import * as moteur from "./moteur.js";
  import Accueil from "./composants/Accueil.svelte";
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
    if ((await executer("Fermeture", moteur.fermerTraitement)) !== undefined) {
      etat = null;
      naviguer("source");
    }
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

<div class="application">
  <header class="barre">
    <span class="appli">BPMN-Script</span>
    {#if etat}
      <span class="traitement">· {etat.nom}</span>
      <span class="espace"></span>
      <button class="bouton" disabled={!!occupe} onclick={() => executer("Ouverture du dossier", () => moteur.afficherDansDossier(etat.dossier))}>
        Afficher le dossier du traitement
      </button>
      <button class="bouton" disabled={!!occupe} onclick={fermer}>Fermer le traitement</button>
    {/if}
  </header>

  <div class="corps" class:accueil={!etat}>
    {#if etat}
      <Etapes {etat} {ecran} {naviguer} />
    {/if}

    <main class="contenu" bind:this={contenu}>
      {#if erreur}
        <Message type="erreur" etiquette="Erreur" titre={erreur.message} details={erreur.details} />
      {/if}

      {#if !etat}
        <Accueil {executer} {naviguerSelonEtape} occupe={!!occupe} />
      {:else if ecran === "source"}
        <EtapeSource {etat} {naviguer} />
      {:else if ecran === "correspondances"}
        <EtapeCorrespondances {etat} {executer} {naviguer} occupe={!!occupe} />
      {:else if ecran === "analyse"}
        <EtapeAnalyse {etat} {executer} {naviguer} occupe={!!occupe} />
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

<div role="status" aria-live="polite">
  {#if occupe}<span class="occupe">{occupe}…</span>{/if}
</div>
