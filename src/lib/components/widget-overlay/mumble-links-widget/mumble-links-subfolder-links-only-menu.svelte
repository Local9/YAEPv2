<script lang="ts">
  import * as Menubar from "$lib/components/ui/menubar";
  import MumbleFolderIcon from "$lib/mumble/mumble-folder-icon.svelte";
  import type { MumbleFolder, MumbleLink } from "$models/domain";
  import MumbleLinksItems from "./mumble-links-items.svelte";

  let {
    gid,
    subfolder,
    linksForFolder,
    nestedSubfoldersIgnored,
    openLink
  }: {
    gid: number;
    subfolder: MumbleFolder;
    linksForFolder: (gid: number, folderId: number | null) => MumbleLink[];
    nestedSubfoldersIgnored: (gid: number, subfolderId: number) => boolean;
    openLink: (linkId: number) => void;
  } = $props();

  let folderLinks = $derived(linksForFolder(gid, subfolder.id));
</script>

<Menubar.Sub>
  <Menubar.SubTrigger class="w-full max-w-full min-w-0 cursor-default">
    <MumbleFolderIcon iconKey={subfolder.iconKey ?? null} class="size-3.5 shrink-0" />
    <span class="truncate">{subfolder.name}</span>
  </Menubar.SubTrigger>
  <Menubar.SubContent
    class="max-h-[min(18rem,85dvh)] overflow-y-auto overflow-x-hidden p-1"
    interactOutsideBehavior="ignore"
  >
    <MumbleLinksItems links={folderLinks} openLink={openLink} />
    {#if folderLinks.length === 0}
      <Menubar.Item disabled>Empty folder</Menubar.Item>
    {/if}
    {#if nestedSubfoldersIgnored(gid, subfolder.id)}
      <Menubar.Item disabled>Nested folders: edit on Mumble Links page</Menubar.Item>
    {/if}
  </Menubar.SubContent>
</Menubar.Sub>