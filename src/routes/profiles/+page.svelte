<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { Profile } from "$models/domain";

  let profiles = $state<Profile[]>([]);
  let newProfileName = $state("");
  let status = $state("");

  async function refreshProfiles() {
    profiles = await backend.getProfiles();
  }

  async function addProfile() {
    if (!newProfileName.trim()) return;
    await backend.createProfile(newProfileName.trim());
    newProfileName = "";
    await refreshProfiles();
  }

  async function setActive(profileId: number) {
    await backend.setCurrentProfile(profileId);
    await refreshProfiles();
  }

  async function saveHotkey(profileId: number, hotkey: string) {
    await backend.updateProfileHotkey(profileId, hotkey);
    status = "Hotkey saved";
    await refreshProfiles();
  }

  async function removeProfile(profileId: number) {
    await backend.deleteProfile(profileId);
    await refreshProfiles();
  }

  onMount(async () => {
    await refreshProfiles();
  });
</script>

<section class="card">
  <h2>Profiles</h2>
  <p>Manage profiles, active profile state, and profile switch hotkeys.</p>
  <div style="display:flex; gap:0.5rem; margin: 1rem 0;">
    <input bind:value={newProfileName} placeholder="New profile name" />
    <button onclick={addProfile}>Add</button>
  </div>
  {#if status}<p>{status}</p>{/if}
  <table style="width:100%; border-collapse: collapse;">
    <thead>
      <tr>
        <th style="text-align:left;">Name</th>
        <th style="text-align:left;">Hotkey</th>
        <th style="text-align:left;">Active</th>
        <th style="text-align:left;">Actions</th>
      </tr>
    </thead>
    <tbody>
      {#each profiles as profile (profile.id)}
        <tr>
          <td>{profile.name}</td>
          <td>
            <input
              value={profile.switchHotkey}
              onblur={(e) => saveHotkey(profile.id, (e.currentTarget as HTMLInputElement).value)}
              placeholder="Ctrl+Alt+F1"
            />
          </td>
          <td>{profile.isActive ? "Yes" : "No"}</td>
          <td style="display:flex; gap:0.5rem;">
            <button onclick={() => setActive(profile.id)} disabled={profile.isActive}>Set Active</button>
            <button onclick={() => removeProfile(profile.id)} disabled={profile.isActive}>Delete</button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</section>
