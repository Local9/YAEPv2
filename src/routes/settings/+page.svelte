<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import { setMode } from "mode-watcher";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Select from "$lib/components/ui/select";
  import { toast } from "svelte-sonner";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import SaveIcon from "@lucide/svelte/icons/save";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import UploadIcon from "@lucide/svelte/icons/upload";

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

  function formatYyyyMmDd(date: Date): string {
    const y = date.getFullYear();
    const m = String(date.getMonth() + 1).padStart(2, "0");
    const d = String(date.getDate()).padStart(2, "0");
    return `${y}-${m}-${d}`;
  }

  async function exportYaepSettings() {
    if (backupBusy) return;
    backupBusy = true;
    error = "";
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const defaultPath = `yaep-settings-${formatYyyyMmDd(new Date())}.json`;
      const filePath = await save({
        title: "Export YAEP settings",
        defaultPath,
        filters: [{ name: "YAEP settings backup", extensions: ["json"] }]
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
        multiple: false
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
    <div class="mt-4 grid max-w-3xl gap-3">
      <Field>
        <FieldContent class="flex w-full flex-row items-center gap-2 flex-initial!">
          <Checkbox
            id="settings-enable-thumbnail-dragging"
            bind:checked={enableThumbnailDragging}
            class="cursor-pointer"
          />
          <FieldLabel
            for="settings-enable-thumbnail-dragging"
            class="text-muted-foreground mb-0 cursor-pointer leading-snug font-normal"
          >
            Enable Thumbnail Dragging
          </FieldLabel>
        </FieldContent>
      </Field>
      <Field>
        <FieldContent class="flex w-full flex-row items-center gap-2 flex-initial!">
          <Checkbox id="settings-start-hidden" bind:checked={startHidden} class="cursor-pointer" />
          <FieldLabel
            for="settings-start-hidden"
            class="text-muted-foreground mb-0 cursor-pointer leading-snug font-normal"
          >
            Start Hidden
          </FieldLabel>
        </FieldContent>
      </Field>
      <Field class="max-w-md">
        <FieldLabel class="text-muted-foreground">Theme</FieldLabel>
        <FieldContent>
          <Select.Root
            type="single"
            bind:value={theme}
            items={[
              { value: "Dark", label: "Dark" },
              { value: "Light", label: "Light" },
            ]}
          >
            <Select.Trigger class="w-full">
              <span data-slot="select-value">{theme}</span>
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="Dark">Dark</Select.Item>
              <Select.Item value="Light">Light</Select.Item>
            </Select.Content>
          </Select.Root>
        </FieldContent>
      </Field>
      <div>
        <Button onclick={save} class="gap-2">
          <SaveIcon class="size-4 shrink-0" aria-hidden="true" />
          Save settings
        </Button>
      </div>
      <Field class="max-w-3xl pt-4">
        <FieldLabel class="text-foreground font-medium">Backup and restore</FieldLabel>
        <FieldContent class="mt-2 flex flex-row flex-nowrap items-center gap-2">
          <Button
            type="button"
            variant="outline"
            class="gap-2"
            disabled={backupBusy}
            onclick={() => void exportYaepSettings()}
          >
            <DownloadIcon class="size-4 shrink-0" aria-hidden="true" />
            Export settings
          </Button>
          <Button
            type="button"
            variant="outline"
            class="gap-2"
            disabled={backupBusy}
            onclick={() => {
              importConfirmOpen = true;
            }}
          >
            <UploadIcon class="size-4 shrink-0" aria-hidden="true" />
            Import settings
          </Button>
        </FieldContent>
        <p class="text-muted-foreground mt-2 max-w-2xl text-sm leading-snug">
          Saves or replaces all YAEP data in this app: profiles, thumbnails, Mumble links and overlay,
          widget overlay, client groups, EVE log paths and chat channels, theme, and other app settings.
          Import cannot be undone; export a backup first.
        </p>
      </Field>
      <div class="pt-2">
        <a class="text-sm text-primary underline underline-offset-2" href="/settings/eve-logs">
          Open EVE log settings
        </a>
      </div>
    </div>
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
