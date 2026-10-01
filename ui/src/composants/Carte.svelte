<script>
  // Bloc encadré : titre, explication, puis le contenu (`children`).
  let { titre, explication, children } = $props();
  const id = $props.id();
</script>

<section class="grid content-start gap-3 rounded-lg border border-trait p-4.5" aria-labelledby={id}>
  <h2 {id} class="text-base font-semibold">{titre}</h2>
  <p class="text-[13px] text-encre-2">{explication}</p>
  {@render children()}
</section>
