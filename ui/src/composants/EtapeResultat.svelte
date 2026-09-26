<script>
  import * as moteur from "../moteur.js";
  import { dateHeure, COULEURS_DES_STATUTS } from "../format.js";
  import { textes } from "../textes.js";
  import Bouton from "./Bouton.svelte";
  import Chiffre from "./Chiffre.svelte";
  import Fichier from "./Fichier.svelte";
  import Message from "./Message.svelte";
  import Pastille from "./Pastille.svelte";

  let { etat, executer, naviguer, occupe } = $props();
  const texte = textes.resultat;

  // La tentative courante porte sur les entrées actuellement lues ; les autres sont des résultats précédents.
  const courante = $derived(etat.tentatives.findLast((tentative) => tentative.courante));
  const precedentes = $derived(etat.tentatives.filter((tentative) => tentative !== courante).reverse());
  const entreesModifiees = $derived(etat.correspondances_a_relire || etat.decisions_a_relire);

  const ETATS_FICHIER = {
    Absent: texte.fichierAbsent,
    Modifie: texte.fichierModifie,
  };

  const CAUSES = {
    AucuneDecisionAdmissible: texte.causeAucuneAdmissible,
    ControleBloquant: texte.causeBloquant,
  };

  let copie = $state(null);

  async function enregistrerCopie() {
    const destination = await executer(textes.actions.copie, () => moteur.enregistrerCopie(courante.sgx));
    if (destination) copie = destination;
  }

  function rapport(tentative) {
    return `${tentative.dossier}/controle_validation.xlsx`;
  }
</script>

<h1 class="text-xl leading-snug font-semibold text-balance" tabindex="-1">{texte.titre}</h1>

{#each etat.tentatives_interrompues as dossier}
  <Message type="alerte" etiquette={texte.etiquetteInterrompue} titre={texte.interrompue} details={[dossier]} />
{/each}

{#if courante && entreesModifiees}
  <Message type="alerte" etiquette={texte.etiquetteARelire} titre={texte.entreesModifiees} />
{/if}

{#if courante?.sgx}
  {#if courante.etat_sgx === "Disponible"}
    <Message type="ok" etiquette={texte.etiquetteProduit} titre={texte.produit} />
  {:else}
    <Message type="erreur" etiquette={texte.etiquetteAttention} titre={ETATS_FICHIER[courante.etat_sgx]} />
  {/if}
  <Fichier type="SGX" chemin={courante.sgx}>
    {#if courante.etat_sgx === "Disponible"}
      <Bouton disabled={occupe} onclick={enregistrerCopie}>{texte.enregistrerCopie}</Bouton>
    {/if}
  </Fichier>
  {#if copie}
    <Message type="ok" etiquette={texte.etiquetteCopie} titre={texte.copieEnregistree(copie)} />
  {/if}
  <div class="grid grid-cols-[repeat(auto-fit,minmax(130px,1fr))] gap-2.5">
    <Chiffre valeur={courante.modeles_modifies} libelle={texte.modeles} />
    <Chiffre valeur={courante.occurrences_modifiees} libelle={texte.occurrences} />
    <Chiffre valeur={dateHeure(courante.horodatage)} libelle={texte.tentative(courante.numero)} />
  </div>
  <div class="flex flex-wrap items-center gap-2">
    <Bouton principal disabled={occupe || courante.etat_sgx === "Absent"} onclick={() => executer(textes.actions.ouvertureDossier, () => moteur.afficherDansDossier(courante.sgx))}>
      {texte.afficher}
    </Bouton>
    <Bouton disabled={occupe} onclick={() => executer(textes.actions.ouvertureRapport, () => moteur.ouvrirFichier(rapport(courante)))}>
      {texte.ouvrirRapport}
    </Bouton>
  </div>
  <Message type="info" etiquette={texte.etiquetteSuivante} titre={texte.suivante} />
{:else if courante}
  <Message type="alerte" etiquette={texte.etiquetteAucunSgx} titre={texte.aucunSgx(CAUSES[courante.statut] ?? textes.statuts[courante.statut])}>
    <Bouton onclick={() => naviguer("decisions")}>{texte.retourDecisions}</Bouton>
    <Bouton disabled={occupe} onclick={() => executer(textes.actions.ouvertureRapport, () => moteur.ouvrirFichier(rapport(courante)))}>
      {texte.ouvrirRapport}
    </Bouton>
  </Message>
{:else}
  <p>{texte.aucunResultat}</p>
{/if}

{#if precedentes.length > 0}
  <section class="grid content-start gap-3 rounded-lg border border-trait p-4.5" aria-labelledby="titre-precedents">
    <h2 id="titre-precedents" class="text-base font-semibold">{texte.precedents}</h2>
    <p class="text-xs text-encre-2">{texte.precedentsExplication}</p>
    {#each precedentes as tentative}
      <div class="grid gap-1 border-t border-trait pt-2.5 text-[13px]">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <strong>{texte.tentativeDatee(tentative.numero, dateHeure(tentative.horodatage))}</strong>
          <Pastille couleur={COULEURS_DES_STATUTS[tentative.statut]}>{textes.statuts[tentative.statut] ?? tentative.statut}</Pastille>
        </div>
        {#if tentative.sgx}
          <span class="text-xs text-encre-2">{tentative.sgx}{tentative.etat_sgx !== "Disponible" ? ` — ${ETATS_FICHIER[tentative.etat_sgx]}` : ""}</span>
        {/if}
      </div>
    {/each}
  </section>
{/if}
