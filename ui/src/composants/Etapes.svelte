<script>
  import { textes } from "../textes.js";

  let { etat, ecran, naviguer } = $props();
  const texte = textes.etapes;

  // Une étape n'est accessible que si le moteur a produit ce qu'elle affiche.
  const etapes = $derived([
    { id: "source", libelle: texte.source, accessible: true, faite: true, aRelire: false },
    {
      id: "correspondances",
      libelle: texte.correspondances,
      accessible: true,
      faite: etat.correspondances_adoptees,
      aRelire: etat.correspondances_a_relire,
    },
    // Analyse non préparée : les propositions ne sont pas calculées, ce n'est pas une analyse vide.
    { id: "analyse", libelle: texte.analyse, accessible: etat.analyse_preparee, faite: etat.analyse_preparee, aRelire: false },
    {
      id: "decisions",
      libelle: texte.decisions,
      accessible: etat.analyse_preparee && etat.propositions.length > 0,
      faite: etat.decisions_adoptees,
      aRelire: etat.decisions_a_relire,
    },
    {
      id: "resultat",
      libelle: texte.resultat,
      // Une tentative interrompue seule doit rester consultable (son dossier y est indiqué).
      accessible: etat.tentatives.length > 0 || etat.tentatives_interrompues.length > 0,
      faite: etat.etape === "ResultatProduit",
      aRelire: false,
    },
  ]);

  function etatDeLEtape(etape) {
    if (etape.aRelire) return texte.aRelire;
    if (etape.faite) return texte.terminee;
    if (etape.id === ecran) return texte.enCours;
    return "";
  }

  function classesDuNumero(etape) {
    if (etape.faite && !etape.aRelire) return "border-ok bg-ok text-fond";
    if (etape.id === ecran) return "border-accent text-accent";
    return "border-trait text-encre-2";
  }
</script>

<nav
  class="grid content-start gap-0.5 overflow-y-auto border-r border-trait px-2.5 py-3.5
    max-[980px]:auto-cols-[minmax(96px,1fr)] max-[980px]:grid-flow-col max-[980px]:overflow-x-auto
    max-[980px]:border-r-0 max-[980px]:border-b"
  aria-label={texte.navigation}
>
  {#each etapes as etape, index}
    <button
      type="button"
      class="grid w-full cursor-pointer grid-cols-[24px_1fr] items-center gap-x-2 gap-y-0.5 rounded-md p-2 text-left
        enabled:hover:bg-surface disabled:cursor-default disabled:opacity-50 aria-[current=step]:bg-accent-doux"
      disabled={!etape.accessible}
      aria-current={etape.id === ecran ? "step" : undefined}
      data-etape={etape.id}
      onclick={() => naviguer(etape.id)}
    >
      <span class="grid size-6 place-items-center rounded-full border-[1.5px] text-xs font-semibold {classesDuNumero(etape)}" aria-hidden="true">
        {etape.faite && !etape.aRelire ? "✓" : index + 1}
      </span>
      <span class="font-semibold">{etape.libelle}</span>
      <span class="col-start-2 text-xs text-encre-2 max-[980px]:hidden">{etatDeLEtape(etape)}</span>
    </button>
  {/each}
</nav>
