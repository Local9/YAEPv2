<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import type {
    GridLayoutFormPrefs,
    GridLayoutPayload,
    GridLayoutPreviewItem,
    MonitorInfoDto,
    Profile,
    ThumbnailSetting,
  } from "$models/domain";
  import { toast } from "svelte-sonner";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import Grid3x3Icon from "@lucide/svelte/icons/grid-3x3";
  import GridLayoutControls from "$lib/components/grid-layout/grid-layout-controls.svelte";
  import GridLayoutActionBar from "$lib/components/grid-layout/grid-layout-action-bar.svelte";
  import GridLayoutPreviewTable from "$lib/components/grid-layout/grid-layout-preview-table.svelte";
  import { formatYyyyMmDdDash } from "$lib/datetime/format-yyyy-mm-dd";
  import {
    buildGridLayoutFormPrefs,
    buildGridLayoutPayload,
    formatMonitorLabel,
    monitorWorkOffset as computeMonitorWorkOffset,
    syncHeightFromWidth as computeHeightFromWidth,
    syncWidthFromHeight as computeWidthFromHeight,
  } from "$lib/grid-layout/grid-layout-helpers";
  import {
    ASPECT_RATIO_OPTIONS,
    THUMBNAIL_LAYOUT_HEIGHT_MAX as CELL_H_MAX,
    THUMBNAIL_LAYOUT_HEIGHT_MIN as CELL_H_MIN,
    THUMBNAIL_LAYOUT_WIDTH_MAX as CELL_W_MAX,
    THUMBNAIL_LAYOUT_WIDTH_MIN as CELL_W_MIN,
  } from "$lib/grid-layout/thumbnail-layout-bounds";

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
  let exportBusy = $state(false);

  let aspectRatioItems = $derived<{ value: string; label: string }[]>(
    ASPECT_RATIO_OPTIONS.map((r) => ({ value: r, label: r })),
  );

  /** Keep current width; set height from ratio, then clamp both to slider bounds. */
  function syncHeightFromWidth(width = gridCellWidth) {
    const next = computeHeightFromWidth({
      ratio: selectedAspectRatio,
      width,
      height: gridCellHeight,
      minWidth: CELL_W_MIN,
      maxWidth: CELL_W_MAX,
      minHeight: CELL_H_MIN,
      maxHeight: CELL_H_MAX,
    });
    gridCellWidth = next.width;
    gridCellHeight = next.height;
  }

  /** Keep current height; set width from ratio, then clamp both to slider bounds. */
  function syncWidthFromHeight(height = gridCellHeight) {
    const next = computeWidthFromHeight({
      ratio: selectedAspectRatio,
      width: gridCellWidth,
      height,
      minWidth: CELL_W_MIN,
      maxWidth: CELL_W_MAX,
      minHeight: CELL_H_MIN,
      maxHeight: CELL_H_MAX,
    });
    gridCellWidth = next.width;
    gridCellHeight = next.height;
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
    return computeMonitorWorkOffset(selectedMonitorIndex, monitors);
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

  function collectFormPrefs(): GridLayoutFormPrefs {
    return buildGridLayoutFormPrefs({
      selectedAspectRatio,
      gridCellWidth,
      gridCellHeight,
      gridStartX,
      gridStartY,
      gridColumns,
      onlyAffectActiveThumbnails,
      selectedMonitorIndex,
      selectedAnchorTitle,
    });
  }

  function applyLoadedPrefs(prefs: GridLayoutFormPrefs) {
    selectedAspectRatio = prefs.aspectRatio;
    gridCellWidth = prefs.gridCellWidth;
    gridCellHeight = prefs.gridCellHeight;
    gridStartX = prefs.gridStartX;
    gridStartY = prefs.gridStartY;
    gridColumns = prefs.gridColumns;
    onlyAffectActiveThumbnails = prefs.onlyAffectActiveThumbnails;
    selectedMonitorIndex = prefs.selectedMonitorIndex;
    const titles = new Set(thumbnailSettings.map((t) => t.windowTitle));
    selectedAnchorTitle =
      prefs.selectedAnchorTitle !== "" && titles.has(prefs.selectedAnchorTitle)
        ? prefs.selectedAnchorTitle
        : "";
  }

  function buildPayload(): GridLayoutPayload | null {
    const { payload, error: payloadError } = buildGridLayoutPayload({
      activeProfileId,
      gridCellWidth,
      gridCellHeight,
      gridStartX,
      gridStartY,
      gridColumns,
      onlyAffectActiveThumbnails,
      selectedMonitorIndex,
      selectedAnchorTitle,
    });
    if (payload == null && payloadError) {
      error = payloadError;
    }
    return payload;
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
      try {
        const saved = await backend.gridLayoutGetPrefs(activeProfileId);
        if (saved) {
          applyLoadedPrefs(saved);
        }
      } catch {
        /* keep defaults */
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
    const profileId = payload.profileId;
    try {
      await backend.gridApplyLayout(payload);
      await backend.gridLayoutSavePrefs(profileId, collectFormPrefs());
      status = `Applied layout for ${preview.length} thumbnails; grid settings saved for this profile`;
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function exportGridLayoutPrefs() {
    if (exportBusy || activeProfileId == null) return;
    exportBusy = true;
    error = "";
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const defaultPath = `yaep-grid-layout-${formatYyyyMmDdDash(new Date())}.json`;
      const filePath = await save({
        title: "Export grid layout settings",
        defaultPath,
        filters: [{ name: "YAEP grid layout", extensions: ["json"] }],
      });
      if (!filePath) return;
      await backend.gridLayoutExportPrefsToPath(filePath, activeProfileId, collectFormPrefs());
      status = "Grid layout settings exported.";
    } catch (e) {
      error = String(e);
    } finally {
      exportBusy = false;
    }
  }

  onMount(() => {
    void loadContext();
    let unlistenImported: UnlistenFn | undefined;
    void listen("yaep-settings-imported", () => {
      void loadContext();
    }).then((u) => {
      unlistenImported = u;
    });
    return () => {
      unlistenImported?.();
    };
  });

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
      <Grid3x3Icon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Grid Layout</CardTitle>
        <CardDescription>Build preview and apply bulk layout updates.</CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
    <GridLayoutControls
      bind:selectedAspectRatio
      bind:gridCellWidth
      bind:gridCellHeight
      bind:gridStartX
      bind:gridStartY
      bind:gridColumns
      bind:onlyAffectActiveThumbnails
      bind:selectedMonitorIndex
      bind:selectedAnchorTitle
      {thumbnailSettings}
      {monitors}
      {aspectRatioItems}
      {monitorSelectItems}
      {anchorSelectItems}
      {anchorTriggerLabel}
      {monitorTriggerLabel}
      {startXDisplay}
      {startYDisplay}
      {useAnchorOrigin}
      syncHeightFromWidth={syncHeightFromWidth}
      syncWidthFromHeight={syncWidthFromHeight}
      {formatMonitorLabel}
    />
    <GridLayoutActionBar
      {activeProfileId}
      {exportBusy}
      onGeneratePreview={generatePreview}
      onApplyLayout={applyLayout}
      onExportPrefs={exportGridLayoutPrefs}
    />
    <GridLayoutPreviewTable {preview} />
  </CardContent>
</Card>
