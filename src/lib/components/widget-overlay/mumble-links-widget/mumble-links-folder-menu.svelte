<script lang="ts">
  import * as Menubar from "$lib/components/ui/menubar";
  import type { MumbleFolder, MumbleLink, MumbleServerGroup } from "$models/domain";
  import MumbleFolderIcon from "$lib/mumble/mumble-folder-icon.svelte";
  import { formatMumbleServerGroupDisplayName } from "$lib/utils/mumble-display";
  import MumbleLinksItems from "./mumble-links-items.svelte";
  import {
    MUMBLE_MENU_FILLED_TRIGGER_CLASS,
    MUMBLE_MENU_SCROLL_LIST_CLASS
  } from "./mumble-links-menu-classes";
  import MumbleLinksSubfolderLinksOnlyMenu from "./mumble-links-subfolder-links-only-menu.svelte";

  let {
    group,
    folder,
    multipleServerGroups,
    foldersForParent,
    linksForFolder,
    nestedSubfoldersIgnored,
    openLink
  }: {
    group: MumbleServerGroup;
    folder: MumbleFolder;
    multipleServerGroups: boolean;
    foldersForParent: (gid: number, parentId: number | null) => MumbleFolder[];
    linksForFolder: (gid: number, folderId: number | null) => MumbleLink[];
    nestedSubfoldersIgnored: (gid: number, subfolderId: number) => boolean;
    openLink: (linkId: number) => void;
  } = $props();

  let rootLinks = $derived(linksForFolder(group.id, folder.id));
  let childFolders = $derived(foldersForParent(group.id, folder.id));

</script>

<Menubar.Menu value="mumble-g{group.id}-f{folder.id}">
  <Menubar.Trigger
    class={MUMBLE_MENU_FILLED_TRIGGER_CLASS}
    aria-label="Mumble folder {folder.name}"
  >
    <MumbleFolderIcon iconKey={folder.iconKey ?? null} class="size-3.5 shrink-0" />
    <span class="truncate">
      {#if multipleServerGroups}
        {formatMumbleServerGroupDisplayName(group.name)} / {folder.name}
      {:else}
        {folder.name}
      {/if}
    </span>
  </Menubar.Trigger>
  <Menubar.Content
    class="flex max-h-96 w-56 min-w-0 flex-col overflow-visible p-0"
    align="start"
    side="bottom"
    interactOutsideBehavior="ignore"
  >
    <div class="flex w-full min-w-0 flex-col">
      <div class={MUMBLE_MENU_SCROLL_LIST_CLASS}>
        <MumbleLinksItems links={rootLinks} openLink={openLink} />

        {#if rootLinks.length === 0 && childFolders.length === 0}
          <Menubar.Item disabled>Empty folder</Menubar.Item>
        {/if}
      </div>

      {#each childFolders as sub (sub.id)}
        <MumbleLinksSubfolderLinksOnlyMenu
          gid={group.id}
          subfolder={sub}
          linksForFolder={linksForFolder}
          nestedSubfoldersIgnored={nestedSubfoldersIgnored}
          openLink={openLink}
        />
      {/each}
    </div>
  </Menubar.Content>
</Menubar.Menu>

