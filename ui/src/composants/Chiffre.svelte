<script>
  let { valeur, libelle } = $props();
</script>

<div class="rounded-md border border-trait px-3 py-2.5">
  <b class="block text-[22px] leading-tight font-bold tabular-nums" data-chiffre>{valeur}</b>
  <span class="text-xs text-encre-2">{libelle}</span>
</div>
