<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import { setMode } from "mode-watcher";
  import { toast } from "svelte-sonner";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import AppSettingsForm from "$lib/components/settings/app-settings-form.svelte";
  import { formatYyyyMmDdDash } from "$lib/datetime/format-yyyy-mm-dd";

  let enableThumbnailDragging = $state(true);
  let startHidden = $state(false);
  let theme = $state("Dark");
  let saveStatus = $state("");
  let error = $state("");
  let importConfirmOpen = $state(false);
  let backupBusy = $state(false);

  function userSafeErrorMessage(): string {
    return "Unable to save settings right now. Please try again.";
  }

  async function refresh() {
    try {
      const dragging = await backend.getAppSetting("EnableThumbnailDragging");
      const hidden = await backend.getAppSetting("StartHidden");
      const currentTheme = await backend.getAppSetting("Theme");
      enableThumbnailDragging = dragging == null ? true : dragging === "true";
      startHidden = hidden == null ? false : hidden === "true";
      theme = currentTheme ?? "Dark";
      error = "";
    } catch (e) {
      error = userSafeErrorMessage();
    }
  }

  async function exportYaepSettings() {
    if (backupBusy) return;
    backupBusy = true;
    error = "";
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const defaultPath = `yaep-settings-${formatYyyyMmDdDash(new Date())}.json`;
      const filePath = await save({
        title: "Export YAEP settings",
        defaultPath,
        filters: [{ name: "YAEP settings backup", extensions: ["json"] }],
      });
      if (!filePath) return;
      await backend.yaepExportSettingsToPath(filePath);
      saveStatus = "Settings exported to file.";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      backupBusy = false;
    }
  }

  async function pickAndImportYaepSettings() {
    importConfirmOpen = false;
    if (backupBusy) return;
    backupBusy = true;
    error = "";
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const picked = await open({
        title: "Import YAEP settings",
        filters: [{ name: "YAEP settings backup", extensions: ["json"] }],
        multiple: false,
      });
      if (picked === null) return;
      const path = Array.isArray(picked) ? picked[0] : picked;
      if (!path) return;
      await backend.yaepImportSettingsFromPath(path);
      await refresh();
      setMode(theme === "Light" ? "light" : "dark");
      saveStatus = "Settings imported. Thumbnails, Mumble links, widgets, and profiles were replaced.";
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      backupBusy = false;
    }
  }

  async function save() {
    try {
      await backend.setAppSetting("EnableThumbnailDragging", String(enableThumbnailDragging));
      await backend.setAppSetting("StartHidden", String(startHidden));
      await backend.setAppSetting("Theme", theme);
      setMode(theme === "Light" ? "light" : "dark");
      saveStatus = "Settings saved";
      error = "";
    } catch (e) {
      error = userSafeErrorMessage();
    }
  }

  onMount(refresh);

  $effect(() => {
    if (saveStatus) toast.success(saveStatus);
  });

  $effect(() => {
    if (error) toast.error(error);
  });
</script>

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <SettingsIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Settings</CardTitle>
        <CardDescription>App-level settings with persisted values.</CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
    <AppSettingsForm
      bind:enableThumbnailDragging
      bind:startHidden
      bind:theme
      {backupBusy}
      onSave={() => void save()}
      onExport={exportYaepSettings}
      onImportClick={() => {
        importConfirmOpen = true;
      }}
    />
  </CardContent>
</Card>

<AlertDialog.Root bind:open={importConfirmOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Replace all YAEP settings?</AlertDialog.Title>
      <AlertDialog.Description>
        Importing will overwrite everything stored in YAEP on this PC with the backup file. Export a backup
        first if you need to keep your current setup.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
      <AlertDialog.Action onclick={() => void pickAndImportYaepSettings()}>Import</AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
