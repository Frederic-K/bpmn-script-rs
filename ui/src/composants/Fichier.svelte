<script>
  import { nomFichier } from "../format.js";

  // Un fichier : type, nom, chemin complet (sélectionnable) et actions éventuelles.
  let { type, chemin, children } = $props();
</script>

<div class="fichier">
  <span class="type">{type}</span>
  <span class="nom">{nomFichier(chemin)}</span>
  <div class="actions">{@render children?.()}</div>
  <span class="chemin">{chemin}</span>
</div>
