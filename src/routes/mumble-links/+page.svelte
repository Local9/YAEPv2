<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { MumbleLink, MumbleServerGroup } from "$models/domain";

  let links = $state<MumbleLink[]>([]);
  let groups = $state<MumbleServerGroup[]>([]);

  async function refresh() {
    links = await backend.getMumbleLinks();
    groups = await backend.getMumbleServerGroups();
  }

  onMount(refresh);
</script>

<section class="card">
  <h2>Mumble Links</h2>
  <p>Links/groups read path is active; editing flows can now be layered in.</p>
  <h3>Groups</h3>
  <ul>
    {#each groups as group (group.id)}
      <li>{group.name}</li>
    {/each}
  </ul>
  <h3>Links</h3>
  <ul>
    {#each links as link (link.id)}
      <li>
        {link.name} - <code>{link.url}</code> {link.hotkey ? `(${link.hotkey})` : ""}
      </li>
    {/each}
  </ul>
</section>
