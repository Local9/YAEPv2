<script lang="ts">
	import { ContextMenu as ContextMenuPrimitive } from "bits-ui";
	import { cn } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		class: className,
		...restProps
	}: ContextMenuPrimitive.SubContentProps = $props();
</script>

<ContextMenuPrimitive.SubContent
	bind:ref
	data-slot="context-menu-sub-content"
	class={cn(
		"data-open:animate-in data-closed:animate-out data-closed:fade-out-0 data-open:fade-in-0 bg-popover text-popover-foreground box-border min-w-[96px] w-max max-w-[min(22rem,calc(100vw-12px))] overflow-x-hidden overflow-y-visible rounded-md border p-1 shadow-lg duration-100",
		className
	)}
	{...restProps}
/>
