<script lang="ts">
  import { Handle, Position } from "@xyflow/svelte";
  import type { Node, NodeProps } from "@xyflow/svelte";
  import type { PiPlacementNodeData } from "$lib/pi-template/template-to-flow";

  let { data }: NodeProps<Node<PiPlacementNodeData, "piPlacement">> = $props();
</script>

<div class="min-w-28 rounded-md border border-border bg-card px-2.5 py-2 text-xs shadow-sm">
  <div class="font-mono text-sm font-semibold tabular-nums">T {data.typeId}</div>
  {#if data.structureId != null}
    <div class="mt-0.5 font-mono text-[11px] text-muted-foreground tabular-nums">S {data.structureId}</div>
  {:else}
    <div class="mt-0.5 text-[11px] text-muted-foreground">S -</div>
  {/if}
</div>

<Handle type="target" position={Position.Left} class="size-2 border border-border bg-muted" />
<Handle type="source" position={Position.Right} class="size-2 border border-border bg-muted" />
