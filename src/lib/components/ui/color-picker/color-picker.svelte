<script lang="ts">
  import { cn, type WithElementRef, type WithoutChildren } from "$lib/utils.js";
  import type { HTMLInputAttributes } from "svelte/elements";

  let {
    ref = $bindable<HTMLInputElement | null>(null),
    value = $bindable("#d47800"),
    class: className,
    ...restProps
  }: WithoutChildren<WithElementRef<HTMLInputAttributes>> = $props();
</script>

<input
  bind:this={ref}
  type="color"
  bind:value
  data-slot="color-picker"
  class={cn(
    "h-10 w-12 cursor-pointer rounded-md border border-input bg-background p-1",
    className,
  )}
  {...restProps}
/>
