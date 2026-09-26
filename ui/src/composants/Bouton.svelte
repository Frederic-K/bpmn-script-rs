<script>
  // Bouton de l'application. `principal` : l'action principale de l'écran.
  // Les autres attributs (onclick, disabled, aria-pressed...) sont transmis tels quels.
  let { principal = false, children, ...attributs } = $props();
</script>

<button
  type="button"
  class="cursor-pointer rounded-md border px-3.5 py-2 text-sm font-semibold disabled:cursor-not-allowed disabled:opacity-45
    {principal
    ? 'border-accent bg-accent text-accent-encre enabled:hover:brightness-110'
    : 'border-trait bg-fond text-encre enabled:hover:bg-surface aria-pressed:bg-accent-doux'}"
  {...attributs}
>
  {@render children()}
</button>
