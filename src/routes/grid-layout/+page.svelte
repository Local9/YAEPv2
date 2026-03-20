<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type {
    GridLayoutPayload,
    GridLayoutPreviewItem,
    MonitorInfoDto,
    Profile,
  } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";
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
  import MonitorIcon from "@lucide/svelte/icons/monitor";
  import PlayIcon from "@lucide/svelte/icons/play";

  let profiles = $state<Profile[]>([]);
  let activeProfileId = $state<number | null>(null);
  let gridCellWidth = $state(400);
  let gridCellHeight = $state(300);
  let gridStartX = $state(100);
  let gridStartY = $state(100);
  let gridColumns = $state(3);
  let onlyAffectActiveThumbnails = $state(false);
  let monitors = $state<MonitorInfoDto[]>([]);
  /** Empty string = no monitor offset / clamp */
  let selectedMonitorIndex = $state("");
  let preview = $state<GridLayoutPreviewItem[]>([]);
  let status = $state("");
  let error = $state("");

  const selectClass =
    "mt-1.5 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring";

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

<section class="rounded-lg border border-border bg-card p-4 text-card-foreground shadow-sm">
  <div class="mb-4 flex items-start gap-3">
    <Grid3x3Icon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
    <div>
      <h2 class="text-lg font-semibold tracking-tight">Grid Layout</h2>
      <p class="mt-1 text-sm text-muted-foreground">Build preview and apply bulk layout updates.</p>
    </div>
  </div>

  <div class="grid max-w-4xl grid-cols-1 gap-3 sm:grid-cols-3">
    <label class="grid gap-1.5 text-sm font-medium">
      <span class="text-muted-foreground">Cell Width</span>
      <Input type="number" bind:value={gridCellWidth} />
    </label>
    <label class="grid gap-1.5 text-sm font-medium">
      <span class="text-muted-foreground">Cell Height</span>
      <Input type="number" bind:value={gridCellHeight} />
    </label>
    <label class="grid gap-1.5 text-sm font-medium">
      <span class="text-muted-foreground">Columns</span>
      <Input type="number" bind:value={gridColumns} />
    </label>
    <label class="grid gap-1.5 text-sm font-medium">
      <span class="text-muted-foreground">Start X</span>
      <Input type="number" bind:value={gridStartX} />
    </label>
    <label class="grid gap-1.5 text-sm font-medium">
      <span class="text-muted-foreground">Start Y</span>
      <Input type="number" bind:value={gridStartY} />
    </label>
    <label class="grid gap-1.5 text-sm font-medium sm:col-span-3">
      <span class="flex items-center gap-1.5 text-muted-foreground">
        <MonitorIcon class="size-3.5 shrink-0" aria-hidden="true" />
        Monitor
      </span>
      <select class={selectClass} bind:value={selectedMonitorIndex}>
        <option value="">All / default origin</option>
        {#each monitors as m (m.index)}
          <option value={String(m.index)}>
            {m.index}: {m.name || "Display"}{m.isPrimary ? " (primary)" : ""}
          </option>
        {/each}
      </select>
    </label>
    <label
      class="flex cursor-pointer items-center gap-2 self-end text-sm font-medium sm:col-span-3"
    >
      <input
        class="size-4 rounded border border-input text-primary focus-visible:ring-2 focus-visible:ring-ring"
        type="checkbox"
        bind:checked={onlyAffectActiveThumbnails}
      />
      <span class="text-muted-foreground">Only active thumbnails</span>
    </label>
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
</section>
