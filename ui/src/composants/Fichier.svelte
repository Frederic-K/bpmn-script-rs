<script>
  import { nomFichier } from "../format.js";

  // Un fichier : type, nom, chemin complet (sélectionnable) et actions éventuelles.
  let { type, chemin, children } = $props();
</script>

<div class="grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-x-3 gap-y-1 rounded-md border border-trait px-3 py-2.5">
  <span class="rounded border border-accent px-1.5 py-px font-mono text-[11px] font-semibold text-accent">{type}</span>
  <span class="font-semibold [overflow-wrap:anywhere]">{nomFichier(chemin)}</span>
  <div class="flex flex-wrap justify-end gap-1.5">{@render children?.()}</div>
  <span class="col-span-2 col-start-2 font-mono text-xs text-encre-2 select-text [overflow-wrap:anywhere]">{chemin}</span>
</div>
