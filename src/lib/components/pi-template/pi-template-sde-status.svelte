<script lang="ts">
  let {
    pending,
    error,
    showEmptyCatalogHint
  }: {
    pending: boolean;
    error: string;
    showEmptyCatalogHint: boolean;
  } = $props();
</script>

{#if pending}
  <p class="text-xs text-muted-foreground" aria-live="polite">Loading type names from SDE...</p>
{:else if error}
  <p class="text-xs text-destructive" role="alert">{error}</p>
{:else if showEmptyCatalogHint}
  <p class="text-xs text-muted-foreground">
    No matching rows in the SQLite type catalog for this template. On the Dashboard, open EVE static data and use Import
    into database if JSONL is installed but <span class="font-mono">EveSdeTypes</span> is empty.
  </p>
{/if}
