<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { Profile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import CheckCircle2Icon from "@lucide/svelte/icons/check-circle-2";
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

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <UsersIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Profiles</CardTitle>
        <CardDescription>
          Manage profiles, active profile state, and profile switch hotkeys.
        </CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
    <div class="mb-4 flex flex-wrap items-center gap-2">
      <Input class="max-w-xs min-w-0 flex-1" bind:value={newProfileName} placeholder="New profile name" />
      <Button type="button" onclick={addProfile}>
        <PlusIcon class="size-4 shrink-0" aria-hidden="true" />
        Add
      </Button>
    </div>

    {#if status}
      <Alert class="border-primary/30 bg-primary/5">
        <CheckCircle2Icon class="size-4 text-primary" aria-hidden="true" />
        <AlertTitle>Status</AlertTitle>
        <AlertDescription>{status}</AlertDescription>
      </Alert>
    {/if}
    {#if error}
      <Alert variant="destructive">
        <AlertCircleIcon class="size-4 shrink-0" aria-hidden="true" />
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    {/if}

    <div class="mt-4 overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Hotkey</TableHead>
            <TableHead>Active</TableHead>
            <TableHead>Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {#each profiles as profile (profile.id)}
            <TableRow>
              <TableCell>{profile.name}</TableCell>
              <TableCell>
                <Input
                  class="min-w-[8rem]"
                  value={profile.switchHotkey}
                  onblur={(e) =>
                    saveHotkey(profile.id, (e.currentTarget as HTMLInputElement).value)}
                  placeholder="Ctrl+Alt+F1"
                />
              </TableCell>
              <TableCell>{profile.isActive ? "Yes" : "No"}</TableCell>
              <TableCell>
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
              </TableCell>
            </TableRow>
          {/each}
        </TableBody>
      </Table>
    </div>
  </CardContent>
</Card>
