<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { GridLayoutPayload, GridLayoutPreviewItem, Profile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow
  } from "$lib/components/ui/table";

  let profiles = $state<Profile[]>([]);
  let activeProfileId = $state<number | null>(null);
  let gridCellWidth = $state(400);
  let gridCellHeight = $state(300);
  let gridStartX = $state(100);
  let gridStartY = $state(100);
  let gridColumns = $state(3);
  let onlyAffectActiveThumbnails = $state(false);
  let preview = $state<GridLayoutPreviewItem[]>([]);
  let status = $state("");
  let error = $state("");

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
      onlyAffectActiveThumbnails
    };
  }

  async function loadContext() {
    profiles = await backend.getProfiles();
    activeProfileId = profiles.find((p) => p.isActive)?.id ?? null;
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

<section class="card">
  <h2>Grid Layout</h2>
  <p>Build preview and apply bulk layout updates.</p>

  <div style="display:grid; grid-template-columns: repeat(3, minmax(140px, 1fr)); gap:0.5rem; max-width: 760px;">
    <label>Cell Width <Input type="number" bind:value={gridCellWidth} /></label>
    <label>Cell Height <Input type="number" bind:value={gridCellHeight} /></label>
    <label>Columns <Input type="number" bind:value={gridColumns} /></label>
    <label>Start X <Input type="number" bind:value={gridStartX} /></label>
    <label>Start Y <Input type="number" bind:value={gridStartY} /></label>
    <label style="display:flex; align-items:center; gap:0.5rem; margin-top:1.4rem;">
      <input type="checkbox" bind:checked={onlyAffectActiveThumbnails} />
      Only active thumbnails
    </label>
  </div>

  <div style="display:flex; gap:0.5rem; margin-top:0.75rem;">
    <Button onclick={generatePreview}>Generate Preview</Button>
    <Button onclick={applyLayout}>Apply Layout</Button>
  </div>

  {#if status}
    <Alert style="margin-top:0.75rem;">
      <AlertTitle>Grid Layout</AlertTitle>
      <AlertDescription>{status}</AlertDescription>
    </Alert>
  {/if}
  {#if error}
    <Alert style="margin-top:0.75rem;">
      <AlertTitle>Error</AlertTitle>
      <AlertDescription>{error}</AlertDescription>
    </Alert>
  {/if}

  <div style="margin-top:1rem;">
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
