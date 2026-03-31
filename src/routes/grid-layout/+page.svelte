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
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select";
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

  let monitorTriggerLabel = $derived.by(() => {
    if (selectedMonitorIndex === "") return "All / default origin";
    const m = monitors.find((x) => String(x.index) === selectedMonitorIndex);
    if (!m) return selectedMonitorIndex;
    return `${m.index}: ${m.name || "Display"}${m.isPrimary ? " (primary)" : ""}`;
  });

  let monitorSelectItems = $derived<
    { value: string; label: string }[]
  >([
    { value: "", label: "All / default origin" },
    ...monitors.map((m) => ({
      value: String(m.index),
      label: `${m.index}: ${m.name || "Display"}${m.isPrimary ? " (primary)" : ""}`,
    })),
  ]);

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
  <div class="grid max-w-4xl grid-cols-1 gap-3 sm:grid-cols-3">
    <Field>
      <FieldLabel class="text-muted-foreground">Cell Width</FieldLabel>
      <FieldContent>
        <Input type="number" bind:value={gridCellWidth} />
      </FieldContent>
    </Field>
    <Field>
      <FieldLabel class="text-muted-foreground">Cell Height</FieldLabel>
      <FieldContent>
        <Input type="number" bind:value={gridCellHeight} />
      </FieldContent>
    </Field>
    <Field>
      <FieldLabel class="text-muted-foreground">Columns</FieldLabel>
      <FieldContent>
        <Input type="number" bind:value={gridColumns} />
      </FieldContent>
    </Field>
    <Field>
      <FieldLabel class="text-muted-foreground">Start X</FieldLabel>
      <FieldContent>
        <Input type="number" bind:value={gridStartX} />
      </FieldContent>
    </Field>
    <Field>
      <FieldLabel class="text-muted-foreground">Start Y</FieldLabel>
      <FieldContent>
        <Input type="number" bind:value={gridStartY} />
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
              <Select.Item
                value={String(m.index)}
                label={`${m.index}: ${m.name || "Display"}${m.isPrimary ? " (primary)" : ""}`}
              >
                {m.index}: {m.name || "Display"}{m.isPrimary ? " (primary)" : ""}
              </Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </FieldContent>
    </Field>
    <Field orientation="horizontal" class="cursor-pointer self-end sm:col-span-3">
      <FieldContent>
        <Checkbox bind:checked={onlyAffectActiveThumbnails} />
      </FieldContent>
      <FieldLabel class="text-muted-foreground">Only active thumbnails</FieldLabel>
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
