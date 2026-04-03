<script lang="ts">
  import * as Menubar from "$lib/components/ui/menubar";
  import HeadphonesIcon from "@lucide/svelte/icons/headphones";
  import type { MumbleFolder, MumbleLink, MumbleServerGroup } from "$models/domain";
  import { formatMumbleServerGroupDisplayName } from "$lib/utils/mumble-display";
  import MumbleLinksFolderMenu from "./mumble-links-folder-menu.svelte";
  import MumbleLinksRootLinksOnlyMenu from "./mumble-links-root-links-only-menu.svelte";

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

<Menubar.Root bind:value={menubarValue} loop>
  {#if sortedGroups.length === 0}
    <Menubar.Menu value="mumble-empty">
      <Menubar.Trigger
        class="text-muted-foreground mumble-folder-trigger min-h-6 leading-[1.2] max-w-full min-w-0 gap-1 rounded-md px-1.5 py-0.5 text-xs font-medium"
        disabled
      >
        <HeadphonesIcon class="size-3.5 shrink-0" aria-hidden="true" />
        <span class="truncate">Mumble</span>
      </Menubar.Trigger>
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
          <Menubar.Trigger
            class="text-muted-foreground mumble-folder-trigger min-h-6 leading-[1.2] max-w-full min-w-0 gap-1 rounded-md px-1.5 py-0.5 text-xs font-medium"
            disabled
          >
            <HeadphonesIcon class="size-3.5 shrink-0" aria-hidden="true" />
            <span class="truncate">{formatMumbleServerGroupDisplayName(group.name)}</span>
          </Menubar.Trigger>
        </Menubar.Menu>
      {/if}
    {/each}
  {/if}
</Menubar.Root>

