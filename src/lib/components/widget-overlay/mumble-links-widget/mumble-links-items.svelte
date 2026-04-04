<script lang="ts">
  import * as Menubar from "$lib/components/ui/menubar";
  import type { MumbleLink } from "$models/domain";

  let { links, openLink }: { links: MumbleLink[]; openLink: (linkId: number) => void } = $props();
</script>

{#each links as link (link.id)}
  <Menubar.Item onclick={() => openLink(link.id)}>{link.name}</Menubar.Item>
{/each}
