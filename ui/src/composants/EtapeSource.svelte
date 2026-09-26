<script>
  import Bouton from "./Bouton.svelte";
  import Chiffre from "./Chiffre.svelte";
  import Fichier from "./Fichier.svelte";

  let { etat, naviguer } = $props();
</script>

<h1 class="text-xl leading-snug font-semibold text-balance" tabindex="-1">Source</h1>
<Fichier type="SGX" chemin={etat.source_chemin_original} />
<p class="text-xs text-encre-2">
  L'application travaille sur une copie contrôlée par empreinte, dans le dossier du traitement. L'original peut être
  déplacé sans gêner la reprise.
</p>

<div class="grid grid-cols-[repeat(auto-fit,minmax(130px,1fr))] gap-2.5">
  <Chiffre valeur={etat.modeles_reconnus} libelle="modèles reconnus" />
  <Chiffre valeur={etat.occurrences_inventoriees} libelle="occurrences de lanes" />
  <Chiffre valeur={etat.noms_distincts} libelle="noms distincts" />
</div>

{#if etat.occurrences_inventoriees === 0}
  <p>Les modèles de ce SGX ne contiennent aucune lane nommée : il n'y a rien à harmoniser.</p>
{:else}
  <div><Bouton principal onclick={() => naviguer("correspondances")}>Préparer les correspondances</Bouton></div>
{/if}
