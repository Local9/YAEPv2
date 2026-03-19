<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { Profile } from "$models/domain";

  let profiles = $state<Profile[]>([]);
  let newProfileName = $state("");
  let status = $state("");
  let error = $state("");

  async function refreshProfiles() {
    profiles = await backend.getProfiles();
  }

  async function addProfile() {
    if (!newProfileName.trim()) return;
    try {
      await backend.createProfile(newProfileName.trim());
      newProfileName = "";
      error = "";
      status = "Profile created";
      await refreshProfiles();
    } catch (e) {
      error = String(e);
    }
  }

  async function setActive(profileId: number) {
    try {
      await backend.setCurrentProfile(profileId);
      error = "";
      status = "Active profile updated";
      await refreshProfiles();
    } catch (e) {
      error = String(e);
    }
  }

  async function saveHotkey(profileId: number, hotkey: string) {
    try {
      await backend.updateProfileHotkey(profileId, hotkey);
      error = "";
      status = "Hotkey saved";
      await refreshProfiles();
    } catch (e) {
      error = String(e);
    }
  }

  async function removeProfile(profileId: number) {
    try {
      await backend.deleteProfile(profileId);
      error = "";
      status = "Profile deleted";
      await refreshProfiles();
    } catch (e) {
      error = String(e);
    }
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
  {#if error}<p style="color:#ff8f8f;">{error}</p>{/if}
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
