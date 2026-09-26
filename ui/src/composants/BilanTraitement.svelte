<script>
  import { dateHeure, COULEURS_DES_STATUTS } from "../format.js";
  import { textes } from "../textes.js";
  import Pastille from "./Pastille.svelte";

  let { etat } = $props();
  const texte = textes.bilan;

  const bilan = $derived(etat.dernier_bilan);
  const derniere = $derived(etat.tentatives.at(-1));
  // Classeur modifié sans être relu : ces valeurs sont celles de la dernière lecture.
  const decisionsAActualiser = $derived(etat.decisions_a_relire || etat.correspondances_a_relire);
</script>

<aside
  class="grid min-w-0 content-start gap-3.5 overflow-y-auto border-l border-trait bg-surface px-4 py-4.5
    max-[980px]:border-t max-[980px]:border-l-0"
  aria-label={texte.region}
>
  <h2 class="text-xs font-semibold tracking-wider text-encre-2 uppercase">{texte.titre}</h2>
  <dl class="grid grid-cols-[auto_1fr] gap-x-2.5 gap-y-1.5 text-[13px] [&_dd]:text-right [&_dd]:tabular-nums [&_dt]:text-encre-2">
    <dt>{texte.modeles}</dt><dd>{etat.modeles_reconnus}</dd>
    <dt>{texte.occurrences}</dt><dd>{etat.occurrences_inventoriees}</dd>
    <dt>{texte.noms}</dt><dd>{etat.noms_distincts}</dd>
    {#if etat.correspondances_adoptees && bilan}
      <dt>{texte.demandesLues}</dt>
      <dd>{bilan.correspondances_retenues}{#if etat.correspondances_a_relire}<span class="block text-xs text-alerte">{texte.aActualiser}</span>{/if}</dd>
    {/if}
    {#if etat.analyse_preparee}
      <dt>{texte.propositions}</dt><dd>{etat.propositions.length}</dd>
    {/if}
    {#if etat.decisions_adoptees && bilan}
      <dt>{texte.admises}</dt>
      <dd data-bilan="admises">{bilan.lignes_admises}{#if decisionsAActualiser}<span class="block text-xs text-alerte">{texte.aActualiser}</span>{/if}</dd>
    {/if}
  </dl>
  <p class="text-xs text-encre-2">{texte.source(etat.source_nom)}</p>

  {#if derniere}
    <h2 class="text-xs font-semibold tracking-wider text-encre-2 uppercase">{texte.derniereTentative}</h2>
    <div class="grid gap-1 border-t border-trait pt-2.5 text-[13px]">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <strong>{texte.tentative(derniere.numero)}</strong>
        <Pastille couleur={COULEURS_DES_STATUTS[derniere.statut]}>{textes.statuts[derniere.statut] ?? derniere.statut}</Pastille>
      </div>
      <span class="text-xs text-encre-2">{dateHeure(derniere.horodatage)} · {derniere.courante ? texte.entreesActuelles : texte.entreesPrecedentes}</span>
    </div>
  {/if}
  {#if etat.tentatives_interrompues.length > 0}
    <Pastille couleur="alerte">{texte.interrompues(etat.tentatives_interrompues.length)}</Pastille>
  {/if}
</aside>
