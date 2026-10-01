<script>
  import { textes } from "../textes.js";

  let { etat, ecran, naviguer } = $props();
  const texte = textes.etapes;

  // Un classeur modifié sans être relu : il est « à relire », et les étapes qui
  // en dépendent sont « à actualiser ».
  const correspondancesModifiees = $derived(etat.correspondances_a_relire);
  const decisionsModifiees = $derived(etat.decisions_a_relire);

  // Une étape n'est accessible que si le moteur a produit ce qu'elle affiche.
  const etapes = $derived([
    { id: "source", libelle: texte.source, accessible: true, faite: true, alerte: "" },
    {
      id: "correspondances",
      libelle: texte.correspondances,
      accessible: true,
      faite: etat.correspondances_adoptees,
      alerte: correspondancesModifiees ? texte.aRelire : "",
    },
    {
      id: "analyse",
      libelle: texte.analyse,
      // Analyse non préparée : les propositions ne sont pas calculées, ce n'est pas une analyse vide.
      accessible: etat.analyse_preparee,
      faite: etat.analyse_preparee,
      alerte: etat.analyse_preparee && correspondancesModifiees ? texte.aActualiser : "",
    },
    {
      id: "decisions",
      libelle: texte.decisions,
      accessible: etat.analyse_preparee && etat.propositions.length > 0,
      // Terminée seulement si le contrôle autorise la génération (pas sur un contrôle bloquant).
      faite:
        etat.decisions_adoptees &&
        (etat.dernier_bilan?.statut === "ProductionPossible" || etat.dernier_bilan?.statut === "SgxProduit"),
      alerte: decisionsModifiees ? texte.aRelire : etat.decisions_adoptees && correspondancesModifiees ? texte.aActualiser : "",
    },
    {
      id: "resultat",
      libelle: texte.resultat,
      // Une tentative interrompue seule doit rester consultable (son dossier y est indiqué).
      accessible: etat.tentatives.length > 0 || etat.tentatives_interrompues.length > 0,
      faite: etat.etape === "ResultatProduit",
      alerte: etat.etape === "ResultatProduit" && (correspondancesModifiees || decisionsModifiees) ? texte.aActualiser : "",
    },
  ]);

  function etatDeLEtape(etape) {
    if (etape.alerte) return etape.alerte;
    if (etape.faite) return texte.terminee;
    if (etape.id === ecran) return texte.enCours;
    return "";
  }

  function classesDuNumero(etape) {
    if (etape.faite && !etape.alerte) return "border-ok bg-ok text-fond";
    if (etape.id === ecran) return "border-accent text-accent";
    return "border-trait text-encre-2";
  }
</script>

<nav class="flex gap-0.5 overflow-x-auto border-b border-trait px-2.5 py-1.5" aria-label={texte.navigation}>
  {#each etapes as etape, index}
    <button
      type="button"
      class="grid flex-1 cursor-pointer grid-cols-[24px_1fr] items-center gap-x-2 gap-y-0.5 rounded-md p-2 text-left whitespace-nowrap
        enabled:hover:bg-surface disabled:cursor-default disabled:opacity-50 aria-[current=step]:bg-accent-doux"
      disabled={!etape.accessible}
      aria-current={etape.id === ecran ? "step" : undefined}
      data-etape={etape.id}
      onclick={() => naviguer(etape.id)}
    >
      <span class="grid size-6 place-items-center rounded-full border-[1.5px] text-xs font-semibold {classesDuNumero(etape)}" aria-hidden="true">
        {etape.faite && !etape.alerte ? "✓" : index + 1}
      </span>
      <span class="font-semibold">{etape.libelle}</span>
      <span class="col-start-2 text-xs text-encre-2">{etatDeLEtape(etape)}</span>
    </button>
  {/each}
</nav>
