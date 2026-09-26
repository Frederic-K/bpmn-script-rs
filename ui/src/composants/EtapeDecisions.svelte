<script>
  import * as moteur from "../moteur.js";
  import Bouton from "./Bouton.svelte";
  import Fichier from "./Fichier.svelte";
  import Message from "./Message.svelte";
  import Pastille from "./Pastille.svelte";
  import Tableau from "./Tableau.svelte";

  let { etat, executer, naviguer, occupe } = $props();

  const bilan = $derived(etat.dernier_bilan);
  const aLire = $derived(!etat.decisions_adoptees || etat.decisions_a_relire);
  // Le moteur refait tous ces contrôles à la génération ; le bouton ne fait que les refléter.
  const generationPossible = $derived(
    etat.decisions_adoptees &&
      !etat.decisions_a_relire &&
      !etat.correspondances_a_relire &&
      (bilan?.statut === "ProductionPossible" || bilan?.statut === "SgxProduit"),
  );

  function lire(fichier = null) {
    return executer("Lecture et contrôle des décisions", () => moteur.lireDecisions(etat.revision, fichier));
  }

  async function importer() {
    const fichier = await executer("Choix du classeur", moteur.choisirClasseur);
    if (fichier) await lire(fichier);
  }

  async function generer() {
    const reponse = await executer("Génération et vérification du SGX", () => moteur.produire(etat.revision));
    if (reponse) naviguer("resultat");
  }
</script>

<h1 class="text-xl leading-snug font-semibold text-balance" tabindex="-1">Décisions</h1>
<p class="max-w-[72ch] text-encre-2">
  Renseignez OUI ou NON dans la colonne Validation, puis lisez le classeur enregistré. Les lignes absentes ne seront
  pas appliquées.
</p>

{#if etat.edition_decisions}
  <Fichier type="XLSX" chemin={etat.edition_decisions}>
    <Bouton disabled={occupe} onclick={() => executer("Ouverture du classeur", () => moteur.ouvrirFichier(etat.edition_decisions))}>
      Ouvrir le classeur de décision
    </Bouton>
  </Fichier>
{/if}

{#if etat.decisions_adoptees && etat.decisions_a_relire}
  <Message type="alerte" etiquette="À relire" titre="Ce classeur a changé depuis sa dernière lecture. Relisez-le pour actualiser le traitement ; la génération est suspendue d'ici là." />
{:else if etat.correspondances_a_relire}
  <Message type="alerte" etiquette="À relire" titre="Le classeur de correspondance a changé depuis sa dernière lecture. Relisez-le avant de générer le SGX." />
{:else if aLire}
  <Message type="info" etiquette="Info" titre="Enregistrez puis fermez le classeur avant de le lire. Un retour d'arbitre s'importe tel quel : il est contrôlé ligne par ligne contre l'analyse." />
{/if}

<div class="flex flex-wrap items-center gap-2">
  <Bouton principal={aLire} disabled={occupe} onclick={() => lire()}>
    {etat.decisions_adoptees ? "Relire et contrôler les décisions" : "Lire et contrôler les décisions"}
  </Bouton>
  <Bouton disabled={occupe} onclick={importer}>Importer un retour d'arbitrage</Bouton>
</div>

{#if etat.decisions_adoptees && bilan}
  <div
    class="grid grid-cols-[repeat(auto-fit,minmax(112px,1fr))] gap-2.5
      [&>div]:grid [&>div]:content-start [&>div]:gap-1 [&>div]:rounded-md [&>div]:border [&>div]:border-trait [&>div]:px-3 [&>div]:py-2.5
      [&_b]:text-[22px] [&_b]:leading-tight [&_b]:tabular-nums [&_small]:text-xs [&_small]:text-encre-2"
  >
    <div><Pastille couleur="ok">Admises</Pastille><b>{bilan.lignes_admises}</b><small>{bilan.occurrences_admises} occurrences prévues</small></div>
    <div><Pastille>Refusées</Pastille><b>{bilan.lignes_refusees}</b><small>NON</small></div>
    <div><Pastille couleur="alerte">Ignorées</Pastille><b>{bilan.lignes_ignorees}</b><small>OUI non conforme</small></div>
    <div><Pastille>En attente</Pastille><b>{bilan.lignes_en_attente}</b><small>sans OUI ni NON</small></div>
    <div><Pastille>Sans décision</Pastille><b>{bilan.propositions_sans_decision}</b><small>propositions absentes du classeur</small></div>
  </div>

  {#if bilan.lignes_a_examiner.length > 0}
    <Tableau legende="Lignes non admises : elles ne seront pas appliquées.">
      <thead>
        <tr>
          <th scope="col" class="text-right">Ligne</th>
          <th scope="col">Nom actuel</th>
          <th scope="col">Nouveau nom</th>
          <th scope="col">Résultat</th>
          <th scope="col">Motif</th>
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
    <Message
      type="erreur"
      etiquette="Bloquant"
      titre="Les modifications ne correspondent pas aux occurrences attendues. Aucun nouveau SGX ne peut être créé : corrigez les décisions ou les correspondances."
      details={bilan.divergences}
    />
  {:else if bilan.statut === "AucuneDecisionAdmissible"}
    <Message type="alerte" etiquette="Aucun SGX" titre="Aucune décision admissible : aucun SGX ne peut être généré." />
  {:else if generationPossible}
    <Message type="ok" etiquette="Contrôlé" titre="Décisions contrôlées contre l'analyse ; le recomptage en mémoire est conforme." />
  {/if}

  <div class="flex flex-wrap items-center gap-2">
    <Bouton principal disabled={occupe || !generationPossible} onclick={generer}>Générer le SGX modifié</Bouton>
    <Bouton disabled={occupe} onclick={() => executer("Ouverture du rapport", () => moteur.ouvrirFichier(etat.fichier_controle))}>
      Ouvrir le rapport de contrôle
    </Bouton>
  </div>
{/if}
