<script>
  import { dateHeure, COULEURS_DES_STATUTS } from "../format.js";
  import { textes } from "../textes.js";
  import Pastille from "./Pastille.svelte";

  // Barre d'état, toujours visible en bas de la fenêtre : une ligne, deux si la place manque.
  // La liste est en « contents » : ses paires s'enchaînent avec le titre et la source.
  let { etat } = $props();
  const texte = textes.bilan;

  const bilan = $derived(etat.dernier_bilan);
  const derniere = $derived(etat.tentatives.at(-1));
  // Classeur modifié sans être relu : ces valeurs sont celles de la dernière lecture.
  const decisionsAActualiser = $derived(etat.decisions_a_relire || etat.correspondances_a_relire);
</script>

<footer
  class="flex flex-wrap items-center gap-x-5.5 gap-y-1.5 border-t border-trait bg-surface px-4 py-2 text-[13px]"
  aria-label={texte.region}
>
  <h2 class="text-xs font-semibold tracking-wider text-encre-2 uppercase">{texte.titre}</h2>
  <dl class="contents [&_dd]:font-semibold [&_dd]:tabular-nums [&_dt]:text-encre-2 [&>div]:flex [&>div]:items-baseline [&>div]:gap-1.5">
    <div><dt>{texte.modeles}</dt><dd>{etat.modeles_reconnus}</dd></div>
    <div><dt>{texte.occurrences}</dt><dd>{etat.occurrences_inventoriees}</dd></div>
    <div><dt>{texte.noms}</dt><dd>{etat.noms_distincts}</dd></div>
    {#if etat.correspondances_adoptees && bilan}
      <div>
        <dt>{texte.demandesLues}</dt>
        <dd>{bilan.correspondances_retenues}{#if etat.correspondances_a_relire}<span class="ml-1 text-xs font-normal text-alerte">{texte.aActualiser}</span>{/if}</dd>
      </div>
    {/if}
    {#if etat.analyse_preparee}
      <div>
        <dt>{texte.propositions}</dt>
        <dd data-bilan="propositions">{etat.propositions.length}{#if etat.correspondances_a_relire}<span class="ml-1 text-xs font-normal text-alerte">{texte.aActualiser}</span>{/if}</dd>
      </div>
    {/if}
    {#if etat.decisions_adoptees && bilan}
      <div>
        <dt>{texte.admises}</dt>
        <dd data-bilan="admises">{bilan.lignes_admises}{#if decisionsAActualiser}<span class="ml-1 text-xs font-normal text-alerte">{texte.aActualiser}</span>{/if}</dd>
      </div>
    {/if}
    {#if derniere}
      <div>
        <dt>{texte.derniereTentative}</dt>
        <dd class="flex flex-wrap items-center gap-2">
          {texte.tentative(derniere.numero)}
          <Pastille couleur={COULEURS_DES_STATUTS[derniere.statut]}>{textes.statuts[derniere.statut] ?? derniere.statut}</Pastille>
          <span class="text-xs font-normal text-encre-2">{dateHeure(derniere.horodatage)} · {derniere.courante ? texte.entreesActuelles : texte.entreesPrecedentes}</span>
        </dd>
      </div>
    {/if}
  </dl>
  {#if etat.tentatives_interrompues.length > 0}
    <Pastille couleur="alerte">{texte.interrompues(etat.tentatives_interrompues.length)}</Pastille>
  {/if}
  <p class="ml-auto text-xs text-encre-2">{texte.source(etat.source_nom)}</p>
</footer>
