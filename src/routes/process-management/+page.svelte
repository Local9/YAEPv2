<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { Profile } from "$models/domain";

  let profiles = $state<Profile[]>([]);
  let activeProfileId = $state<number | null>(null);
  let processes = $state<string[]>([]);
  let newProcessName = $state("");

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
    await backend.addProcessToPreview(activeProfileId, newProcessName.trim());
    newProcessName = "";
    await refresh();
  }

  async function removeProcess(name: string) {
    if (activeProfileId == null) return;
    await backend.removeProcessToPreview(activeProfileId, name);
    await refresh();
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
  <ul>
    {#each processes as process (process)}
      <li style="display:flex; gap:0.5rem; align-items:center; margin-bottom:0.25rem;">
        <code>{process}</code>
        <button onclick={() => removeProcess(process)}>Remove</button>
      </li>
    {/each}
  </ul>
</section>
