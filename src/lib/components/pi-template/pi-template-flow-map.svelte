<script lang="ts">
  import { Background, Controls, SvelteFlow } from "@xyflow/svelte";
  import type { Edge, Node } from "@xyflow/svelte";
  import type { OnSelectionChange } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";

  import PiPlacementNode from "$lib/components/pi-template/pi-placement-node.svelte";
  import type { PiPlacementNodeData } from "$lib/pi-template/template-to-flow";

  const nodeTypes = { piPlacement: PiPlacementNode };

  let {
    nodes,
    edges,
    onselectionchange
  }: {
    nodes: Node<PiPlacementNodeData>[];
    edges: Edge[];
    onselectionchange: OnSelectionChange<Node<PiPlacementNodeData>>;
  } = $props();
</script>

<div class="relative h-[min(70vh,560px)] min-h-[320px] flex-1 overflow-hidden rounded-md border bg-background">
  <SvelteFlow
    {nodes}
    {edges}
    colorMode="system"
    proOptions={{ hideAttribution: true }}
    {nodeTypes}
    fitView
    fitViewOptions={{ padding: 0.15, minZoom: 0.05, maxZoom: 1.5 }}
    nodesDraggable={false}
    nodesConnectable={false}
    elementsSelectable={true}
    {onselectionchange}
    class="h-full w-full"
  >
    <Background gap={16} size={1} />
    <Controls showLock={false} />
  </SvelteFlow>
</div>
