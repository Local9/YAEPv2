<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { EveDetectedProfile, EveProfileSettingsSources } from "$models/domain";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import Gamepad2Icon from "@lucide/svelte/icons/gamepad-2";
  import { toast } from "svelte-sonner";
  import {
    discoveredServersFromProfiles,
    filterNonFrontierSupported,
    formatYyyyMmDdCompact,
    normalizeServerName,
    resolveSelectedServer,
    sanitizeFileNameSegment,
    splitDirAndFile,
  } from "$lib/eve-profiles/eve-server-helpers";
  import EveProfilesServerToolbar from "$lib/components/eve-profiles/eve-profiles-server-toolbar.svelte";
  import EveProfilesDetectedTable from "$lib/components/eve-profiles/eve-profiles-detected-table.svelte";
  import EveProfileCopyDialog from "$lib/components/eve-profiles/eve-profile-copy-dialog.svelte";
  import EveProfileDeleteDialog from "$lib/components/eve-profiles/eve-profile-delete-dialog.svelte";
  import EveProfileCopySettingsDialog from "$lib/components/eve-profiles/eve-profile-copy-settings-dialog.svelte";

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

  let nonFrontierDetectedProfiles = $derived(filterNonFrontierSupported(detectedProfiles));
  let discoveredServers = $derived(discoveredServersFromProfiles(nonFrontierDetectedProfiles));
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
      copySettingsSources = await backend.eveGetProfileSettingsSources(
        profile.serverName,
        profile.profileName,
      );
    } catch (e) {
      error = String(e);
    } finally {
      isLoadingCopySettingsSources = false;
    }
  }

  async function refresh() {
    const detected = await backend.eveProfilesDetected();
    detectedProfiles = detected;
    selectedServer = resolveSelectedServer(detected, selectedServer);
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
      await backend.eveDeleteProfileOnServer(
        deleteDialogProfile.serverName,
        deleteDialogProfile.profileName,
      );
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

      if (!filePath) return;

      if (!selectedServer) {
        throw new Error("Select a server before backing up");
      }

      const { dir } = splitDirAndFile(filePath);
      if (!dir) {
        throw new Error("Invalid backup destination selected");
      }

      const dateStr = formatYyyyMmDdCompact(new Date());
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
    <EveProfilesServerToolbar
      bind:selectedServer
      {serverSelectItems}
      {normalizeServerName}
      isSubmittingBackup={isSubmittingBackup}
      onBackup={backupAllProfiles}
    />

    <EveProfilesDetectedTable
      profiles={filteredDetectedProfiles}
      onCopy={openCopyDialog}
      onCopySettings={openCopySettingsDialog}
      onDelete={openDeleteDialog}
    />

    <EveProfileCopyDialog
      bind:open={copyDialogOpen}
      profile={copyDialogProfile}
      bind:newName={copyDialogNewName}
      isSubmitting={isSubmittingCopy}
      onSubmit={() => void submitCopyDialog()}
      onOpenChange={(open) => {
        if (!open) resetCopyDialogState();
      }}
    />

    <EveProfileDeleteDialog
      bind:open={deleteDialogOpen}
      profile={deleteDialogProfile}
      isSubmitting={isSubmittingDelete}
      onSubmit={() => void submitDeleteDialog()}
      onOpenChange={(open) => {
        if (!open) resetDeleteDialogState();
      }}
    />

    <EveProfileCopySettingsDialog
      bind:open={copySettingsDialogOpen}
      profile={copySettingsDialogProfile}
      bind:characterId={copySettingsCharacterId}
      bind:userId={copySettingsUserId}
      characterItems={copySettingsCharacterItems}
      userItems={copySettingsUserItems}
      isLoadingSources={isLoadingCopySettingsSources}
      isSubmitting={isSubmittingCopySettings}
      onSubmit={() => void submitCopySettingsDialog()}
      onOpenChange={(open) => {
        if (!open) resetCopySettingsDialogState();
      }}
    />
  </CardContent>
</Card>
