<script>
  // Statut court, toujours écrit en toutes lettres. couleur : ok | alerte | erreur | neutre.
  let { couleur = "neutre", children } = $props();

  const COULEURS = {
    ok: "bg-ok-doux text-ok",
    alerte: "bg-alerte-doux text-alerte",
    erreur: "bg-erreur-doux text-erreur",
    neutre: "bg-neutre-doux text-encre-2",
  };
</script>

<span
  class="inline-flex items-center gap-1.5 justify-self-start rounded-full px-2.5 py-0.5 text-xs font-semibold
    before:size-1.75 before:flex-none before:rounded-full before:bg-current before:content-['']
    {COULEURS[couleur]}"
>
  {@render children()}
</span>
