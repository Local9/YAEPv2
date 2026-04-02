<script lang="ts">
	import { Menubar as MenubarPrimitive } from "bits-ui";
	import { cn } from "$lib/utils.js";

	let {
		ref = $bindable(null),
		value = $bindable(""),
		class: className,
		...restProps
	}: MenubarPrimitive.RootProps = $props();
</script>

<MenubarPrimitive.Root
	bind:ref
	bind:value
	data-slot="menubar"
	class={cn("h-9 gap-1 rounded-md border p-1 shadow-xs flex items-center", className)}
	{...restProps}
/>
