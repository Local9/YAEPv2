<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Select from "$lib/components/ui/select";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
  import SaveIcon from "@lucide/svelte/icons/save";
  import DownloadIcon from "@lucide/svelte/icons/download";
  import UploadIcon from "@lucide/svelte/icons/upload";

  interface Props {
    enableThumbnailDragging: boolean;
    startHidden: boolean;
    diagnosticsLogEnabled: boolean;
    requireAppFocusForHotkeys: boolean;
    theme: string;
    backupBusy: boolean;
    onSave: () => void;
    onExport: () => void;
    onImportClick: () => void;
  }

  let {
    enableThumbnailDragging = $bindable(),
    startHidden = $bindable(),
    diagnosticsLogEnabled = $bindable(false),
    requireAppFocusForHotkeys = $bindable(false),
    theme = $bindable(),
    backupBusy,
    onSave,
    onExport,
    onImportClick,
  }: Props = $props();

  const themeItems = [
    { value: "Dark", label: "Dark" },
    { value: "Light", label: "Light" },
  ];
</script>

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
  <Field>
    <FieldContent class="flex w-full flex-row items-center gap-2 flex-initial!">
      <Checkbox
        id="settings-diagnostics-log-enabled"
        bind:checked={diagnosticsLogEnabled}
        class="cursor-pointer"
      />
      <FieldLabel
        for="settings-diagnostics-log-enabled"
        class="text-muted-foreground mb-0 cursor-pointer leading-snug font-normal"
      >
        Enable Diagnostics Log (portable)
      </FieldLabel>
    </FieldContent>
  </Field>
  <Field>
    <FieldContent class="flex w-full flex-row items-center gap-2 flex-initial!">
      <Checkbox
        id="settings-enable-global-hotkeys"
        bind:checked={requireAppFocusForHotkeys}
        class="cursor-pointer"
      />
      <FieldLabel
        for="settings-enable-global-hotkeys"
        class="text-muted-foreground mb-0 cursor-pointer leading-snug font-normal"
      >
        Enable Hotkeys to work Globally
      </FieldLabel>
    </FieldContent>
  </Field>
  <Field class="max-w-md">
    <FieldLabel class="text-muted-foreground">Theme</FieldLabel>
    <FieldContent>
      <Select.Root type="single" bind:value={theme} items={themeItems}>
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
    <Button onclick={() => onSave()} class="gap-2">
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
        onclick={() => void onExport()}
      >
        <DownloadIcon class="size-4 shrink-0" aria-hidden="true" />
        Export settings
      </Button>
      <Button
        type="button"
        variant="outline"
        class="gap-2"
        disabled={backupBusy}
        onclick={onImportClick}
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
