<script lang="ts">
  import * as Menubar from "$lib/components/ui/menubar";
  import HeadphonesIcon from "@lucide/svelte/icons/headphones";
  import type { MumbleLink, MumbleServerGroup } from "$models/domain";
  import { formatMumbleServerGroupDisplayName } from "$lib/utils/mumble-display";

  let { group, linksForFolder, openLink }: { group: MumbleServerGroup; linksForFolder: (gid: number, folderId: number | null) => MumbleLink[]; openLink: (linkId: number) => void } =
    $props();

  let rootLinks = $derived(linksForFolder(group.id, null));
</script>

<Menubar.Menu value="mumble-g{group.id}-root">
  <Menubar.Trigger
    class="border-input bg-secondary text-secondary-foreground hover:bg-muted aria-expanded:bg-muted mumble-folder-trigger min-h-6 leading-[1.2] max-w-full min-w-0 gap-1 rounded-md border px-1.5 py-0.5 text-xs font-medium shadow-xs"
    aria-label="Mumble links for {formatMumbleServerGroupDisplayName(group.name)}"
  >
    <HeadphonesIcon class="size-3.5 shrink-0" aria-hidden="true" />
    <span class="truncate">{formatMumbleServerGroupDisplayName(group.name)}</span>
  </Menubar.Trigger>
  <Menubar.Content
    class="max-h-96 w-56 overflow-visible p-0"
    align="start"
    side="bottom"
    interactOutsideBehavior="ignore"
  >
    <div class="max-h-96 overflow-y-auto overflow-x-hidden px-1 py-1">
      {#each rootLinks as link (link.id)}
        <Menubar.Item onclick={() => openLink(link.id)}>{link.name}</Menubar.Item>
      {/each}
    </div>
  </Menubar.Content>
</Menubar.Menu>

