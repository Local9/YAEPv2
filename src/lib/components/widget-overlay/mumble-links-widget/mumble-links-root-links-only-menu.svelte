<script lang="ts">
  import * as Menubar from "$lib/components/ui/menubar";
  import HeadphonesIcon from "@lucide/svelte/icons/headphones";
  import type { MumbleLink, MumbleServerGroup } from "$models/domain";
  import { formatMumbleServerGroupDisplayName } from "$lib/utils/mumble-display";
  import MumbleLinksItems from "./mumble-links-items.svelte";
  import {
    MUMBLE_MENU_FILLED_TRIGGER_CLASS,
    MUMBLE_MENU_SCROLL_LIST_CLASS
  } from "./mumble-links-menu-classes";

  let { group, linksForFolder, openLink }: { group: MumbleServerGroup; linksForFolder: (gid: number, folderId: number | null) => MumbleLink[]; openLink: (linkId: number) => void } =
    $props();

  let rootLinks = $derived(linksForFolder(group.id, null));

</script>

<Menubar.Menu value="mumble-g{group.id}-root">
  <Menubar.Trigger
    class={MUMBLE_MENU_FILLED_TRIGGER_CLASS}
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
    <div class={MUMBLE_MENU_SCROLL_LIST_CLASS}>
      <MumbleLinksItems links={rootLinks} openLink={openLink} />
    </div>
  </Menubar.Content>
</Menubar.Menu>

