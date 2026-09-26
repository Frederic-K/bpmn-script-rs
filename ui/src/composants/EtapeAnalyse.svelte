<script>
  import * as moteur from "../moteur.js";
  import Message from "./Message.svelte";

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

<h1 tabindex="-1">Analyse</h1>

{#if propositions.length === 0}
  <Message type="info" etiquette="Résultat" titre="Aucun changement proposé. Vérifiez les nouveaux noms renseignés : aucun nom actuel ne correspond à l'inventaire.">
    <button class="bouton" type="button" onclick={() => naviguer("correspondances")}>Revenir aux correspondances</button>
  </Message>
{:else}
  <p class="intro">Aperçu en lecture seule des changements proposés. Rien n'est encore modifié.</p>

  <div class="chiffres">
    <div class="chiffre"><b>{propositions.length}</b><span>propositions</span></div>
    <div class="chiffre"><b>{occurrencesVisees}</b><span>occurrences visées</span></div>
    <div class="chiffre"><b>{modelesConcernes}</b><span>modèles concernés</span></div>
  </div>

  <div class="tableau">
    <table>
      <caption class="petit">Une ligne par modèle et par nom ; des flux homonymes restent distincts par leur fichier modèle.</caption>
      <thead>
        <tr><th scope="col">Flux</th><th scope="col">Nom actuel</th><th scope="col">Nouveau nom</th><th scope="col" class="nombre">Occ.</th><th scope="col">Fichier modèle</th></tr>
      </thead>
      <tbody>
        {#each propositions as proposition}
          <tr>
            <td>{proposition.flux}</td>
            <td class="nom-lane">{proposition.nom_actuel}</td>
            <td class="nom-lane nouveau">{proposition.nouveau_nom}</td>
            <td class="nombre">{proposition.occurrences}</td>
            <td class="mono">{proposition.fichier_modele}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#if copie}
    <Message type="ok" etiquette="Copie" titre="Copie enregistrée pour arbitrage : {copie}" />
  {/if}

  <div class="rangee">
    <button class="bouton principal" type="button" disabled={occupe} onclick={() => naviguer("decisions")}>Passer aux décisions</button>
    <button class="bouton" type="button" disabled={occupe} onclick={() => executer("Ouverture de l'analyse", () => moteur.ouvrirFichier(etat.fichier_analyse))}>
      Ouvrir l'analyse
    </button>
    <button class="bouton" type="button" disabled={occupe} onclick={enregistrerCopie}>Enregistrer une copie pour arbitrage</button>
  </div>
{/if}
