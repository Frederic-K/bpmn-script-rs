<script>
  import { textes } from "../textes.js";
  import Bouton from "./Bouton.svelte";
  import Chiffre from "./Chiffre.svelte";
  import Fichier from "./Fichier.svelte";

  let { etat, naviguer } = $props();
  const texte = textes.source;
</script>

<Fichier type="SGX" chemin={etat.source_chemin_original} />
<p class="text-xs text-encre-2">{texte.copie}</p>

<div class="grid grid-cols-[repeat(auto-fit,minmax(130px,1fr))] gap-2.5">
  <Chiffre valeur={etat.modeles_reconnus} libelle={texte.modeles} />
  <Chiffre valeur={etat.occurrences_inventoriees} libelle={texte.occurrences} />
  <Chiffre valeur={etat.noms_distincts} libelle={texte.noms} />
</div>

{#if etat.occurrences_inventoriees === 0}
  <p>{texte.rienAHarmoniser}</p>
{:else}
  <div><Bouton principal onclick={() => naviguer("correspondances")}>{texte.suivant}</Bouton></div>
{/if}
