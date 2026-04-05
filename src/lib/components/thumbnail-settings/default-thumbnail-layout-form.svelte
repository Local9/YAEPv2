<script lang="ts">
  import type { ThumbnailConfig, ThumbnailSetting } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { ColorPicker } from "$lib/components/ui/color-picker";
  import * as Select from "$lib/components/ui/select";
  import { Input } from "$lib/components/ui/input";
  import { Slider } from "$lib/components/ui/slider";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
  import SaveIcon from "@lucide/svelte/icons/save";
  import SlidersHorizontalIcon from "@lucide/svelte/icons/sliders-horizontal";
  import {
    syncHeightFromWidth as computeHeightFromWidth,
    syncWidthFromHeight as computeWidthFromHeight,
  } from "$lib/grid-layout/grid-layout-helpers";
  import {
    ASPECT_RATIO_OPTIONS,
    WIDTH_MIN,
    WIDTH_MAX,
    HEIGHT_MIN,
    HEIGHT_MAX,
    POSITION_MIN,
    POSITION_MAX,
    OPACITY_MIN,
    OPACITY_MAX,
    BORDER_THICKNESS_MIN,
    BORDER_THICKNESS_MAX,
    DECLOAK_FLASH_THICKNESS_MIN,
    DECLOAK_FLASH_THICKNESS_MAX,
    DECLOAK_FLASH_DURATION_MIN,
    DECLOAK_FLASH_DURATION_MAX,
  } from "./thumbnail-settings-constants";

  interface Props {
    config: ThumbnailConfig;
    settings: ThumbnailSetting[];
    selectedTemplateTitle?: string;
    selectedAspectRatio?: string;
    onSaveDefault: () => void;
  }

  let {
    config = $bindable(),
    settings,
    selectedTemplateTitle = $bindable(""),
    selectedAspectRatio = $bindable("16:9"),
    onSaveDefault,
  }: Props = $props();

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

  function syncHeightFromWidth(width = config.width) {
    const next = computeHeightFromWidth({
      ratio: selectedAspectRatio,
      width,
      height: config.height,
      minWidth: WIDTH_MIN,
      maxWidth: WIDTH_MAX,
      minHeight: HEIGHT_MIN,
      maxHeight: HEIGHT_MAX,
    });
    config.width = next.width;
    config.height = next.height;
  }

  function syncWidthFromHeight(height = config.height) {
    const next = computeWidthFromHeight({
      ratio: selectedAspectRatio,
      width: config.width,
      height,
      minWidth: WIDTH_MIN,
      maxWidth: WIDTH_MAX,
      minHeight: HEIGHT_MIN,
      maxHeight: HEIGHT_MAX,
    });
    config.width = next.width;
    config.height = next.height;
  }

  function applyTemplateSettings(windowTitle: string) {
    selectedTemplateTitle = windowTitle;
    if (windowTitle === "") return;
    const template = settings.find((setting) => setting.windowTitle === windowTitle);
    if (!template) return;
    Object.assign(config, template.config);
  }
</script>

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
        Copies width/height/position/opacity/border settings into these controls. Click Save default to
        persist.
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
      <span class="text-sm tabular-nums text-muted-foreground">{config.width}px</span>
    </div>
    <FieldContent class="pt-1">
      <Slider
        type="single"
        bind:value={config.width}
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
      <span class="text-sm tabular-nums text-muted-foreground">{config.height}px</span>
    </div>
    <FieldContent class="pt-1">
      <Slider
        type="single"
        bind:value={config.height}
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
      <span class="text-sm tabular-nums text-muted-foreground">{config.x}</span>
    </div>
    <FieldContent class="pt-1">
      <Slider type="single" bind:value={config.x} min={POSITION_MIN} max={POSITION_MAX} step={1} />
    </FieldContent>
  </Field>
  <Field class="sm:col-span-2 lg:col-span-4">
    <div class="flex items-baseline justify-between gap-2">
      <FieldLabel class="text-muted-foreground">Y</FieldLabel>
      <span class="text-sm tabular-nums text-muted-foreground">{config.y}</span>
    </div>
    <FieldContent class="pt-1">
      <Slider type="single" bind:value={config.y} min={POSITION_MIN} max={POSITION_MAX} step={1} />
    </FieldContent>
  </Field>
  <Field class="sm:col-span-2 lg:col-span-4">
    <div class="flex items-baseline justify-between gap-2">
      <FieldLabel class="text-muted-foreground">Opacity</FieldLabel>
      <span class="text-sm tabular-nums text-muted-foreground">{config.opacity.toFixed(2)}</span>
    </div>
    <FieldContent class="pt-1">
      <Slider
        type="single"
        bind:value={config.opacity}
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
        <ColorPicker bind:value={config.focusBorderColor} />
        <Input bind:value={config.focusBorderColor} class="font-mono" />
      </div>
    </FieldContent>
  </Field>
  <Field class="sm:col-span-2 lg:col-span-3">
    <div class="flex items-baseline justify-between gap-2">
      <FieldLabel class="text-muted-foreground">Border thickness</FieldLabel>
      <span class="text-sm tabular-nums text-muted-foreground">{config.focusBorderThickness}</span>
    </div>
    <FieldContent class="pt-1">
      <Slider
        type="single"
        bind:value={config.focusBorderThickness}
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
        <ColorPicker bind:value={config.decloakFlashColor} />
        <Input bind:value={config.decloakFlashColor} class="font-mono" />
      </div>
    </FieldContent>
  </Field>
  <Field class="sm:col-span-2 lg:col-span-3">
    <div class="flex items-baseline justify-between gap-2">
      <FieldLabel class="text-muted-foreground">Decloak flash border thickness</FieldLabel>
      <span class="text-sm tabular-nums text-muted-foreground">{config.decloakFlashThickness}</span>
    </div>
    <FieldContent class="pt-1">
      <Slider
        type="single"
        bind:value={config.decloakFlashThickness}
        min={DECLOAK_FLASH_THICKNESS_MIN}
        max={DECLOAK_FLASH_THICKNESS_MAX}
        step={1}
      />
    </FieldContent>
  </Field>
  <Field class="sm:col-span-2 lg:col-span-4">
    <div class="flex items-baseline justify-between gap-2">
      <FieldLabel class="text-muted-foreground">Decloak flash pulse duration (ms)</FieldLabel>
      <span class="text-sm tabular-nums text-muted-foreground">{config.decloakFlashDurationMs}</span>
    </div>
    <FieldContent class="pt-1">
      <Slider
        type="single"
        bind:value={config.decloakFlashDurationMs}
        min={DECLOAK_FLASH_DURATION_MIN}
        max={DECLOAK_FLASH_DURATION_MAX}
        step={250}
      />
    </FieldContent>
  </Field>
  <Field orientation="horizontal" class="cursor-pointer self-end sm:col-span-2 lg:col-span-1">
    <FieldContent>
      <Checkbox bind:checked={config.showTitleOverlay} />
    </FieldContent>
    <FieldLabel class="text-muted-foreground">Show title</FieldLabel>
  </Field>
</div>
<div class="mt-4 flex flex-wrap items-center gap-2">
  <Button type="button" onclick={onSaveDefault} class="gap-2">
    <SaveIcon class="size-4 shrink-0" aria-hidden="true" />
    Save default
  </Button>
</div>
