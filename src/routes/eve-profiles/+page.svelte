<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";

  let profileFolders = $state<string[]>([]);

  onMount(async () => {
    profileFolders = await backend.eveProfilesList();
  });
</script>

<section class="card">
  <h2>EVE Profiles</h2>
  <p>EVE profile tool endpoint is wired (copy and ESI flows to expand next).</p>
  <ul>
    {#each profileFolders as profile (profile)}
      <li>{profile}</li>
    {/each}
  </ul>
</section>
