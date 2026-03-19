<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { Profile } from "$models/domain";

  let profiles = $state<Profile[]>([]);
  let activeProfileId = $state<number | null>(null);
  let processes = $state<string[]>([]);
  let newProcessName = $state("");
  let status = $state("");
  let error = $state("");

  async function refresh() {
    profiles = await backend.getProfiles();
    const active = profiles.find((p) => p.isActive);
    activeProfileId = active?.id ?? null;
    if (activeProfileId != null) {
      processes = await backend.getProcessesToPreview(activeProfileId);
    } else {
      processes = [];
    }
  }

  async function addProcess() {
    if (activeProfileId == null || !newProcessName.trim()) return;
    try {
      await backend.addProcessToPreview(activeProfileId, newProcessName.trim());
      newProcessName = "";
      status = "Process added";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function removeProcess(name: string) {
    if (activeProfileId == null) return;
    try {
      await backend.removeProcessToPreview(activeProfileId, name);
      status = "Process removed";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  onMount(refresh);
</script>

<section class="card">
  <h2>Process Management</h2>
  <p>Configure process names to scan per active profile (for example, `exefile`).</p>
  <p>
    Active profile:
    <strong>{profiles.find((p) => p.id === activeProfileId)?.name ?? "None"}</strong>
  </p>
  <div style="display:flex; gap:0.5rem; margin-bottom:0.75rem;">
    <input bind:value={newProcessName} placeholder="exefile" />
    <button onclick={addProcess} disabled={activeProfileId == null}>Add process</button>
  </div>
  {#if status}<p>{status}</p>{/if}
  {#if error}<p style="color:#ff8f8f;">{error}</p>{/if}
  <ul>
    {#each processes as process (process)}
      <li style="display:flex; gap:0.5rem; align-items:center; margin-bottom:0.25rem;">
        <code>{process}</code>
        <button onclick={() => removeProcess(process)}>Remove</button>
      </li>
    {/each}
  </ul>
</section>
