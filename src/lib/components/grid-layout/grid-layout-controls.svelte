<script lang="ts">
  import type { MonitorInfoDto, ThumbnailSetting } from "$models/domain";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Select from "$lib/components/ui/select";
  import { Slider } from "$lib/components/ui/slider";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
  import CrosshairIcon from "@lucide/svelte/icons/crosshair";
  import MonitorIcon from "@lucide/svelte/icons/monitor";
  import {
    ASPECT_RATIO_OPTIONS,
    THUMBNAIL_LAYOUT_HEIGHT_MAX as CELL_H_MAX,
    THUMBNAIL_LAYOUT_HEIGHT_MIN as CELL_H_MIN,
    THUMBNAIL_LAYOUT_WIDTH_MAX as CELL_W_MAX,
    THUMBNAIL_LAYOUT_WIDTH_MIN as CELL_W_MIN,
  } from "$lib/grid-layout/thumbnail-layout-bounds";

  interface Props {
    selectedAspectRatio: string;
    gridCellWidth: number;
    gridCellHeight: number;
    gridStartX: number;
    gridStartY: number;
    gridColumns: number;
    onlyAffectActiveThumbnails: boolean;
    selectedMonitorIndex: string;
    selectedAnchorTitle: string;
    thumbnailSettings: ThumbnailSetting[];
    monitors: MonitorInfoDto[];
    aspectRatioItems: { value: string; label: string }[];
    monitorSelectItems: { value: string; label: string }[];
    anchorSelectItems: { value: string; label: string }[];
    anchorTriggerLabel: string;
    monitorTriggerLabel: string;
    startXDisplay: number;
    startYDisplay: number;
    useAnchorOrigin: boolean;
    syncHeightFromWidth: (width?: number) => void;
    syncWidthFromHeight: (height?: number) => void;
    formatMonitorLabel: (m: MonitorInfoDto) => string;
  }

  let {
    selectedAspectRatio = $bindable(),
    gridCellWidth = $bindable(),
    gridCellHeight = $bindable(),
    gridStartX = $bindable(),
    gridStartY = $bindable(),
    gridColumns = $bindable(),
    onlyAffectActiveThumbnails = $bindable(),
    selectedMonitorIndex = $bindable(),
    selectedAnchorTitle = $bindable(),
    thumbnailSettings,
    monitors,
    aspectRatioItems,
    monitorSelectItems,
    anchorSelectItems,
    anchorTriggerLabel,
    monitorTriggerLabel,
    startXDisplay,
    startYDisplay,
    useAnchorOrigin,
    syncHeightFromWidth,
    syncWidthFromHeight,
    formatMonitorLabel,
  }: Props = $props();
</script>

<div class="grid max-w-4xl grid-cols-1 gap-4 sm:grid-cols-3">
  <Field class="sm:col-span-3">
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
          {#each ASPECT_RATIO_OPTIONS as r (r)}
            <Select.Item value={r} label={r}>{r}</Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>
    </FieldContent>
  </Field>

  <Field class="sm:col-span-3">
    <div class="flex items-baseline justify-between gap-2">
      <FieldLabel class="text-muted-foreground">Cell width</FieldLabel>
      <span class="text-muted-foreground text-sm tabular-nums">{gridCellWidth}px</span>
    </div>
    <FieldContent class="pt-1">
      <Slider
        type="single"
        bind:value={gridCellWidth}
        min={CELL_W_MIN}
        max={CELL_W_MAX}
        step={1}
        onValueChange={(w) => syncHeightFromWidth(w)}
      />
    </FieldContent>
  </Field>

  <Field class="sm:col-span-3">
    <div class="flex items-baseline justify-between gap-2">
      <FieldLabel class="text-muted-foreground">Cell height</FieldLabel>
      <span class="text-muted-foreground text-sm tabular-nums">{gridCellHeight}px</span>
    </div>
    <FieldContent class="pt-1">
      <Slider
        type="single"
        bind:value={gridCellHeight}
        min={CELL_H_MIN}
        max={CELL_H_MAX}
        step={1}
        onValueChange={(h) => syncWidthFromHeight(h)}
      />
      <p class="text-muted-foreground mt-1 text-xs">Stays on the selected aspect ratio</p>
    </FieldContent>
  </Field>

  <Field class="sm:col-span-3">
    <FieldLabel class="flex items-center gap-1.5 text-muted-foreground">
      <CrosshairIcon class="size-3.5 shrink-0" aria-hidden="true" />
      Initial thumbnail
    </FieldLabel>
    <FieldContent>
      <Select.Root type="single" bind:value={selectedAnchorTitle} items={anchorSelectItems}>
        <Select.Trigger class="w-full">
          <span data-slot="select-value">{anchorTriggerLabel}</span>
        </Select.Trigger>
        <Select.Content class="max-h-72 overflow-y-auto">
          <Select.Item value="" label="Manual start position">Manual start position</Select.Item>
          {#each thumbnailSettings as t (t.windowTitle)}
            <Select.Item value={t.windowTitle} label={t.windowTitle}>
              {t.windowTitle}
            </Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>
      <p class="text-muted-foreground mt-1 text-xs">
        When set, the grid starts at this thumbnail&apos;s saved position and it occupies the first cell.
        Choose &quot;Manual start position&quot; to use the Start X/Y sliders.
      </p>
    </FieldContent>
  </Field>

  <Field class="sm:col-span-3">
    <div class="flex items-baseline justify-between gap-2">
      <FieldLabel class="text-muted-foreground">Start X</FieldLabel>
      <span class="text-muted-foreground text-sm tabular-nums">{startXDisplay}</span>
    </div>
    <FieldContent class="pt-1">
      <Slider
        type="single"
        bind:value={gridStartX}
        min={-10000}
        max={31000}
        step={1}
        disabled={useAnchorOrigin}
      />
    </FieldContent>
  </Field>

  <Field class="sm:col-span-3">
    <div class="flex items-baseline justify-between gap-2">
      <FieldLabel class="text-muted-foreground">Start Y</FieldLabel>
      <span class="text-muted-foreground text-sm tabular-nums">{startYDisplay}</span>
    </div>
    <FieldContent class="pt-1">
      <Slider
        type="single"
        bind:value={gridStartY}
        min={-10000}
        max={31000}
        step={1}
        disabled={useAnchorOrigin}
      />
    </FieldContent>
  </Field>

  <Field class="sm:col-span-3">
    <div class="flex items-baseline justify-between gap-2">
      <FieldLabel class="text-muted-foreground">Columns</FieldLabel>
      <span class="text-muted-foreground text-sm tabular-nums">{gridColumns}</span>
    </div>
    <FieldContent class="pt-1">
      <Slider type="single" bind:value={gridColumns} min={1} max={10} step={1} />
    </FieldContent>
  </Field>

  <Field class="sm:col-span-3">
    <FieldLabel class="flex items-center gap-1.5 text-muted-foreground">
      <MonitorIcon class="size-3.5 shrink-0" aria-hidden="true" />
      Monitor
    </FieldLabel>
    <FieldContent>
      <Select.Root type="single" bind:value={selectedMonitorIndex} items={monitorSelectItems}>
        <Select.Trigger class="w-full">
          <span data-slot="select-value">{monitorTriggerLabel}</span>
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="" label="All / default origin">All / default origin</Select.Item>
          {#each monitors as m (m.index)}
            <Select.Item value={String(m.index)} label={formatMonitorLabel(m)}>
              {formatMonitorLabel(m)}
            </Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>
    </FieldContent>
  </Field>
  <Field class="self-end sm:col-span-3">
    <FieldContent class="flex w-full flex-row items-center gap-2 flex-initial!">
      <Checkbox
        id="grid-layout-only-active-thumbnails"
        bind:checked={onlyAffectActiveThumbnails}
        class="cursor-pointer"
      />
      <FieldLabel
        for="grid-layout-only-active-thumbnails"
        class="text-muted-foreground mb-0 cursor-pointer leading-snug font-normal"
      >
        Only active thumbnails
      </FieldLabel>
    </FieldContent>
  </Field>
</div>
