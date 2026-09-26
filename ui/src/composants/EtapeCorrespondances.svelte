<script>
  import * as moteur from "../moteur.js";
  import Bouton from "./Bouton.svelte";
  import Fichier from "./Fichier.svelte";
  import Message from "./Message.svelte";

  let { etat, executer, naviguer, occupe } = $props();

  const bilan = $derived(etat.dernier_bilan);
  const aLire = $derived(!etat.correspondances_adoptees || etat.correspondances_a_relire);

  function lire(fichier = null) {
    return executer("Lecture des correspondances", () => moteur.lireCorrespondances(etat.revision, fichier));
  }

  async function importer() {
    const fichier = await executer("Choix du classeur", moteur.choisirClasseur);
    if (fichier) await lire(fichier);
  }

  async function preparerAnalyse() {
    const reponse = await executer("Préparation de l'analyse", () => moteur.preparerAnalyse(etat.revision));
    if (reponse) naviguer("analyse");
  }
</script>

<h1 class="text-xl leading-snug font-semibold text-balance" tabindex="-1">Correspondances</h1>
<p class="max-w-[72ch] text-encre-2">Renseignez les nouveaux noms dans Excel. Une cellule Nouveau nom vide laisse le nom inchangé.</p>

<Fichier type="XLSX" chemin={etat.edition_correspondances}>
  <Bouton disabled={occupe} onclick={() => executer("Ouverture du classeur", () => moteur.ouvrirFichier(etat.edition_correspondances))}>
    Ouvrir le classeur
  </Bouton>
</Fichier>

{#if etat.correspondances_adoptees && etat.correspondances_a_relire}
  <Message type="alerte" etiquette="À relire" titre="Ce classeur a changé depuis sa dernière lecture. Relisez-le pour actualiser le traitement." />
{:else if aLire}
  <Message type="info" etiquette="Info" titre="Enregistrez puis fermez le classeur avant de le lire." />
{/if}

{#if etat.correspondances_adoptees && bilan}
  <Message type="ok" etiquette="Lu" titre="{bilan.correspondances_retenues} nom(s) avec une demande de renommage ont été lus." />
  {#if bilan.noms_inconnus.length > 0}
    <Message
      type="alerte"
      etiquette="À vérifier"
      titre="Ces noms n'existent pas dans l'inventaire et n'auront aucun effet (faute de frappe ?) :"
      details={bilan.noms_inconnus.map((nom) => `« ${nom} »`)}
    />
  {/if}
  {#if bilan.noms_en_doublon.length > 0}
    <Message
      type="info"
      etiquette="Information"
      titre="Ces noms sont renseignés plusieurs fois ; la dernière ligne est retenue :"
      details={bilan.noms_en_doublon.map((nom) => `« ${nom} »`)}
    />
  {/if}
{/if}

<div class="flex flex-wrap items-center gap-2">
  <Bouton principal={aLire} disabled={occupe} onclick={() => lire()}>
    {etat.correspondances_adoptees ? "Relire les correspondances" : "Lire les correspondances"}
  </Bouton>
  <Bouton disabled={occupe} onclick={importer}>Importer un classeur de correspondances</Bouton>
  {#if etat.analyse_preparee}
    <Bouton principal disabled={occupe || aLire} onclick={() => naviguer("analyse")}>Voir l'analyse</Bouton>
  {:else if etat.correspondances_adoptees}
    <Bouton principal disabled={occupe || aLire} onclick={preparerAnalyse}>Préparer l'analyse</Bouton>
  {/if}
</div>
