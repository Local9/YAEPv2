<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { HealthSnapshot } from "$models/domain";

  let health: HealthSnapshot | null = null;
  let error = "";

  onMount(async () => {
    try {
      health = await backend.health();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  });
</script>

<section class="card">
  <h2>Initial Scaffold Ready</h2>
  <p>
    This is the first pass for the YAEP Tauri + SvelteKit rebuild. Core backend modules and
    frontend routes are scaffolded.
  </p>

  {#if health}
    <p>Backend status: <strong>{health.backendReady ? "ready" : "not ready"}</strong></p>
    <p>Active profile id: {health.activeProfileId ?? "none"}</p>
  {:else if error}
    <p>Backend status: error ({error})</p>
  {:else}
    <p>Checking backend status...</p>
  {/if}
</section>
