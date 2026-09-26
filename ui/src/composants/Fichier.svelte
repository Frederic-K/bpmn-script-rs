<script>
  import { nomFichier } from "../format.js";

  // Un fichier : type, nom, chemin complet (sélectionnable) et actions éventuelles.
  // Les actions passent à la ligne quand la place manque, sans écraser le nom.
  let { type, chemin, children } = $props();
</script>

<div class="flex flex-wrap items-center gap-x-3 gap-y-1.5 rounded-md border border-trait px-3 py-2.5">
  <span class="rounded border border-accent px-1.5 py-px font-mono text-[11px] font-semibold text-accent">{type}</span>
  <span class="min-w-40 flex-1 font-semibold [overflow-wrap:anywhere]">{nomFichier(chemin)}</span>
  {#if children}
    <div class="flex flex-wrap gap-1.5">{@render children()}</div>
  {/if}
  <span class="basis-full font-mono text-xs text-encre-2 select-text [overflow-wrap:anywhere]">{chemin}</span>
</div>
