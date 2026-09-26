<script>
  import { dateHeure, STATUTS } from "../format.js";
  import Pastille from "./Pastille.svelte";

  let { etat } = $props();

  const bilan = $derived(etat.dernier_bilan);
  const derniere = $derived(etat.tentatives.at(-1));
</script>

<aside
  class="grid min-w-0 content-start gap-3.5 overflow-y-auto border-l border-trait bg-surface px-4 py-4.5
    max-[980px]:border-t max-[980px]:border-l-0"
  aria-label="Bilan du traitement"
>
  <h2 class="text-xs font-semibold tracking-wider text-encre-2 uppercase">Traitement</h2>
  <dl class="grid grid-cols-[auto_1fr] gap-x-2.5 gap-y-1.5 text-[13px] [&_dd]:text-right [&_dd]:tabular-nums [&_dt]:text-encre-2">
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
  <p class="text-xs text-encre-2">Source : {etat.source_nom}, copie contrôlée par empreinte.</p>

  {#if derniere}
    <h2 class="text-xs font-semibold tracking-wider text-encre-2 uppercase">Dernière tentative</h2>
    <div class="grid gap-1 border-t border-trait pt-2.5 text-[13px]">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <strong>Tentative {derniere.numero}</strong>
        <Pastille couleur={STATUTS[derniere.statut]?.couleur}>{STATUTS[derniere.statut]?.libelle ?? derniere.statut}</Pastille>
      </div>
      <span class="text-xs text-encre-2">{dateHeure(derniere.horodatage)} · {derniere.courante ? "entrées actuelles" : "entrées précédentes"}</span>
    </div>
  {/if}
  {#if etat.tentatives_interrompues.length > 0}
    <Pastille couleur="alerte">{etat.tentatives_interrompues.length} tentative(s) interrompue(s)</Pastille>
  {/if}
</aside>
