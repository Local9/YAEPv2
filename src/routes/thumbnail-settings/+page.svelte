<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import type { Profile, ThumbnailConfig, ThumbnailSetting } from "$models/domain";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Separator } from "$lib/components/ui/separator";
  import ImageIcon from "@lucide/svelte/icons/image";
  import { toast } from "svelte-sonner";
  import DefaultThumbnailLayoutForm from "$lib/components/thumbnail-settings/default-thumbnail-layout-form.svelte";
  import ThumbnailOverridesSection from "$lib/components/thumbnail-settings/thumbnail-overrides-section.svelte";
  import CopyThumbnailSettingsDialog from "$lib/components/thumbnail-settings/copy-thumbnail-settings-dialog.svelte";
  import {
    parseOptionalCharacterId,
    parseOptionalCharacterIdFromText,
    validatePositiveThumbnailDimensions,
  } from "$lib/thumbnail/thumbnail-setting-validation";

  let profiles = $state<Profile[]>([]);
  let activeProfileId = $state<number | null>(null);
  let defaultConfig = $state<ThumbnailConfig | null>(null);
  let settings = $state<ThumbnailSetting[]>([]);
  let windowTitle = $state("");
  let characterIdText = $state("");
  let saveMessage = $state("");
  let error = $state("");
  let selectedTemplateTitle = $state("");
  let selectedAspectRatio = $state<string>("16:9");
  let copyDialogOpen = $state(false);
  let copyTargetWindowTitle = $state("");

  function cloneDefault(): ThumbnailConfig | null {
    if (!defaultConfig) return null;
    return { ...defaultConfig };
  }

  async function refresh() {
    profiles = await backend.getProfiles();
    activeProfileId = profiles.find((p) => p.isActive)?.id ?? null;
    if (activeProfileId == null) return;
    defaultConfig = await backend.getThumbnailDefaultConfig(activeProfileId);
    settings = await backend.getThumbnailSettings(activeProfileId);
  }

  async function saveDefault() {
    if (activeProfileId == null || defaultConfig == null) return;
    try {
      await backend.setThumbnailDefaultConfig(activeProfileId, defaultConfig);
      saveMessage = "Default config saved";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function addOrUpdateWindowOverride() {
    if (activeProfileId == null || !windowTitle.trim()) return;
    const config = cloneDefault();
    if (!config) return;
    const cid = parseOptionalCharacterIdFromText(characterIdText);
    if (!cid.ok) {
      error = cid.message;
      return;
    }
    try {
      await backend.saveThumbnailSetting(activeProfileId, windowTitle.trim(), config, cid.value);
      windowTitle = "";
      characterIdText = "";
      saveMessage = "Override saved";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function saveOverrideOnBlur(setting: ThumbnailSetting) {
    if (activeProfileId == null) return;
    const width = Number(setting.config.width);
    const height = Number(setting.config.height);
    const dim = validatePositiveThumbnailDimensions(width, height);
    if (!dim.ok) {
      error = dim.message;
      return;
    }
    const cid = parseOptionalCharacterId(setting.characterId);
    if (!cid.ok) {
      error = cid.message;
      return;
    }
    try {
      await backend.saveThumbnailSetting(activeProfileId, setting.windowTitle, setting.config, cid.value);
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  function openCopyFromDialog(setting: ThumbnailSetting) {
    copyTargetWindowTitle = setting.windowTitle;
    copyDialogOpen = true;
  }

  async function applyThumbnailSettingsFromSource(sourceWindowTitle: string) {
    if (activeProfileId == null) return;
    const target = settings.find((s) => s.windowTitle === copyTargetWindowTitle);
    const source = settings.find((s) => s.windowTitle === sourceWindowTitle);
    if (!target || !source) return;
    const cid = parseOptionalCharacterId(target.characterId);
    if (!cid.ok) {
      error = cid.message;
      return;
    }
    target.config = { ...source.config };
    try {
      await backend.saveThumbnailSetting(activeProfileId, target.windowTitle, target.config, cid.value);
      saveMessage = "Settings copied";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
      await refresh();
    }
  }

  onMount(() => {
    void refresh();
    let unlistenImported: UnlistenFn | undefined;
    void listen("yaep-settings-imported", () => {
      void refresh();
    }).then((u) => {
      unlistenImported = u;
    });
    return () => {
      unlistenImported?.();
    };
  });

  $effect(() => {
    if (saveMessage) toast.success(saveMessage);
  });

  $effect(() => {
    if (error) toast.error(error);
  });
</script>

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <ImageIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Thumbnail Settings</CardTitle>
        <CardDescription>Edit default and per-window-title thumbnail config.</CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
    {#if defaultConfig}
      <DefaultThumbnailLayoutForm
        bind:config={defaultConfig}
        bind:selectedTemplateTitle
        bind:selectedAspectRatio
        {settings}
        onSaveDefault={saveDefault}
      />
    {/if}

    <Separator class="my-6" orientation="horizontal" />

    <ThumbnailOverridesSection
      bind:windowTitle
      bind:characterIdText
      {settings}
      onAddOverride={addOrUpdateWindowOverride}
      onSaveBlur={saveOverrideOnBlur}
      onCopyFrom={openCopyFromDialog}
    />

    <CopyThumbnailSettingsDialog
      bind:open={copyDialogOpen}
      targetWindowTitle={copyTargetWindowTitle}
      {settings}
      onApply={applyThumbnailSettingsFromSource}
      onDismiss={() => {
        copyTargetWindowTitle = "";
      }}
    />
  </CardContent>
</Card>
