<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";

  let profileFolders = $state<string[]>([]);
  let sourceProfile = $state("");
  let newProfile = $state("");
  let copyTargetProfile = $state("");
  let status = $state("");
  let error = $state("");

  async function refresh() {
    profileFolders = await backend.eveProfilesList();
  }

  async function copyProfile() {
    if (!sourceProfile.trim() || !newProfile.trim()) return;
    try {
      await backend.eveCopyProfile(sourceProfile.trim(), newProfile.trim());
      status = "Profile copied";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function copyCharacterFiles() {
    if (!sourceProfile.trim() || !copyTargetProfile.trim()) return;
    try {
      await backend.eveCopyCharacterFiles(sourceProfile.trim(), copyTargetProfile.trim());
      status = "Character/user files copied";
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  onMount(async () => {
    await refresh();
  });
</script>

<section class="card">
  <h2>EVE Profiles</h2>
  <p>Copy profile folders and core character/user files (blocked while `exefile` is running).</p>
  {#if status}
    <Alert>
      <AlertTitle>Status</AlertTitle>
      <AlertDescription>{status}</AlertDescription>
    </Alert>
  {/if}
  {#if error}
    <Alert>
      <AlertTitle>Error</AlertTitle>
      <AlertDescription>{error}</AlertDescription>
    </Alert>
  {/if}

  <div style="display:grid; grid-template-columns: 1fr 1fr auto; gap:0.5rem; margin: 0.75rem 0;">
    <Input bind:value={sourceProfile} placeholder="Source profile name" />
    <Input bind:value={newProfile} placeholder="New profile name" />
    <Button onclick={copyProfile}>Copy Profile</Button>
  </div>

  <div style="display:grid; grid-template-columns: 1fr 1fr auto; gap:0.5rem; margin-bottom: 0.75rem;">
    <Input bind:value={sourceProfile} placeholder="Source profile name" />
    <Input bind:value={copyTargetProfile} placeholder="Target profile name" />
    <Button onclick={copyCharacterFiles}>Copy Char/User Files</Button>
  </div>

  <h3>Detected Profiles</h3>
  <ul>
    {#each profileFolders as profile (profile)}
      <li>{profile}</li>
    {/each}
  </ul>
</section>
