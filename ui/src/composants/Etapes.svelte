<script>
  let { etat, ecran, naviguer } = $props();

  // Une étape n'est accessible que si le moteur a produit ce qu'elle affiche.
  const etapes = $derived([
    { id: "source", libelle: "Source", accessible: true, faite: true, aRelire: false },
    {
      id: "correspondances",
      libelle: "Correspondances",
      accessible: true,
      faite: etat.correspondances_adoptees,
      aRelire: etat.correspondances_a_relire,
    },
    { id: "analyse", libelle: "Analyse", accessible: etat.correspondances_adoptees, faite: etat.analyse_preparee, aRelire: false },
    {
      id: "decisions",
      libelle: "Décisions",
      accessible: etat.analyse_preparee && etat.propositions.length > 0,
      faite: etat.decisions_adoptees,
      aRelire: etat.decisions_a_relire,
    },
    {
      id: "resultat",
      libelle: "Résultat",
      accessible: etat.tentatives.length > 0,
      faite: etat.etape === "ResultatProduit",
      aRelire: false,
    },
  ]);

  function etatDeLEtape(etape) {
    if (etape.aRelire) return "à relire";
    if (etape.faite) return "terminée";
    if (etape.id === ecran) return "en cours";
    return "";
  }
</script>

<nav class="etapes" aria-label="Étapes du traitement">
  {#each etapes as etape, index}
    <button
      type="button"
      class:faite={etape.faite && !etape.aRelire}
      disabled={!etape.accessible}
      aria-current={etape.id === ecran ? "step" : undefined}
      onclick={() => naviguer(etape.id)}
    >
      <span class="numero" aria-hidden="true">{etape.faite && !etape.aRelire ? "✓" : index + 1}</span>
      <span class="libelle">{etape.libelle}</span>
      <span class="sous">{etatDeLEtape(etape)}</span>
    </button>
  {/each}
</nav>
