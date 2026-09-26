<script>
  import { dateHeure, STATUTS } from "../format.js";

  let { etat } = $props();

  const bilan = $derived(etat.dernier_bilan);
  const derniere = $derived(etat.tentatives.at(-1));
</script>

<aside class="bilan" aria-label="Bilan du traitement">
  <h2>Traitement</h2>
  <dl>
    <dt>Modèles</dt><dd>{etat.modeles_reconnus}</dd>
    <dt>Occurrences</dt><dd>{etat.occurrences_inventoriees}</dd>
    <dt>Noms distincts</dt><dd>{etat.noms_distincts}</dd>
    {#if etat.correspondances_adoptees && bilan}
      <dt>Demandes lues</dt><dd>{bilan.correspondances_retenues}</dd>
    {/if}
    {#if etat.analyse_preparee}
      <dt>Propositions</dt><dd>{etat.propositions.length}</dd>
    {/if}
    {#if etat.decisions_adoptees && bilan}
      <dt>Décisions admises</dt><dd>{bilan.lignes_admises}</dd>
    {/if}
  </dl>
  <p class="petit">Source : {etat.source_nom}, copie contrôlée par empreinte.</p>

  {#if derniere}
    <h2>Dernière tentative</h2>
    <div class="tentative">
      <div class="ligne">
        <strong>Tentative {derniere.numero}</strong>
        <span class="pastille {STATUTS[derniere.statut]?.couleur ?? 'neutre'}">{STATUTS[derniere.statut]?.libelle ?? derniere.statut}</span>
      </div>
      <span class="petit">{dateHeure(derniere.horodatage)} · {derniere.courante ? "entrées actuelles" : "entrées précédentes"}</span>
    </div>
  {/if}
  {#if etat.tentatives_interrompues.length > 0}
    <span class="pastille alerte">{etat.tentatives_interrompues.length} tentative(s) interrompue(s)</span>
  {/if}
</aside>
