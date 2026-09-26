<script>
  import Fichier from "./Fichier.svelte";

  let { etat, naviguer } = $props();
</script>

<h1 class="text-xl leading-snug font-semibold text-balance" tabindex="-1">Source</h1>
<Fichier type="SGX" chemin={etat.source_chemin_original} />
<p class="text-xs text-encre-2">
  L'application travaille sur une copie contrôlée par empreinte, dans le dossier du traitement. L'original peut être
  déplacé sans gêner la reprise.
</p>

<div class="chiffres">
  <div class="chiffre"><b>{etat.modeles_reconnus}</b><span>modèles reconnus</span></div>
  <div class="chiffre"><b>{etat.occurrences_inventoriees}</b><span>occurrences de lanes</span></div>
  <div class="chiffre"><b>{etat.noms_distincts}</b><span>noms distincts</span></div>
</div>

{#if etat.occurrences_inventoriees === 0}
  <p>Les modèles de ce SGX ne contiennent aucune lane nommée : il n'y a rien à harmoniser.</p>
{:else}
  <div class="flex flex-wrap items-center gap-2">
    <button class="bouton bouton-principal" type="button" onclick={() => naviguer("correspondances")}>
      Préparer les correspondances
    </button>
  </div>
{/if}
