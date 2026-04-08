<script lang="ts">
  import { onMount } from "svelte";
  import type { Edge, Node } from "@xyflow/svelte";
  import type { OnSelectionChange } from "@xyflow/svelte";

  import PiTemplateDecodedComment from "$lib/components/pi-template/pi-template-decoded-comment.svelte";
  import PiTemplateFileSelect from "$lib/components/pi-template/pi-template-file-select.svelte";
  import PiTemplateIntro from "$lib/components/pi-template/pi-template-intro.svelte";
  import PiTemplateSdeStatus from "$lib/components/pi-template/pi-template-sde-status.svelte";
  import PiTemplateSummaryGrid from "$lib/components/pi-template/pi-template-summary-stats.svelte";
  import PiTemplateWorkspace from "$lib/components/pi-template/pi-template-workspace.svelte";
  import type {
    PiTemplateSummaryStats,
    SelectedPlacementDetail
  } from "$lib/pi-template/pi-template-page.types";
  import {
    piTemplateToFlowElements,
    type PiPlacementNodeData,
    type PiRouteMaterialStub
  } from "$lib/pi-template/template-to-flow";
  import { backend } from "$services/backend";
  import type { EveStaticDataStatus, EveTypeSnapshot, PiTemplate } from "$models/domain";

  let templateFiles = $state<string[]>([]);
  let selectedFile = $state("");
  let templateModel = $state<PiTemplate | null>(null);
  let piTypesById = $state<Record<string, EveTypeSnapshot>>({});

  let flowNodes = $state<Node<PiPlacementNodeData>[]>([]);
  let flowEdges = $state<Edge[]>([]);

  let listLoading = $state(true);
  let listError = $state("");
  let contentLoading = $state(false);
  let contentError = $state("");
  let typesLookupPending = $state(false);
  let typesLookupError = $state("");
  let selectedFlowNodeId = $state<string | null>(null);
  let eveStaticStatus = $state<EveStaticDataStatus | null>(null);

  let staticDataCatalogEmpty = $derived(
    eveStaticStatus != null && eveStaticStatus.sdeSqliteTypesCount === 0
  );

  /** Stable signature when SDE map changes; forces SvelteFlow remount so XYFlow reapplies node `data` (adoptUserNodes skips updates when userNode reference is unchanged). */
  let piTypesLookupSignature = $derived.by(() => Object.keys(piTypesById).sort().join("\u001f"));

  let flowRemountKey = $derived(`${selectedFile}\u0000${piTypesLookupSignature}`);

  let decodedComment = $derived(
    templateModel ? decodeHtmlEntities(templateModel.Cmt) : ""
  );

  let templateSummary = $derived.by((): PiTemplateSummaryStats | null => {
    if (!templateModel) return null;
    const plnId = Number(templateModel.Pln);
    const plnSnap = Number.isFinite(plnId) ? piTypesById[String(plnId)] ?? null : null;
    let totalStructureCostIsk = 0;
    let placementsMissingBasePrice = 0;
    for (const pin of templateModel.P) {
      const snap = piTypesById[String(pin.T)];
      const p = snap?.basePrice;
      if (p != null && Number.isFinite(p)) {
        totalStructureCostIsk += p;
      } else {
        placementsMissingBasePrice++;
      }
    }
    return {
      cmdCtrLv: templateModel.CmdCtrLv,
      diam: templateModel.Diam,
      plnId: templateModel.Pln,
      plnTypeName: plnSnap?.name?.trim() ?? null,
      placements: templateModel.P.length,
      routes: templateModel.R.length,
      links: templateModel.L.length,
      totalStructureCostIsk,
      placementsMissingBasePrice
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
    selectedFlowNodeId = null;
    if (!t) {
      piTypesById = {};
      typesLookupPending = false;
      typesLookupError = "";
      return;
    }
    // PI-Template-Field-Reference: P[].T = structure type IDs, R[].T = routed commodity type IDs (same SDE type-id space).
    const fromPins = t.P.map((p) => Number(p.T));
    const fromRoutes = t.R.map((r) => Number(r.T));
    const plnId = Number(t.Pln);
    const ids = [
      ...new Set([
        ...fromPins,
        ...fromRoutes,
        ...(Number.isFinite(plnId) ? [plnId] : [])
      ])
    ]
      .filter((id) => Number.isFinite(id))
      .sort((a, b) => a - b);
    let cancelled = false;
    typesLookupPending = true;
    typesLookupError = "";
    void (async () => {
      try {
        const m = await backend.eveSdeLookupTypes(ids);
        if (!cancelled) piTypesById = m && typeof m === "object" && !Array.isArray(m) ? m : {};
      } catch (e) {
        if (!cancelled) {
          piTypesById = {};
          typesLookupError = e instanceof Error ? e.message : String(e);
        }
      } finally {
        if (!cancelled) typesLookupPending = false;
      }
    })();
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    const t = templateModel;
    const types = piTypesById;
    if (!t) {
      flowNodes = [];
      flowEdges = [];
      return;
    }
    const { nodes, edges } = piTemplateToFlowElements(t, types);
    flowNodes = nodes;
    flowEdges = edges;
  });

  const onFlowSelectionChange: OnSelectionChange<Node<PiPlacementNodeData>> = (params) => {
    if (params.nodes.length === 1) {
      selectedFlowNodeId = params.nodes[0].id;
    } else {
      selectedFlowNodeId = null;
    }
  };

  /** Prefer template + live `piTypesById` so SDE fields stay correct even if flow node `data` is stale internally. */
  let selectedPlacementResolved = $derived.by((): SelectedPlacementDetail | null => {
    if (!templateModel || !selectedFlowNodeId) return null;
    const m = /^p-(\d+)$/.exec(selectedFlowNodeId);
    if (!m) return null;
    const idx = Number(m[1]);
    const pin = templateModel.P[idx];
    if (!pin) return null;
    const typeId = Number(pin.T);
    const typeDetails = piTypesById[String(typeId)] ?? null;
    const node = flowNodes.find((n) => n.id === selectedFlowNodeId);
    const incomingRoutes: PiRouteMaterialStub[] = node?.data.incomingRoutes ?? [];
    const outgoingRoutes: PiRouteMaterialStub[] = node?.data.outgoingRoutes ?? [];
    return {
      typeId,
      structureId: pin.S,
      index: idx,
      layerH: pin.H,
      typeDetails,
      incomingRoutes,
      outgoingRoutes
    };
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
    void backend
      .eveStaticDataStatus()
      .then((s) => {
        eveStaticStatus = s;
      })
      .catch(() => {
        eveStaticStatus = null;
      });
  });
</script>

<div class="flex max-w-6xl flex-col gap-4">
  <PiTemplateIntro staticDataCatalogEmpty={staticDataCatalogEmpty} />

  {#if listLoading}
    <p class="text-sm text-muted-foreground">Loading templates...</p>
  {:else if listError}
    <p class="text-sm text-destructive">{listError}</p>
  {:else if templateFiles.length === 0}
    <p class="text-sm text-muted-foreground">No PI templates were found.</p>
  {:else}
    <PiTemplateFileSelect bind:selectedFile {templateFiles} onfilechange={() => void onTemplateChange()} />

    {#if contentLoading}
      <p class="text-sm text-muted-foreground">Loading template...</p>
    {:else if contentError}
      <p class="text-sm text-destructive">{contentError}</p>
    {:else if templateModel && templateSummary}
      <PiTemplateDecodedComment text={decodedComment} />

      <PiTemplateSummaryGrid summary={templateSummary} />

      <PiTemplateSdeStatus
        pending={typesLookupPending}
        error={typesLookupError}
        showEmptyCatalogHint={templateModel != null &&
          Object.keys(piTypesById).length === 0 &&
          !typesLookupPending}
      />

      <PiTemplateWorkspace
        flowKey={flowRemountKey}
        flowNodes={flowNodes}
        flowEdges={flowEdges}
        onselectionchange={onFlowSelectionChange}
        selection={selectedPlacementResolved}
        placementCount={templateModel.P.length}
      />
    {/if}
  {/if}
</div>
