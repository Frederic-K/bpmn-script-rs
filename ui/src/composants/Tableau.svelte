<script>
  // Tableau en lecture seule, défilant dans sa zone. `children` : <thead> et <tbody>.
  let { legende, children } = $props();
</script>

<div class="max-h-90 overflow-x-auto rounded-md border border-trait">
  <table
    class="w-full border-collapse text-left text-[13px]
      [&_td]:border-t [&_td]:border-trait [&_td]:px-2.5 [&_td]:py-1.5 [&_td]:align-top
      [&_th]:sticky [&_th]:top-0 [&_th]:bg-surface [&_th]:px-2.5 [&_th]:py-1.5 [&_th]:text-xs [&_th]:font-semibold
      [&_th]:whitespace-nowrap [&_th]:text-encre-2"
  >
    <caption class="py-1 text-xs text-encre-2">{legende}</caption>
    {@render children()}
  </table>
</div>
