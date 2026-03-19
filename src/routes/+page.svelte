<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import type { HealthSnapshot } from "$models/domain";

  type ThumbnailEvent = { pid: number; windowTitle: string };
  type FocusEvent = { pid: number | null; windowTitle: string | null };

  let health = $state<HealthSnapshot | null>(null);
  let error = $state("");
  let activeThumbnails = $state<ThumbnailEvent[]>([]);
  let focused = $state<FocusEvent>({ pid: null, windowTitle: null });

  onMount(() => {
    const cleanup: Array<() => void> = [];
    void (async () => {
      cleanup.push(
        await listen<ThumbnailEvent>("thumbnailAdded", (event) => {
          if (activeThumbnails.some((x) => x.pid === event.payload.pid)) {
            return;
          }
          activeThumbnails = [...activeThumbnails, event.payload];
        })
      );
      cleanup.push(
        await listen<ThumbnailEvent>("thumbnailRemoved", (event) => {
          activeThumbnails = activeThumbnails.filter((x) => x.pid !== event.payload.pid);
        })
      );
      cleanup.push(
        await listen<ThumbnailEvent>("thumbnailUpdated", (event) => {
          activeThumbnails = activeThumbnails.map((x) =>
            x.pid === event.payload.pid ? event.payload : x
          );
        })
      );
      cleanup.push(
        await listen<FocusEvent>("focusChanged", (event) => {
          focused = event.payload;
        })
      );

      try {
        health = await backend.health();
      } catch (err) {
        error = err instanceof Error ? err.message : String(err);
      }
    })();

    return () => {
      for (const fn of cleanup) fn();
    };
  });

  async function activateWindow(pid: number) {
    try {
      await backend.activateWindowByPid(pid);
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }
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

  <hr style="margin: 1rem 0;" />
  <h3>Phase 3 Runtime Events</h3>
  <p>Tracked runtime thumbnails: {activeThumbnails.length}</p>
  <p>Focused thumbnail: {focused.windowTitle ?? "none"}</p>
  <ul>
    {#each activeThumbnails as thumb (thumb.pid)}
      <li style="display:flex; gap:0.5rem; align-items:center;">
        <span>{thumb.windowTitle} (PID {thumb.pid})</span>
        <button onclick={() => activateWindow(thumb.pid)}>Activate</button>
      </li>
    {/each}
  </ul>
</section>
