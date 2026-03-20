<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { Profile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import CheckIcon from "@lucide/svelte/icons/check";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import UsersIcon from "@lucide/svelte/icons/users";

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

<section class="rounded-lg border border-border bg-card p-4 text-card-foreground shadow-sm">
  <div class="mb-4 flex items-start gap-3">
    <UsersIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
    <div>
      <h2 class="text-lg font-semibold tracking-tight">Profiles</h2>
      <p class="mt-1 text-sm text-muted-foreground">
        Manage profiles, active profile state, and profile switch hotkeys.
      </p>
    </div>
  </div>

  <div class="mb-4 flex flex-wrap items-center gap-2">
    <Input class="max-w-xs min-w-0 flex-1" bind:value={newProfileName} placeholder="New profile name" />
    <Button type="button" onclick={addProfile}>
      <PlusIcon class="size-4 shrink-0" aria-hidden="true" />
      Add
    </Button>
  </div>

  {#if status}
    <p class="mb-2 text-sm text-muted-foreground">{status}</p>
  {/if}
  {#if error}
    <p class="mb-2 flex items-center gap-2 text-sm text-destructive" role="alert">
      <AlertCircleIcon class="size-4 shrink-0" aria-hidden="true" />
      {error}
    </p>
  {/if}

  <div class="overflow-x-auto">
    <table class="w-full border-collapse text-sm">
      <thead>
        <tr class="border-b border-border">
          <th class="p-2 text-left font-medium text-muted-foreground">Name</th>
          <th class="p-2 text-left font-medium text-muted-foreground">Hotkey</th>
          <th class="p-2 text-left font-medium text-muted-foreground">Active</th>
          <th class="p-2 text-left font-medium text-muted-foreground">Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each profiles as profile (profile.id)}
          <tr class="border-b border-border/60">
            <td class="p-2 align-middle">{profile.name}</td>
            <td class="p-2 align-middle">
              <Input
                class="min-w-[8rem]"
                value={profile.switchHotkey}
                onblur={(e) =>
                  saveHotkey(profile.id, (e.currentTarget as HTMLInputElement).value)}
                placeholder="Ctrl+Alt+F1"
              />
            </td>
            <td class="p-2 align-middle">{profile.isActive ? "Yes" : "No"}</td>
            <td class="p-2 align-middle">
              <div class="flex flex-wrap gap-2">
                <Button
                  type="button"
                  variant="outline"
                  onclick={() => setActive(profile.id)}
                  disabled={profile.isActive}
                >
                  <CheckIcon class="size-4 shrink-0" aria-hidden="true" />
                  Set Active
                </Button>
                <Button
                  type="button"
                  variant="destructive"
                  onclick={() => removeProfile(profile.id)}
                  disabled={profile.isActive}
                >
                  <Trash2Icon class="size-4 shrink-0" aria-hidden="true" />
                  Delete
                </Button>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</section>
