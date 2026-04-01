<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import { save } from "@tauri-apps/plugin-dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import CheckCircle2Icon from "@lucide/svelte/icons/check-circle-2";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import Gamepad2Icon from "@lucide/svelte/icons/gamepad-2";
  import ListIcon from "@lucide/svelte/icons/list";

  let profileFolders = $state<string[]>([]);
  let sourceProfile = $state("");
  let newProfile = $state("");
  let copyTargetProfile = $state("");
  let status = $state("");
  let error = $state("");
  let isBackingUp = $state(false);
  let profileSelectItems = $derived(profileFolders.map((profile) => ({ value: profile, label: profile })));

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

  async function backupAllProfiles() {
    try {
      isBackingUp = true;
      const backupPath = await save({
        title: "Save EVE Profiles Backup",
        defaultPath: `eve-profiles-backup-${new Date().toISOString().slice(0, 10)}.zip`,
        filters: [{ name: "ZIP archive", extensions: ["zip"] }],
      });
      if (!backupPath) {
        return;
      }
      await backend.eveBackupAllProfiles(backupPath);
      status = `Backup saved to ${backupPath}`;
      error = "";
    } catch (e) {
      error = String(e);
    } finally {
      isBackingUp = false;
    }
  }

  onMount(refresh);
</script>

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <Gamepad2Icon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">EVE Profiles</CardTitle>
        <CardDescription>
          Copy profile folders and core character/user files (blocked while <code
            class="rounded bg-muted px-1 font-mono text-xs">exefile</code> is running).
        </CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
    {#if status}
      <Alert class="border-primary/30 bg-primary/5">
        <CheckCircle2Icon class="size-4 text-primary" aria-hidden="true" />
        <AlertTitle>Status</AlertTitle>
        <AlertDescription>{status}</AlertDescription>
      </Alert>
    {/if}
    {#if error}
      <Alert variant="destructive">
        <AlertCircleIcon class="size-4" aria-hidden="true" />
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    {/if}

    <div class="mt-4 grid gap-2 sm:grid-cols-[1fr_1fr_auto] sm:items-end">
      <Field>
        <FieldLabel class="text-muted-foreground">Source profile</FieldLabel>
        <FieldContent>
          <Select.Root type="single" bind:value={sourceProfile} items={profileSelectItems}>
            <Select.Trigger class="w-full">
              <span data-slot="select-value">{sourceProfile || "Select source profile"}</span>
            </Select.Trigger>
            <Select.Content class="max-h-72 overflow-y-auto">
              {#each profileFolders as profile (profile)}
                <Select.Item value={profile} label={profile}>{profile}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </FieldContent>
      </Field>
      <Field>
        <FieldLabel class="text-muted-foreground">New profile</FieldLabel>
        <FieldContent>
          <Input bind:value={newProfile} placeholder="New profile name" />
        </FieldContent>
      </Field>
      <Button onclick={copyProfile} class="gap-2 sm:mb-0">
        <CopyIcon class="size-4 shrink-0" aria-hidden="true" />
        Copy Profile
      </Button>
    </div>

    <div class="mt-4 grid gap-2 sm:grid-cols-[1fr_1fr_auto] sm:items-end">
      <Field>
        <FieldLabel class="text-muted-foreground">Source profile</FieldLabel>
        <FieldContent>
          <Select.Root type="single" bind:value={sourceProfile} items={profileSelectItems}>
            <Select.Trigger class="w-full">
              <span data-slot="select-value">{sourceProfile || "Select source profile"}</span>
            </Select.Trigger>
            <Select.Content class="max-h-72 overflow-y-auto">
              {#each profileFolders as profile (profile)}
                <Select.Item value={profile} label={profile}>{profile}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </FieldContent>
      </Field>
      <Field>
        <FieldLabel class="text-muted-foreground">Target profile</FieldLabel>
        <FieldContent>
          <Select.Root type="single" bind:value={copyTargetProfile} items={profileSelectItems}>
            <Select.Trigger class="w-full">
              <span data-slot="select-value">{copyTargetProfile || "Select target profile"}</span>
            </Select.Trigger>
            <Select.Content class="max-h-72 overflow-y-auto">
              {#each profileFolders as profile (profile)}
                <Select.Item value={profile} label={profile}>{profile}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </FieldContent>
      </Field>
      <Button onclick={copyCharacterFiles} variant="secondary" class="gap-2 sm:mb-0">
        <CopyIcon class="size-4 shrink-0" aria-hidden="true" />
        Copy Char/User Files
      </Button>
    </div>

    <div class="mt-4">
      <Button onclick={backupAllProfiles} variant="outline" class="gap-2" disabled={isBackingUp}>
        <DownloadIcon class="size-4 shrink-0" aria-hidden="true" />
        {isBackingUp ? "Creating backup..." : "Backup All Profiles"}
      </Button>
    </div>

    <div class="mt-6 flex items-center gap-2 text-sm font-medium text-muted-foreground">
      <FolderOpenIcon class="size-4 shrink-0" aria-hidden="true" />
      <h3 class="text-base font-semibold text-foreground">Detected Profiles</h3>
    </div>
    <ul class="mt-3 space-y-1.5 text-sm">
      {#each profileFolders as profile (profile)}
        <li class="flex items-center gap-2 rounded-md border border-border/60 bg-muted/30 px-3 py-2">
          <ListIcon class="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
          {profile}
        </li>
      {/each}
    </ul>
  </CardContent>
</Card>
