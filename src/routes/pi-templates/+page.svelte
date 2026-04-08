<script lang="ts">
  import { onMount } from "svelte";
  import { Background, Controls, SvelteFlow } from "@xyflow/svelte";
  import type { Edge, Node } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";

  import PiPlacementNode from "$lib/components/pi-template/pi-placement-node.svelte";
  import { piTemplateToFlowElements, type PiPlacementNodeData } from "$lib/pi-template/template-to-flow";
  import { backend } from "$services/backend";
  import type { PiTemplate } from "$models/domain";

  const nodeTypes = { piPlacement: PiPlacementNode };

  let templateFiles = $state<string[]>([]);
  let selectedFile = $state("");
  let templateModel = $state<PiTemplate | null>(null);

  let flowNodes = $state<Node<PiPlacementNodeData>[]>([]);
  let flowEdges = $state<Edge[]>([]);

  let listLoading = $state(true);
  let listError = $state("");
  let contentLoading = $state(false);
  let contentError = $state("");

  let decodedComment = $derived(
    templateModel ? decodeHtmlEntities(templateModel.Cmt) : ""
  );

  let templateSummary = $derived.by(() => {
    if (!templateModel) return null;
    return {
      cmdCtrLv: templateModel.CmdCtrLv,
      diam: templateModel.Diam,
      pln: templateModel.Pln,
      placements: templateModel.P.length,
      routes: templateModel.R.length,
      links: templateModel.L.length
    };
  });

  function decodeHtmlEntities(value: string): string {
    if (!value || typeof document === "undefined") {
      return value;
    }

    const textarea = document.createElement("textarea");
    textarea.innerHTML = value;
    return textarea.value;
  }

  $effect(() => {
    const t = templateModel;
    if (!t) {
      flowNodes = [];
      flowEdges = [];
      return;
    }
    const { nodes, edges } = piTemplateToFlowElements(t);
    flowNodes = nodes;
    flowEdges = edges;
  });

  async function loadTemplateList() {
    listLoading = true;
    listError = "";
    templateFiles = [];
    selectedFile = "";
    templateModel = null;
    contentError = "";

    try {
      const files = await backend.eveListPiTemplates();
      templateFiles = files;
      if (files.length > 0) {
        selectedFile = files[0];
        await loadTemplateContent(files[0]);
      }
    } catch (error) {
      listError = error instanceof Error ? error.message : String(error);
    } finally {
      listLoading = false;
    }
  }

  async function loadTemplateContent(fileName: string) {
    if (!fileName) {
      templateModel = null;
      contentError = "";
      return;
    }

    contentLoading = true;
    contentError = "";
    templateModel = null;

    try {
      templateModel = await backend.eveReadPiTemplateJson(fileName);
    } catch (error) {
      contentError = error instanceof Error ? error.message : String(error);
    } finally {
      contentLoading = false;
    }
  }

  async function onTemplateChange() {
    await loadTemplateContent(selectedFile);
  }

  onMount(() => {
    void loadTemplateList();
  });
</script>

<div class="flex max-w-6xl flex-col gap-4">
  <h1 class="text-xl font-semibold">PI Templates</h1>
  <p class="text-sm text-muted-foreground">
    Visualize colony layouts from your PlanetaryInteractionTemplates folder. Placements show structure type
    IDs (T); names can be resolved later. Edges follow material routes (R).
  </p>

  {#if listLoading}
    <p class="text-sm text-muted-foreground">Loading templates...</p>
  {:else if listError}
    <p class="text-sm text-destructive">{listError}</p>
  {:else if templateFiles.length === 0}
    <p class="text-sm text-muted-foreground">No PI templates were found.</p>
  {:else}
    <div class="space-y-2">
      <label for="pi-template-file" class="text-sm font-medium">Template file</label>
      <select
        id="pi-template-file"
        class="w-full max-w-xl rounded-md border border-input bg-background px-3 py-2 text-sm"
        bind:value={selectedFile}
        onchange={() => void onTemplateChange()}
      >
        {#each templateFiles as fileName (fileName)}
          <option value={fileName}>{fileName}</option>
        {/each}
      </select>
    </div>

    {#if contentLoading}
      <p class="text-sm text-muted-foreground">Loading template...</p>
    {:else if contentError}
      <p class="text-sm text-destructive">{contentError}</p>
    {:else if templateModel && templateSummary}
      <div class="space-y-1 rounded-md border bg-muted/20 p-3">
        <h2 class="text-sm font-semibold">Comment (decoded)</h2>
        <p class="text-sm wrap-break-word">{decodedComment || "No comment."}</p>
      </div>

      <dl
        class="grid max-w-xl grid-cols-2 gap-x-4 gap-y-1 rounded-md border bg-muted/10 px-3 py-2 text-xs sm:grid-cols-3"
      >
        <div><dt class="text-muted-foreground">CmdCtrLv</dt><dd class="font-mono">{templateSummary.cmdCtrLv}</dd></div>
        <div><dt class="text-muted-foreground">Diam</dt><dd class="font-mono">{templateSummary.diam}</dd></div>
        <div><dt class="text-muted-foreground">Pln</dt><dd class="font-mono">{templateSummary.pln}</dd></div>
        <div><dt class="text-muted-foreground">Placements</dt><dd class="font-mono">{templateSummary.placements}</dd></div>
        <div><dt class="text-muted-foreground">Routes</dt><dd class="font-mono">{templateSummary.routes}</dd></div>
        <div><dt class="text-muted-foreground">Links (L)</dt><dd class="font-mono">{templateSummary.links}</dd></div>
      </dl>

      {#key selectedFile}
        <div class="relative h-[min(70vh,560px)] w-full overflow-hidden rounded-md border bg-background">
          <SvelteFlow
            nodes={flowNodes}
            edges={flowEdges}
            {nodeTypes}
            fitView
            fitViewOptions={{ padding: 0.15, minZoom: 0.05, maxZoom: 1.5 }}
            nodesDraggable={false}
            nodesConnectable={false}
            elementsSelectable={true}
            class="h-full w-full"
          >
            <Background gap={16} size={1} />
            <Controls showLock={false} />
          </SvelteFlow>
        </div>
      {/key}
    {/if}
  {/if}
</div>
