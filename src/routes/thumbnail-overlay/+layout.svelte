<script lang="ts">
  import "./overlay-surface.css";
  import type { LayoutProps } from "./$types";

  let { children }: LayoutProps = $props();
</script>

{@render children?.()}
