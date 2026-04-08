<script lang="ts">
  let { staticDataCatalogEmpty = false }: { staticDataCatalogEmpty?: boolean } = $props();
</script>

<div class="flex flex-col gap-1">
  <h1 class="text-xl font-semibold">PI Templates</h1>
  <p class="text-sm text-muted-foreground">
    {#if staticDataCatalogEmpty}
      Templates load from your PlanetaryInteractionTemplates folder. This view needs EVE static data in SQLite (<span
        class="font-mono">EveSdeTypes</span> / <span class="font-mono">EveSdeGroups</span>) for names, prices, and the map.
      Edges follow material routes (R).
      <span class="block pt-1 text-foreground">
        The types table is empty.
        <a href="/" class="font-medium text-primary underline-offset-4 hover:underline">Open the Dashboard</a>
        to download or import static data.
      </span>
    {:else}
      Templates load from your PlanetaryInteractionTemplates folder. Edges follow material routes (R).
    {/if}
  </p>
</div>
