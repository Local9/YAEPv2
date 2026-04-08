<script lang="ts">
  import { Handle, Position } from "@xyflow/svelte";
  import type { Node, NodeProps } from "@xyflow/svelte";
  import {
    PI_NODE_HEIGHT,
    PI_NODE_WIDTH,
    type PiPlacementNodeData
  } from "$lib/pi-template/template-to-flow";

  let { data, selected }: NodeProps<Node<PiPlacementNodeData, "piPlacement">> = $props();

  const handleClass = "size-2 border border-border bg-muted";

  let displayTitle = $derived(
    data.typeDetails?.name?.trim() ? data.typeDetails.name.trim() : `Type ${data.typeId}`
  );
</script>

<!-- Size matches PI_NODE_WIDTH / PI_NODE_HEIGHT in template-to-flow.ts -->
<div
  class="flex items-center justify-center rounded-full border bg-card text-center shadow-sm {selected
    ? 'border-primary ring-2 ring-primary/40 ring-offset-2 ring-offset-background'
    : 'border-border'}"
  style:width="{PI_NODE_WIDTH}px"
  style:height="{PI_NODE_HEIGHT}px"
  title={displayTitle}
>
  <span class="line-clamp-3 px-2 text-[10px] font-semibold leading-tight text-foreground">{displayTitle}</span>
</div>

<Handle id="in-top" type="target" position={Position.Top} class={handleClass} />
<Handle id="out-top" type="source" position={Position.Top} class={handleClass} />
<Handle id="in-right" type="target" position={Position.Right} class={handleClass} />
<Handle id="out-right" type="source" position={Position.Right} class={handleClass} />
<Handle id="in-bottom" type="target" position={Position.Bottom} class={handleClass} />
<Handle id="out-bottom" type="source" position={Position.Bottom} class={handleClass} />
<Handle id="in-left" type="target" position={Position.Left} class={handleClass} />
<Handle id="out-left" type="source" position={Position.Left} class={handleClass} />
