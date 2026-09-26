<script>
  // type : info | ok | alerte | erreur. `children` accueille des actions.
  let { type, etiquette, titre, details = [], children } = $props();
</script>

<div class="message {type}" role={type === "erreur" ? "alert" : "status"}>
  <span class="etiquette">{etiquette}</span>
  <div class="texte">
    <p>{titre}</p>
    {#if details.length > 0}
      <ul>
        {#each details as detail}
          <li>{detail}</li>
        {/each}
      </ul>
    {/if}
    {#if children}
      <div class="rangee">{@render children()}</div>
    {/if}
  </div>
</div>
