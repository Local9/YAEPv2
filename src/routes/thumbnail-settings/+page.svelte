<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import type { Profile, ThumbnailConfig, ThumbnailSetting } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { ColorPicker } from "$lib/components/ui/color-picker";
  import * as Select from "$lib/components/ui/select";
  import { Input } from "$lib/components/ui/input";
  import { Slider } from "$lib/components/ui/slider";
  import { toast } from "svelte-sonner";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
  import { Separator } from "$lib/components/ui/separator";
  import BookmarkIcon from "@lucide/svelte/icons/bookmark";
  import ImageIcon from "@lucide/svelte/icons/image";
  import ListIcon from "@lucide/svelte/icons/list";
  import SaveIcon from "@lucide/svelte/icons/save";
  import SlidersHorizontalIcon from "@lucide/svelte/icons/sliders-horizontal";
  import {
    syncHeightFromWidth as computeHeightFromWidth,
    syncWidthFromHeight as computeWidthFromHeight,
  } from "../grid-layout/grid-layout-helpers";

  const ASPECT_RATIO_OPTIONS = ["21:9", "21:4", "16:9", "4:3", "1:1"] as const;
  const WIDTH_MIN = 192;
  const WIDTH_MAX = 960;
  const HEIGHT_MIN = 108;
  const HEIGHT_MAX = 540;
  const POSITION_MIN = -10000;
  const POSITION_MAX = 31000;
  const OPACITY_MIN = 0.1;
  const OPACITY_MAX = 1;
  const BORDER_THICKNESS_MIN = 0;
  const BORDER_THICKNESS_MAX = 12;
  const DECLOAK_FLASH_THICKNESS_MIN = 1;
  const DECLOAK_FLASH_THICKNESS_MAX = 12;
  const DECLOAK_FLASH_DURATION_MIN = 250;
  const DECLOAK_FLASH_DURATION_MAX = 10000;

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
  let aspectRatioItems = $derived<{ value: string; label: string }[]>(
    ASPECT_RATIO_OPTIONS.map((ratio) => ({ value: ratio, label: ratio })),
  );
  let templateItems = $derived<{ value: string; label: string }[]>([
    { value: "", label: "Select existing thumbnail..." },
    ...settings.map((setting) => ({
      value: setting.windowTitle,
      label: setting.windowTitle,
    })),
  ]);

  function syncHeightFromWidth(width = defaultConfig?.width ?? WIDTH_MIN) {
    if (!defaultConfig) return;
    const next = computeHeightFromWidth({
      ratio: selectedAspectRatio,
      width,
      height: defaultConfig.height,
      minWidth: WIDTH_MIN,
      maxWidth: WIDTH_MAX,
      minHeight: HEIGHT_MIN,
      maxHeight: HEIGHT_MAX,
    });
    defaultConfig.width = next.width;
    defaultConfig.height = next.height;
  }

  function syncWidthFromHeight(height = defaultConfig?.height ?? HEIGHT_MIN) {
    if (!defaultConfig) return;
    const next = computeWidthFromHeight({
      ratio: selectedAspectRatio,
      width: defaultConfig.width,
      height,
      minWidth: WIDTH_MIN,
      maxWidth: WIDTH_MAX,
      minHeight: HEIGHT_MIN,
      maxHeight: HEIGHT_MAX,
    });
    defaultConfig.width = next.width;
    defaultConfig.height = next.height;
  }

  function cloneDefault(): ThumbnailConfig | null {
    if (!defaultConfig) return null;
    return { ...defaultConfig };
  }

  function applyTemplateSettings(windowTitle: string) {
    selectedTemplateTitle = windowTitle;
    if (!defaultConfig || windowTitle === "") return;
    const template = settings.find((setting) => setting.windowTitle === windowTitle);
    if (!template) return;
    defaultConfig = { ...template.config };
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
    const parsedCharacterId = characterIdText.trim() === "" ? null : Number(characterIdText.trim());
    if (parsedCharacterId != null && (!Number.isInteger(parsedCharacterId) || parsedCharacterId <= 0)) {
      error = "Character ID must be a positive whole number";
      return;
    }
    try {
      await backend.saveThumbnailSetting(activeProfileId, windowTitle.trim(), config, parsedCharacterId);
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
    if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) {
      error = "Width and Height must be positive numbers";
      return;
    }
    const parsedCharacterId =
      setting.characterId == null || String(setting.characterId).trim() === ""
        ? null
        : Number(setting.characterId);
    if (parsedCharacterId != null && (!Number.isInteger(parsedCharacterId) || parsedCharacterId <= 0)) {
      error = "Character ID must be a positive whole number";
      return;
    }
    try {
      await backend.saveThumbnailSetting(activeProfileId, setting.windowTitle, setting.config, parsedCharacterId);
      error = "";
    } catch (e) {
      error = String(e);
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
      <div class="mb-2 flex items-center gap-2 text-sm font-medium text-muted-foreground">
        <SlidersHorizontalIcon class="size-4 shrink-0" aria-hidden="true" />
        <span>Default layout</span>
      </div>
      <div class="grid max-w-4xl grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Field class="sm:col-span-2 lg:col-span-4">
          <FieldLabel class="text-muted-foreground">Load settings from existing thumbnail</FieldLabel>
          <FieldContent>
            <Select.Root
              type="single"
              bind:value={selectedTemplateTitle}
              items={templateItems}
              onValueChange={applyTemplateSettings}
            >
              <Select.Trigger class="w-full">
                <span data-slot="select-value">
                  {selectedTemplateTitle === "" ? "Select existing thumbnail..." : selectedTemplateTitle}
                </span>
              </Select.Trigger>
              <Select.Content class="max-h-72 overflow-y-auto">
                <Select.Item value="" label="Select existing thumbnail...">
                  Select existing thumbnail...
                </Select.Item>
                {#each settings as setting (setting.windowTitle)}
                  <Select.Item value={setting.windowTitle} label={setting.windowTitle}>
                    {setting.windowTitle}
                  </Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
            <p class="mt-1 text-xs text-muted-foreground">
              Copies width/height/position/opacity/border settings into these controls. Click Save
              default to persist.
            </p>
          </FieldContent>
        </Field>
        <Field class="sm:col-span-2 lg:col-span-4">
          <FieldLabel class="text-muted-foreground">Aspect ratio</FieldLabel>
          <FieldContent>
            <Select.Root
              type="single"
              bind:value={selectedAspectRatio}
              items={aspectRatioItems}
              onValueChange={() => syncHeightFromWidth()}
            >
              <Select.Trigger class="w-full">
                <span data-slot="select-value">{selectedAspectRatio}</span>
              </Select.Trigger>
              <Select.Content>
                {#each ASPECT_RATIO_OPTIONS as ratio (ratio)}
                  <Select.Item value={ratio} label={ratio}>{ratio}</Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
          </FieldContent>
        </Field>
        <Field class="sm:col-span-2 lg:col-span-4">
          <div class="flex items-baseline justify-between gap-2">
            <FieldLabel class="text-muted-foreground">Width</FieldLabel>
            <span class="text-sm tabular-nums text-muted-foreground">{defaultConfig.width}px</span>
          </div>
          <FieldContent class="pt-1">
            <Slider
              type="single"
              bind:value={defaultConfig.width}
              min={WIDTH_MIN}
              max={WIDTH_MAX}
              step={1}
              onValueChange={(width) => syncHeightFromWidth(width)}
            />
          </FieldContent>
        </Field>
        <Field class="sm:col-span-2 lg:col-span-4">
          <div class="flex items-baseline justify-between gap-2">
            <FieldLabel class="text-muted-foreground">Height</FieldLabel>
            <span class="text-sm tabular-nums text-muted-foreground">{defaultConfig.height}px</span>
          </div>
          <FieldContent class="pt-1">
            <Slider
              type="single"
              bind:value={defaultConfig.height}
              min={HEIGHT_MIN}
              max={HEIGHT_MAX}
              step={1}
              onValueChange={(height) => syncWidthFromHeight(height)}
            />
            <p class="mt-1 text-xs text-muted-foreground">Stays on the selected aspect ratio</p>
          </FieldContent>
        </Field>
        <Field class="sm:col-span-2 lg:col-span-4">
          <div class="flex items-baseline justify-between gap-2">
            <FieldLabel class="text-muted-foreground">X</FieldLabel>
            <span class="text-sm tabular-nums text-muted-foreground">{defaultConfig.x}</span>
          </div>
          <FieldContent class="pt-1">
            <Slider
              type="single"
              bind:value={defaultConfig.x}
              min={POSITION_MIN}
              max={POSITION_MAX}
              step={1}
            />
          </FieldContent>
        </Field>
        <Field class="sm:col-span-2 lg:col-span-4">
          <div class="flex items-baseline justify-between gap-2">
            <FieldLabel class="text-muted-foreground">Y</FieldLabel>
            <span class="text-sm tabular-nums text-muted-foreground">{defaultConfig.y}</span>
          </div>
          <FieldContent class="pt-1">
            <Slider
              type="single"
              bind:value={defaultConfig.y}
              min={POSITION_MIN}
              max={POSITION_MAX}
              step={1}
            />
          </FieldContent>
        </Field>
        <Field class="sm:col-span-2 lg:col-span-4">
          <div class="flex items-baseline justify-between gap-2">
            <FieldLabel class="text-muted-foreground">Opacity</FieldLabel>
            <span class="text-sm tabular-nums text-muted-foreground">
              {defaultConfig.opacity.toFixed(2)}
            </span>
          </div>
          <FieldContent class="pt-1">
            <Slider
              type="single"
              bind:value={defaultConfig.opacity}
              min={OPACITY_MIN}
              max={OPACITY_MAX}
              step={0.01}
            />
          </FieldContent>
        </Field>
        <Field>
          <FieldLabel class="text-muted-foreground">Border color</FieldLabel>
          <FieldContent>
            <div class="flex items-center gap-2">
              <ColorPicker bind:value={defaultConfig.focusBorderColor} />
              <Input bind:value={defaultConfig.focusBorderColor} class="font-mono" />
            </div>
          </FieldContent>
        </Field>
        <Field class="sm:col-span-2 lg:col-span-3">
          <div class="flex items-baseline justify-between gap-2">
            <FieldLabel class="text-muted-foreground">Border thickness</FieldLabel>
            <span class="text-sm tabular-nums text-muted-foreground">
              {defaultConfig.focusBorderThickness}
            </span>
          </div>
          <FieldContent class="pt-1">
            <Slider
              type="single"
              bind:value={defaultConfig.focusBorderThickness}
              min={BORDER_THICKNESS_MIN}
              max={BORDER_THICKNESS_MAX}
              step={1}
            />
          </FieldContent>
        </Field>
        <Field>
          <FieldLabel class="text-muted-foreground">Decloak flash color</FieldLabel>
          <FieldContent>
            <div class="flex items-center gap-2">
              <ColorPicker bind:value={defaultConfig.decloakFlashColor} />
              <Input bind:value={defaultConfig.decloakFlashColor} class="font-mono" />
            </div>
          </FieldContent>
        </Field>
        <Field class="sm:col-span-2 lg:col-span-3">
          <div class="flex items-baseline justify-between gap-2">
            <FieldLabel class="text-muted-foreground">Decloak flash border thickness</FieldLabel>
            <span class="text-sm tabular-nums text-muted-foreground">
              {defaultConfig.decloakFlashThickness}
            </span>
          </div>
          <FieldContent class="pt-1">
            <Slider
              type="single"
              bind:value={defaultConfig.decloakFlashThickness}
              min={DECLOAK_FLASH_THICKNESS_MIN}
              max={DECLOAK_FLASH_THICKNESS_MAX}
              step={1}
            />
          </FieldContent>
        </Field>
        <Field class="sm:col-span-2 lg:col-span-4">
          <div class="flex items-baseline justify-between gap-2">
            <FieldLabel class="text-muted-foreground">Decloak flash pulse duration (ms)</FieldLabel>
            <span class="text-sm tabular-nums text-muted-foreground">
              {defaultConfig.decloakFlashDurationMs}
            </span>
          </div>
          <FieldContent class="pt-1">
            <Slider
              type="single"
              bind:value={defaultConfig.decloakFlashDurationMs}
              min={DECLOAK_FLASH_DURATION_MIN}
              max={DECLOAK_FLASH_DURATION_MAX}
              step={250}
            />
          </FieldContent>
        </Field>
        <Field orientation="horizontal" class="cursor-pointer self-end sm:col-span-2 lg:col-span-1">
          <FieldContent>
            <Checkbox bind:checked={defaultConfig.showTitleOverlay} />
          </FieldContent>
          <FieldLabel class="text-muted-foreground">Show title</FieldLabel>
        </Field>
      </div>
      <div class="mt-4 flex flex-wrap items-center gap-2">
        <Button type="button" onclick={saveDefault} class="gap-2">
          <SaveIcon class="size-4 shrink-0" aria-hidden="true" />
          Save default
        </Button>
      </div>
    {/if}

    <Separator class="my-6" orientation="horizontal" />

    <div class="mb-3 flex items-center gap-2 text-sm font-medium text-muted-foreground">
      <BookmarkIcon class="size-4 shrink-0" aria-hidden="true" />
      <h3 class="text-base font-semibold text-foreground">Per-title override</h3>
    </div>
    <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
      <Input
        class="sm:max-w-md sm:flex-1"
        bind:value={windowTitle}
        placeholder="EVE - CharacterName"
      />
      <Input class="sm:w-56" bind:value={characterIdText} placeholder="Character ID (optional)" />
      <Button type="button" variant="outline" onclick={addOrUpdateWindowOverride} class="shrink-0 gap-2">
        <SaveIcon class="size-4 shrink-0" aria-hidden="true" />
        Save override from default
      </Button>
    </div>
    <p class="mt-2 text-xs text-muted-foreground">
      Character IDs can be found in zKillboard URLs, for example
      <code class="rounded bg-muted px-1">https://zkillboard.com/character/1698894137/</code>.
    </p>
    <div class="mt-4 overflow-x-auto rounded-md border border-border/60">
      <table class="w-full text-sm">
        <thead class="bg-muted/40 text-left">
          <tr>
            <th class="px-3 py-2 font-medium">Thumbnail Title</th>
            <th class="px-3 py-2 font-medium">Width</th>
            <th class="px-3 py-2 font-medium">Height</th>
            <th class="px-3 py-2 font-medium">Character ID</th>
          </tr>
        </thead>
        <tbody>
          {#if settings.length === 0}
            <tr>
              <td class="px-3 py-2 text-muted-foreground" colspan="4">No overrides saved yet.</td>
            </tr>
          {:else}
            {#each settings as setting (setting.windowTitle)}
              <tr class="border-t">
                <td class="px-3 py-2">
                  <div class="flex items-center gap-2">
                    <ListIcon class="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
                    <code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{setting.windowTitle}</code>
                  </div>
                </td>
                <td class="px-3 py-2">
                  <Input
                    type="number"
                    min="1"
                    bind:value={setting.config.width}
                    onblur={() => void saveOverrideOnBlur(setting)}
                  />
                </td>
                <td class="px-3 py-2">
                  <Input
                    type="number"
                    min="1"
                    bind:value={setting.config.height}
                    onblur={() => void saveOverrideOnBlur(setting)}
                  />
                </td>
                <td class="px-3 py-2">
                  <Input
                    type="number"
                    min="1"
                    bind:value={setting.characterId}
                    onblur={() => void saveOverrideOnBlur(setting)}
                    placeholder="Optional"
                  />
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </CardContent>
</Card>
