<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { ClientGroup, Profile } from "$models/domain";

  let profiles = $state<Profile[]>([]);
  let groups = $state<ClientGroup[]>([]);
  let status = $state("");
  let error = $state("");

  async function refresh() {
    profiles = await backend.getProfiles();
    const active = profiles.find((p) => p.isActive);
    if (!active) {
      groups = [];
      return;
    }
    groups = await backend.getClientGroups(active.id);
  }

  async function saveHotkeys(group: ClientGroup) {
    try {
      await backend.updateClientGroupHotkeys(
        group.id,
        group.cycleForwardHotkey,
        group.cycleBackwardHotkey
      );
      status = `Saved hotkeys for ${group.name}`;
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function cycle(group: ClientGroup, direction: "forward" | "backward") {
    try {
      await backend.cycleClientGroup(group.id, direction);
      status = `Cycled ${group.name} (${direction})`;
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  onMount(refresh);
</script>

<section class="card">
  <h2>Client Grouping</h2>
  <p>Client groups for active profile with cycle hotkey and activation baseline.</p>
  {#if status}<p>{status}</p>{/if}
  {#if error}<p style="color:#ff8f8f;">{error}</p>{/if}
  <table style="width:100%; border-collapse: collapse;">
    <thead>
      <tr>
        <th style="text-align:left;">Name</th>
        <th style="text-align:left;">Display Order</th>
        <th style="text-align:left;">Forward Hotkey</th>
        <th style="text-align:left;">Backward Hotkey</th>
        <th style="text-align:left;">Actions</th>
      </tr>
    </thead>
    <tbody>
      {#each groups as group (group.id)}
        <tr>
          <td>{group.name}</td>
          <td>{group.displayOrder}</td>
          <td>
            <input
              bind:value={group.cycleForwardHotkey}
              placeholder="Ctrl+Alt+F13"
              onblur={() => saveHotkeys(group)}
            />
          </td>
          <td>
            <input
              bind:value={group.cycleBackwardHotkey}
              placeholder="Ctrl+Alt+F14"
              onblur={() => saveHotkeys(group)}
            />
          </td>
          <td style="display:flex; gap:0.5rem;">
            <button onclick={() => cycle(group, "forward")}>Cycle Next</button>
            <button onclick={() => cycle(group, "backward")}>Cycle Prev</button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</section>
