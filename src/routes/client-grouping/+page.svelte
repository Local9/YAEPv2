<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { ClientGroup, Profile } from "$models/domain";

  let profiles = $state<Profile[]>([]);
  let groups = $state<ClientGroup[]>([]);

  async function refresh() {
    profiles = await backend.getProfiles();
    const active = profiles.find((p) => p.isActive);
    if (!active) {
      groups = [];
      return;
    }
    groups = await backend.getClientGroups(active.id);
  }

  onMount(refresh);
</script>

<section class="card">
  <h2>Client Grouping</h2>
  <p>Client groups for active profile (CRUD to expand next).</p>
  <table style="width:100%; border-collapse: collapse;">
    <thead>
      <tr>
        <th style="text-align:left;">Name</th>
        <th style="text-align:left;">Display Order</th>
        <th style="text-align:left;">Forward Hotkey</th>
        <th style="text-align:left;">Backward Hotkey</th>
      </tr>
    </thead>
    <tbody>
      {#each groups as group (group.id)}
        <tr>
          <td>{group.name}</td>
          <td>{group.displayOrder}</td>
          <td>{group.cycleForwardHotkey || "-"}</td>
          <td>{group.cycleBackwardHotkey || "-"}</td>
        </tr>
      {/each}
    </tbody>
  </table>
</section>
