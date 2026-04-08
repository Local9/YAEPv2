<script lang="ts">
  import type { Edge, Node } from "@xyflow/svelte";
  import type { OnSelectionChange } from "@xyflow/svelte";

  import PiTemplateFlowMap from "$lib/components/pi-template/pi-template-flow-map.svelte";
  import PiTemplateSelectedStructure from "$lib/components/pi-template/pi-template-selected-structure.svelte";
  import type { PiPlacementNodeData } from "$lib/pi-template/template-to-flow";
  import type { SelectedPlacementDetail } from "$lib/pi-template/pi-template-page.types";

  let {
    flowKey,
    flowNodes,
    flowEdges,
    onselectionchange,
    selection,
    placementCount
  }: {
    flowKey: string;
    flowNodes: Node<PiPlacementNodeData>[];
    flowEdges: Edge[];
    onselectionchange: OnSelectionChange<Node<PiPlacementNodeData>>;
    selection: SelectedPlacementDetail | null;
    placementCount: number;
  } = $props();
</script>

{#key flowKey}
  <div class="flex flex-col gap-3 lg:flex-row lg:items-stretch">
    <PiTemplateFlowMap nodes={flowNodes} edges={flowEdges} {onselectionchange} />
    <PiTemplateSelectedStructure {selection} {placementCount} />
  </div>
{/key}
