<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { backend } from "$services/backend";
  import type {
    GridLayoutPayload,
    GridLayoutPreviewItem,
    MonitorInfoDto,
    Profile,
    ThumbnailSetting,
  } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Select from "$lib/components/ui/select";
  import { Slider } from "$lib/components/ui/slider";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import Grid3x3Icon from "@lucide/svelte/icons/grid-3x3";
  import LayoutGridIcon from "@lucide/svelte/icons/layout-grid";
  import CrosshairIcon from "@lucide/svelte/icons/crosshair";
  import MonitorIcon from "@lucide/svelte/icons/monitor";
  import PlayIcon from "@lucide/svelte/icons/play";

  const ASPECT_RATIO_OPTIONS = ["21:9", "21:4", "16:9", "4:3", "1:1"] as const;

  let profiles = $state<Profile[]>([]);
  let activeProfileId = $state<number | null>(null);
  let gridCellWidth = $state(300);
  let gridCellHeight = $state(169);
  let gridStartX = $state(100);
  let gridStartY = $state(100);
  let gridColumns = $state(3);
  let selectedAspectRatio = $state<string>("16:9");
  let onlyAffectActiveThumbnails = $state(true);
  let monitors = $state<MonitorInfoDto[]>([]);
  /** Empty string = no monitor offset / clamp */
  let selectedMonitorIndex = $state("");
  /** Empty = manual Start X/Y; otherwise grid origin from this thumbnail's saved position */
  let selectedAnchorTitle = $state("");
  let thumbnailSettings = $state<ThumbnailSetting[]>([]);
  let preview = $state<GridLayoutPreviewItem[]>([]);
  let status = $state("");
  let error = $state("");

  let aspectRatioItems = $derived<{ value: string; label: string }[]>(
    ASPECT_RATIO_OPTIONS.map((r) => ({ value: r, label: r })),
  );

  const CELL_W_MIN = 192;
  const CELL_W_MAX = 960;
  const CELL_H_MIN = 108;
  const CELL_H_MAX = 540;

  function parseAspectRatio(ratio: string): { rw: number; rh: number } {
    const trimmed = ratio.trim();
    const sep = trimmed.split(":");
    if (sep.length !== 2) return { rw: 16, rh: 9 };
    const rw = Number.parseFloat(sep[0].trim());
    const rh = Number.parseFloat(sep[1].trim());
    if (!Number.isFinite(rw) || !Number.isFinite(rh) || rw <= 0 || rh <= 0) {
      return { rw: 16, rh: 9 };
    }
    return { rw, rh };
  }

  function clamp(n: number, lo: number, hi: number): number {
    return Math.max(lo, Math.min(hi, n));
  }

  /** Keep current width; set height from ratio, then clamp both to slider bounds. */
  function syncHeightFromWidth(width = gridCellWidth) {
    const { rw, rh } = parseAspectRatio(selectedAspectRatio);
    gridCellWidth = clamp(width, CELL_W_MIN, CELL_W_MAX);
    let nextH = Math.round((gridCellWidth * rh) / rw);
    nextH = clamp(nextH, CELL_H_MIN, CELL_H_MAX);
    gridCellHeight = nextH;
    gridCellWidth = clamp(Math.round((gridCellHeight * rw) / rh), CELL_W_MIN, CELL_W_MAX);
    gridCellHeight = clamp(Math.round((gridCellWidth * rh) / rw), CELL_H_MIN, CELL_H_MAX);
  }

  /** Keep current height; set width from ratio, then clamp both to slider bounds. */
  function syncWidthFromHeight(height = gridCellHeight) {
    const { rw, rh } = parseAspectRatio(selectedAspectRatio);
    gridCellHeight = clamp(height, CELL_H_MIN, CELL_H_MAX);
    let nextW = Math.round((gridCellHeight * rw) / rh);
    nextW = clamp(nextW, CELL_W_MIN, CELL_W_MAX);
    gridCellWidth = nextW;
    gridCellHeight = clamp(Math.round((gridCellWidth * rh) / rw), CELL_H_MIN, CELL_H_MAX);
  }

  $effect(() => {
    void selectedAspectRatio;
    untrack(() => syncHeightFromWidth());
  });

  function formatMonitorLabel(m: MonitorInfoDto): string {
    const widthPx = m.right - m.left;
    const heightPx = m.bottom - m.top;
    const primary = m.isPrimary ? " (Primary)" : "";
    return `#${m.index} - ${m.name || "Display"}${primary} - ${widthPx}x${heightPx} @ (${m.left}, ${m.top})`;
  }

  let monitorTriggerLabel = $derived.by(() => {
    if (selectedMonitorIndex === "") return "All / default origin";
    const m = monitors.find((x) => String(x.index) === selectedMonitorIndex);
    if (!m) return selectedMonitorIndex;
    return formatMonitorLabel(m);
  });

  let monitorSelectItems = $derived<{ value: string; label: string }[]>([
    { value: "", label: "All / default origin" },
    ...monitors.map((m) => ({
      value: String(m.index),
      label: formatMonitorLabel(m),
    })),
  ]);

  function monitorWorkOffset(): { ox: number; oy: number } {
    if (selectedMonitorIndex === "") return { ox: 0, oy: 0 };
    const m = monitors.find((x) => String(x.index) === selectedMonitorIndex);
    if (!m) return { ox: 0, oy: 0 };
    return { ox: m.workLeft, oy: m.workTop };
  }

  let anchorDerivedStart = $derived.by(() => {
    if (selectedAnchorTitle === "") return null;
    const s = thumbnailSettings.find((t) => t.windowTitle === selectedAnchorTitle);
    if (!s) return null;
    const { ox, oy } = monitorWorkOffset();
    return { x: s.config.x - ox, y: s.config.y - oy };
  });

  let anchorSelectItems = $derived<{ value: string; label: string }[]>([
    { value: "", label: "Manual start position" },
    ...thumbnailSettings.map((t) => ({
      value: t.windowTitle,
      label: t.windowTitle,
    })),
  ]);

  let anchorTriggerLabel = $derived.by(() => {
    if (selectedAnchorTitle === "") return "Manual start position";
    const max = 64;
    const t = selectedAnchorTitle;
    return t.length > max ? `${t.slice(0, max)}…` : t;
  });

  let startXDisplay = $derived(
    selectedAnchorTitle !== "" ? (anchorDerivedStart?.x ?? gridStartX) : gridStartX,
  );
  let startYDisplay = $derived(
    selectedAnchorTitle !== "" ? (anchorDerivedStart?.y ?? gridStartY) : gridStartY,
  );

  let useAnchorOrigin = $derived(selectedAnchorTitle !== "");

  function buildPayload(): GridLayoutPayload | null {
    if (activeProfileId == null) {
      error = "No active profile available";
      return null;
    }
    return {
      profileId: activeProfileId,
      gridCellWidth,
      gridCellHeight,
      gridCellRatio: null,
      gridStartX,
      gridStartY,
      gridColumns,
      selectedGroupId: null,
      onlyAffectActiveThumbnails,
      selectedMonitorIndex:
        selectedMonitorIndex === "" ? null : Number.parseInt(selectedMonitorIndex, 10),
      gridAnchorWindowTitle: selectedAnchorTitle === "" ? null : selectedAnchorTitle,
    };
  }

  async function loadContext() {
    profiles = await backend.getProfiles();
    activeProfileId = profiles.find((p) => p.isActive)?.id ?? null;
    try {
      monitors = await backend.listMonitors();
    } catch {
      monitors = [];
    }
    if (activeProfileId != null) {
      try {
        thumbnailSettings = await backend.getThumbnailSettings(activeProfileId);
      } catch {
        thumbnailSettings = [];
      }
    } else {
      thumbnailSettings = [];
    }
  }

  async function generatePreview() {
    const payload = buildPayload();
    if (!payload) return;
    try {
      preview = await backend.gridPreviewLayout(payload);
      status = `Preview generated for ${preview.length} thumbnails`;
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function applyLayout() {
    const payload = buildPayload();
    if (!payload) return;
    try {
      await backend.gridApplyLayout(payload);
      status = `Applied layout for ${preview.length} thumbnails`;
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  onMount(loadContext);
</script>

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <Grid3x3Icon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Grid Layout</CardTitle>
        <CardDescription>Build preview and apply bulk layout updates.</CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
  <div class="grid max-w-4xl grid-cols-1 gap-4 sm:grid-cols-3">
    <Field class="sm:col-span-3">
      <FieldLabel class="text-muted-foreground">Aspect ratio</FieldLabel>
      <FieldContent>
        <Select.Root type="single" bind:value={selectedAspectRatio} items={aspectRatioItems}>
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
          When set, the grid starts at this thumbnail&apos;s saved position and it occupies the first
          cell. Choose &quot;Manual start position&quot; to use the Start X/Y sliders.
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

  <div class="mt-4 flex flex-wrap gap-2">
    <Button onclick={generatePreview} class="gap-2">
      <LayoutGridIcon class="size-4 shrink-0" aria-hidden="true" />
      Generate Preview
    </Button>
    <Button onclick={applyLayout} variant="secondary" class="gap-2">
      <PlayIcon class="size-4 shrink-0" aria-hidden="true" />
      Apply Layout
    </Button>
  </div>

  {#if status}
    <Alert class="mt-4 border-primary/30 bg-primary/5">
      <LayoutGridIcon class="size-4 text-primary" aria-hidden="true" />
      <AlertTitle>Grid Layout</AlertTitle>
      <AlertDescription>{status}</AlertDescription>
    </Alert>
  {/if}
  {#if error}
    <Alert variant="destructive" class="mt-4">
      <AlertCircleIcon class="size-4" aria-hidden="true" />
      <AlertTitle>Error</AlertTitle>
      <AlertDescription>{error}</AlertDescription>
    </Alert>
  {/if}

  <div class="mt-6 overflow-x-auto">
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Window Title</TableHead>
          <TableHead>X</TableHead>
          <TableHead>Y</TableHead>
          <TableHead>Width</TableHead>
          <TableHead>Height</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {#each preview as item (item.windowTitle)}
          <TableRow>
            <TableCell>{item.windowTitle}</TableCell>
            <TableCell>{item.x}</TableCell>
            <TableCell>{item.y}</TableCell>
            <TableCell>{item.width}</TableCell>
            <TableCell>{item.height}</TableCell>
          </TableRow>
        {/each}
      </TableBody>
    </Table>
  </div>
  </CardContent>
</Card>
