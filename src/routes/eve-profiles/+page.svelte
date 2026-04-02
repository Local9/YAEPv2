<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { EveDetectedProfile, EveProfileSettingsSources } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select";
  import * as Dialog from "$lib/components/ui/dialog";
  import { toast } from "svelte-sonner";
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
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import Gamepad2Icon from "@lucide/svelte/icons/gamepad-2";
  import Settings2Icon from "@lucide/svelte/icons/settings-2";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";

  let detectedProfiles = $state<EveDetectedProfile[]>([]);
  let selectedServer = $state("");
  let status = $state("");
  let error = $state("");
  let isSubmittingCopy = $state(false);
  let isSubmittingDelete = $state(false);
  let isSubmittingCopySettings = $state(false);
  let isSubmittingBackup = $state(false);

  let copyDialogOpen = $state(false);
  let copyDialogProfile = $state<EveDetectedProfile | null>(null);
  let copyDialogNewName = $state("");

  let deleteDialogOpen = $state(false);
  let deleteDialogProfile = $state<EveDetectedProfile | null>(null);

  let copySettingsDialogOpen = $state(false);
  let copySettingsDialogProfile = $state<EveDetectedProfile | null>(null);
  let copySettingsSources = $state<EveProfileSettingsSources | null>(null);
  let copySettingsCharacterId = $state("");
  let copySettingsUserId = $state("");
  let isLoadingCopySettingsSources = $state(false);

  function isFrontierServer(serverName: string): boolean {
    return serverName.toLowerCase().includes("frontier");
  }
  function isTranquilityServer(serverName: string): boolean {
    const serverCode = getServerCode(serverName);
    return serverCode === "tq" || serverCode === "tranquility";
  }
  function getServerCode(serverName: string): string {
    const normalized = serverName.trim().toLowerCase();
    if (!normalized) return "";
    const parts = normalized.split(/[^a-z0-9]+/).filter((part) => part.length > 0);
    return parts.length > 0 ? parts[parts.length - 1] : normalized;
  }
  function normalizeServerName(serverName: string): string {
    const serverCode = getServerCode(serverName);
    if (serverCode === "tq") {
      return "Tranquility (TQ)";
    }
    if (serverCode === "tranquility") {
      return "Tranquility (TQ)";
    }
    if (serverCode === "sisi") {
      return "Singularity (SQ)";
    }
    if (serverCode === "singularity") {
      return "Singularity (SQ)";
    }
    if (!serverCode) {
      return serverName;
    }
    return serverCode.charAt(0).toUpperCase() + serverCode.slice(1).toLowerCase();
  }
  function isSupportedServer(serverName: string): boolean {
    const serverCode = getServerCode(serverName);
    return (
      serverCode === "tq" ||
      serverCode === "tranquility" ||
      serverCode === "sisi" ||
      serverCode === "singularity"
    );
  }
  let nonFrontierDetectedProfiles = $derived(
    detectedProfiles.filter(
      (profile) => !isFrontierServer(profile.serverName) && isSupportedServer(profile.serverName),
    ),
  );
  let discoveredServers = $derived(
    [...new Set(nonFrontierDetectedProfiles.map((profile) => profile.serverName))]
      .filter((serverName) => serverName.trim().length > 0)
      .sort((a, b) => a.localeCompare(b)),
  );
  let serverSelectItems = $derived(
    discoveredServers.map((serverName) => ({
      value: serverName,
      label: normalizeServerName(serverName),
    })),
  );
  let filteredDetectedProfiles = $derived(
    nonFrontierDetectedProfiles.filter((profile) => profile.serverName === selectedServer),
  );
  let copySettingsCharacterItems = $derived(
    (copySettingsSources?.characters ?? []).map((item) => ({
      value: item.characterId,
      label: item.characterId,
    })),
  );
  let copySettingsUserItems = $derived(
    (copySettingsSources?.users ?? []).map((item) => ({
      value: item.userId,
      label: item.userId,
    })),
  );

  function sanitizeFileNameSegment(value: string): string {
    return value
      .trim()
      // Windows-invalid filename characters.
      .replace(/[<>:"/\\|?*\u0000-\u001F]/g, "_");
  }

  function formatYyyyMmDd(date: Date): string {
    const yyyy = date.getFullYear();
    const mm = String(date.getMonth() + 1).padStart(2, "0");
    const dd = String(date.getDate()).padStart(2, "0");
    return `${yyyy}${mm}${dd}`;
  }

  function splitDirAndFile(path: string): { dir: string; file: string } {
    const lastSlash = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
    if (lastSlash < 0) return { dir: "", file: path };
    return { dir: path.slice(0, lastSlash), file: path.slice(lastSlash + 1) };
  }

  async function refresh() {
    const detected = await backend.eveProfilesDetected();
    detectedProfiles = detected;
    if (!selectedServer) {
      const tranquility = detected
        .map((profile) => profile.serverName)
        .find((name) => !isFrontierServer(name) && isSupportedServer(name) && isTranquilityServer(name));
      if (tranquility) {
        selectedServer = tranquility;
      }
    }
    const selectedServerIsValid = detected.some(
      (profile) =>
        profile.serverName === selectedServer &&
        !isFrontierServer(profile.serverName) &&
        isSupportedServer(profile.serverName),
    );
    if (!selectedServerIsValid) {
      const tranquility = detected
        .map((profile) => profile.serverName)
        .find((name) => !isFrontierServer(name) && isSupportedServer(name) && isTranquilityServer(name));
      if (tranquility) {
        selectedServer = tranquility;
      } else {
        const singularity = detected
          .map((profile) => profile.serverName)
          .find(
            (name) =>
              !isFrontierServer(name) &&
              isSupportedServer(name) &&
              (getServerCode(name) === "sisi" || getServerCode(name) === "singularity"),
          );
        selectedServer = singularity ?? "";
      }
    }
  }

  function resetCopyDialogState() {
    copyDialogProfile = null;
    copyDialogNewName = "";
    isSubmittingCopy = false;
  }

  function resetDeleteDialogState() {
    deleteDialogProfile = null;
    isSubmittingDelete = false;
  }

  function resetCopySettingsDialogState() {
    copySettingsDialogProfile = null;
    copySettingsSources = null;
    copySettingsCharacterId = "";
    copySettingsUserId = "";
    isLoadingCopySettingsSources = false;
    isSubmittingCopySettings = false;
  }

  function openCopyDialog(profile: EveDetectedProfile) {
    copyDialogProfile = profile;
    copyDialogNewName = "";
    copyDialogOpen = true;
  }

  function openDeleteDialog(profile: EveDetectedProfile) {
    deleteDialogProfile = profile;
    deleteDialogOpen = true;
  }

  async function openCopySettingsDialog(profile: EveDetectedProfile) {
    copySettingsDialogProfile = profile;
    copySettingsDialogOpen = true;
    copySettingsCharacterId = "";
    copySettingsUserId = "";
    copySettingsSources = null;
    isLoadingCopySettingsSources = true;
    try {
      copySettingsSources = await backend.eveGetProfileSettingsSources(profile.serverName, profile.profileName);
    } catch (e) {
      error = String(e);
    } finally {
      isLoadingCopySettingsSources = false;
    }
  }

  async function submitCopyDialog() {
    if (!copyDialogProfile || !copyDialogNewName.trim()) return;
    try {
      isSubmittingCopy = true;
      await backend.eveCopyProfileOnServer(
        copyDialogProfile.serverName,
        copyDialogProfile.profileName,
        copyDialogNewName.trim(),
      );
      status = "Profile copied";
      error = "";
      copyDialogOpen = false;
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      isSubmittingCopy = false;
    }
  }

  async function submitDeleteDialog() {
    if (!deleteDialogProfile) return;
    try {
      isSubmittingDelete = true;
      await backend.eveDeleteProfileOnServer(deleteDialogProfile.serverName, deleteDialogProfile.profileName);
      status = "Profile deleted";
      error = "";
      deleteDialogOpen = false;
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      isSubmittingDelete = false;
    }
  }

  async function submitCopySettingsDialog() {
    if (!copySettingsDialogProfile || !copySettingsCharacterId || !copySettingsUserId) return;
    try {
      isSubmittingCopySettings = true;
      await backend.eveCopyProfileSettingsFromSources(
        copySettingsDialogProfile.serverName,
        copySettingsDialogProfile.profileName,
        copySettingsCharacterId,
        copySettingsUserId,
      );
      status = "Profile settings copied";
      error = "";
      copySettingsDialogOpen = false;
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      isSubmittingCopySettings = false;
    }
  }

  async function backupAllProfiles() {
    try {
      isSubmittingBackup = true;
      status = "";
      error = "";

      // Lazy import to avoid SSR-time issues with Tauri APIs.
      const { save } = await import("@tauri-apps/plugin-dialog");

      const filePath = await save({
        title: "Save EVE profiles backup",
        filters: [
          {
            name: "EVE profiles backup",
            extensions: ["zip"],
          },
        ],
      });

      if (!filePath) return; // user cancelled

      if (!selectedServer) {
        throw new Error("Select a server before backing up");
      }

      const { dir } = splitDirAndFile(filePath);
      if (!dir) {
        throw new Error("Invalid backup destination selected");
      }

      const dateStr = formatYyyyMmDd(new Date());
      const sanitizedServer = sanitizeFileNameSegment(selectedServer);
      const formattedFileName = `${sanitizedServer}_eveProfiles_${dateStr}.zip`;
      const separator = dir.includes("\\") ? "\\" : "/";
      const outputPath = `${dir}${separator}${formattedFileName}`;

      await backend.eveBackupAllProfiles(selectedServer, outputPath);

      status = `EVE profiles backed up: ${normalizeServerName(selectedServer)}`;
      error = "";
    } catch (e) {
      error = String(e);
    } finally {
      isSubmittingBackup = false;
    }
  }

  onMount(refresh);

  $effect(() => {
    if (status) toast.success(status);
  });

  $effect(() => {
    if (error) toast.error(error);
  });
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
    <div class="mt-6 flex items-center gap-2 text-sm font-medium text-muted-foreground">
      <FolderOpenIcon class="size-4 shrink-0" aria-hidden="true" />
      <h3 class="text-base font-semibold text-foreground">Detected Profiles</h3>
    </div>
    <div class="mt-3 flex flex-wrap items-end gap-2">
      <div class="max-w-sm flex-1">
      <Field>
        <FieldLabel class="text-muted-foreground">Server</FieldLabel>
        <FieldContent>
          <Select.Root type="single" bind:value={selectedServer} items={serverSelectItems}>
            <Select.Trigger class="w-full">
              <span data-slot="select-value">
                {selectedServer ? normalizeServerName(selectedServer) : "Select server"}
              </span>
            </Select.Trigger>
            <Select.Content>
              {#each serverSelectItems as item (item.value)}
                <Select.Item value={item.value} label={item.label}>{item.label}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </FieldContent>
      </Field>
      </div>
      <Button
        type="button"
        onclick={() => void backupAllProfiles()}
        disabled={!selectedServer || isSubmittingBackup}
      >
        {isSubmittingBackup ? "Backing up..." : "Back up"}
      </Button>
    </div>

    <div class="mt-3 overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Profile</TableHead>
            <TableHead>Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {#if filteredDetectedProfiles.length === 0}
            <TableRow>
              <TableCell colspan={2} class="text-muted-foreground">
                No profiles detected under %LOCALAPPDATA%\CCP\EVE
              </TableCell>
            </TableRow>
          {:else}
            {#each filteredDetectedProfiles as profile (profile.fullPath)}
              <TableRow>
                <TableCell>{profile.profileName}</TableCell>
                <TableCell>
                  <div class="flex flex-wrap gap-2">
                    <Button type="button" variant="outline" class="gap-1.5" onclick={() => openCopyDialog(profile)}>
                      <CopyIcon class="size-4 shrink-0" aria-hidden="true" />
                      Copy
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      class="gap-1.5"
                      onclick={() => void openCopySettingsDialog(profile)}
                    >
                      <Settings2Icon class="size-4 shrink-0" aria-hidden="true" />
                      Copy Settings
                    </Button>
                    <Button
                      type="button"
                      variant="destructive"
                      class="gap-1.5"
                      onclick={() => openDeleteDialog(profile)}
                    >
                      <Trash2Icon class="size-4 shrink-0" aria-hidden="true" />
                      Delete
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
            {/each}
          {/if}
        </TableBody>
      </Table>
    </div>

    <Dialog.Root
      bind:open={copyDialogOpen}
      onOpenChange={(open) => {
        if (!open) resetCopyDialogState();
      }}
    >
      <Dialog.Content class="sm:max-w-md">
        <Dialog.Header>
          <Dialog.Title>Copy profile</Dialog.Title>
          <Dialog.Description>
            Copy `{copyDialogProfile?.profileName}` on `{copyDialogProfile?.serverName}` to a new folder/profile
            name.
          </Dialog.Description>
        </Dialog.Header>
        <div class="grid gap-2">
          <label class="text-muted-foreground text-xs font-medium" for="copy-profile-new-name">
            New folder/profile name
          </label>
          <Input
            id="copy-profile-new-name"
            bind:value={copyDialogNewName}
            placeholder="New profile name"
            onkeydown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                void submitCopyDialog();
              }
            }}
          />
        </div>
        <Dialog.Footer>
          <Button type="button" variant="outline" onclick={() => (copyDialogOpen = false)}>Cancel</Button>
          <Button
            type="button"
            onclick={submitCopyDialog}
            disabled={!copyDialogProfile || !copyDialogNewName.trim() || isSubmittingCopy}
          >
            {isSubmittingCopy ? "Copying..." : "Copy"}
          </Button>
        </Dialog.Footer>
      </Dialog.Content>
    </Dialog.Root>

    <Dialog.Root
      bind:open={deleteDialogOpen}
      onOpenChange={(open) => {
        if (!open) resetDeleteDialogState();
      }}
    >
      <Dialog.Content class="sm:max-w-md">
        <Dialog.Header>
          <Dialog.Title>Delete profile</Dialog.Title>
          <Dialog.Description>
            Delete `{deleteDialogProfile?.profileName}` on `{deleteDialogProfile?.serverName}`.
          </Dialog.Description>
        </Dialog.Header>
        <Dialog.Footer>
          <Button type="button" variant="outline" onclick={() => (deleteDialogOpen = false)}>Cancel</Button>
          <Button
            type="button"
            variant="destructive"
            onclick={submitDeleteDialog}
            disabled={!deleteDialogProfile || isSubmittingDelete}
          >
            {isSubmittingDelete ? "Deleting..." : "Delete"}
          </Button>
        </Dialog.Footer>
      </Dialog.Content>
    </Dialog.Root>

    <Dialog.Root
      bind:open={copySettingsDialogOpen}
      onOpenChange={(open) => {
        if (!open) resetCopySettingsDialogState();
      }}
    >
      <Dialog.Content class="sm:max-w-md">
        <Dialog.Header>
          <Dialog.Title>Copy profile settings</Dialog.Title>
          <Dialog.Description>
            Select source character and source user for `{copySettingsDialogProfile?.profileName}` on
            `{copySettingsDialogProfile?.serverName}`.
          </Dialog.Description>
        </Dialog.Header>
        {#if isLoadingCopySettingsSources}
          <p class="text-sm text-muted-foreground">Loading settings sources...</p>
        {:else}
          <div class="grid gap-4">
            <Field>
              <FieldLabel class="text-muted-foreground">Source character</FieldLabel>
              <FieldContent>
                <Select.Root
                  type="single"
                  bind:value={copySettingsCharacterId}
                  items={copySettingsCharacterItems}
                  disabled={copySettingsCharacterItems.length === 0}
                >
                  <Select.Trigger class="w-full">
                    <span data-slot="select-value">
                      {copySettingsCharacterId || "Select source character"}
                    </span>
                  </Select.Trigger>
                  <Select.Content>
                    {#each copySettingsCharacterItems as item (item.value)}
                      <Select.Item value={item.value} label={item.label}>{item.label}</Select.Item>
                    {/each}
                  </Select.Content>
                </Select.Root>
                {#if copySettingsCharacterItems.length === 0}
                  <p class="mt-1 text-xs text-muted-foreground">No character settings sources found.</p>
                {/if}
              </FieldContent>
            </Field>

            <Field>
              <FieldLabel class="text-muted-foreground">Source user</FieldLabel>
              <FieldContent>
                <Select.Root
                  type="single"
                  bind:value={copySettingsUserId}
                  items={copySettingsUserItems}
                  disabled={copySettingsUserItems.length === 0}
                >
                  <Select.Trigger class="w-full">
                    <span data-slot="select-value">{copySettingsUserId || "Select source user"}</span>
                  </Select.Trigger>
                  <Select.Content>
                    {#each copySettingsUserItems as item (item.value)}
                      <Select.Item value={item.value} label={item.label}>{item.label}</Select.Item>
                    {/each}
                  </Select.Content>
                </Select.Root>
                {#if copySettingsUserItems.length === 0}
                  <p class="mt-1 text-xs text-muted-foreground">No user settings sources found.</p>
                {/if}
              </FieldContent>
            </Field>
          </div>
        {/if}
        <Dialog.Footer>
          <Button type="button" variant="outline" onclick={() => (copySettingsDialogOpen = false)}>Cancel</Button>
          <Button
            type="button"
            onclick={submitCopySettingsDialog}
            disabled={
              !copySettingsDialogProfile ||
              !copySettingsCharacterId ||
              !copySettingsUserId ||
              isLoadingCopySettingsSources ||
              isSubmittingCopySettings
            }
          >
            {isSubmittingCopySettings ? "Copying..." : "Copy Settings"}
          </Button>
        </Dialog.Footer>
      </Dialog.Content>
    </Dialog.Root>
  </CardContent>
</Card>
