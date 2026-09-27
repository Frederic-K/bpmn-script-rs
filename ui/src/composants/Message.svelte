<script>
  // type : info | ok | alerte | erreur. `children` accueille des actions.
  let { type, etiquette, titre, details = [], children } = $props();

  const COULEURS = {
    info: { fond: "bg-accent-doux", etiquette: "bg-accent text-accent-encre" },
    ok: { fond: "bg-ok-doux", etiquette: "bg-ok text-fond" },
    alerte: { fond: "bg-alerte-doux", etiquette: "bg-alerte text-fond" },
    erreur: { fond: "bg-erreur-doux", etiquette: "bg-erreur text-fond" },
  };
</script>

<div
  class="grid grid-cols-[auto_minmax(0,1fr)] gap-x-2.5 gap-y-1 rounded-md px-3.5 py-3 {COULEURS[type].fond}"
  role={type === "erreur" ? "alert" : "status"}
>
  <span class="mt-0.5 self-start rounded px-1.5 py-0.5 text-[11px] font-bold tracking-wider uppercase {COULEURS[type].etiquette}">
    {etiquette}
  </span>
  <div class="grid min-w-0 gap-1.5">
    <p class="max-w-[72ch]">{titre}</p>
    {#if details.length > 0}
      <ul class="grid list-disc gap-0.5 pl-4.5 text-[13px] select-text wrap-anywhere">
        {#each details as detail}
          <li>{detail}</li>
        {/each}
      </ul>
    {/if}
    {#if children}
      <div class="flex flex-wrap items-center gap-2">{@render children()}</div>
    {/if}
  </div>
</div>
