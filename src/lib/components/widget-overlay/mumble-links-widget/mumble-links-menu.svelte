<script lang="ts">
  import * as Menubar from "$lib/components/ui/menubar";
  import HeadphonesIcon from "@lucide/svelte/icons/headphones";
  import type { MumbleFolder, MumbleLink, MumbleServerGroup } from "$models/domain";
  import { formatMumbleServerGroupDisplayName } from "$lib/utils/mumble-display";
  import MumbleLinksFolderMenu from "./mumble-links-folder-menu.svelte";
  import MumbleLinksRootLinksOnlyMenu from "./mumble-links-root-links-only-menu.svelte";
  import { MUMBLE_MENU_EMPTY_TRIGGER_CLASS } from "./mumble-links-menu-classes";

  let {
    menubarValue = $bindable(""),
    sortedGroups,
    multipleServerGroups,
    foldersForParent,
    linksForFolder,
    nestedSubfoldersIgnored,
    openLink
  }: {
    menubarValue?: string;
    sortedGroups: MumbleServerGroup[];
    multipleServerGroups: boolean;
    foldersForParent: (gid: number, parentId: number | null) => MumbleFolder[];
    linksForFolder: (gid: number, folderId: number | null) => MumbleLink[];
    nestedSubfoldersIgnored: (gid: number, subfolderId: number) => boolean;
    openLink: (linkId: number) => void;
  } = $props();

</script>

{#snippet emptyGroupTrigger(label = "")}
  <Menubar.Trigger class={MUMBLE_MENU_EMPTY_TRIGGER_CLASS} disabled>
    <HeadphonesIcon class="size-3.5 shrink-0" aria-hidden="true" />
    <span class="truncate">{label}</span>
  </Menubar.Trigger>
{/snippet}

<Menubar.Root bind:value={menubarValue} loop>
  {#if sortedGroups.length === 0}
    <Menubar.Menu value="mumble-empty">
      {@render emptyGroupTrigger("Mumble")}
    </Menubar.Menu>
  {:else}
    {#each sortedGroups as group (group.id)}
      {@const rootFolders = foldersForParent(group.id, null)}
      {@const rootLinks = linksForFolder(group.id, null)}

      {#if rootFolders.length > 0}
        {#each rootFolders as folder (folder.id)}
          <MumbleLinksFolderMenu
            group={group}
            folder={folder}
            {multipleServerGroups}
            foldersForParent={foldersForParent}
            linksForFolder={linksForFolder}
            nestedSubfoldersIgnored={nestedSubfoldersIgnored}
            openLink={openLink}
          />
        {/each}
      {:else if rootLinks.length > 0}
        <MumbleLinksRootLinksOnlyMenu
          group={group}
          linksForFolder={linksForFolder}
          openLink={openLink}
        />
      {:else}
        <Menubar.Menu value="mumble-g{group.id}-empty">
          {@render emptyGroupTrigger(formatMumbleServerGroupDisplayName(group.name))}
        </Menubar.Menu>
      {/if}
    {/each}
  {/if}
</Menubar.Root>

