<script lang="ts">
  import type { MumbleFolder } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import * as Collapsible from "$lib/components/ui/collapsible";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Input } from "$lib/components/ui/input";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import LinkIcon from "@lucide/svelte/icons/link";
  import MoreVerticalIcon from "@lucide/svelte/icons/more-vertical";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import MumbleFolderIcon from "$lib/mumble/mumble-folder-icon.svelte";
  import { MUMBLE_FOLDER_ICON_OPTIONS } from "$lib/mumble/folder-icon-keys";
  import MumbleFolderBlock from "./mumble-folder-block.svelte";
  import MumbleFolderDraftPanel from "./mumble-folder-draft-panel.svelte";
  import MumbleLinkDraftPanel from "./mumble-link-draft-panel.svelte";
  import MumbleLinkRow from "./mumble-link-row.svelte";
  import { getMumbleLinksPageContext } from "./mumble-links-context";

  const ctl = getMumbleLinksPageContext();

  interface Props {
    folder: MumbleFolder;
    depth: number;
  }

  let { folder, depth }: Props = $props();

  const children = $derived(ctl.foldersForParent(folder.serverGroupId, folder.id));
  const linksHere = $derived(ctl.linksForFolder(folder.serverGroupId, folder.id));
</script>

<div class="min-w-0" style:margin-left={`${depth * 16}px`}>
  <Collapsible.Root
    class="mt-2 min-w-0 pl-2"
    open={ctl.isFolderExpanded(folder.id)}
    onOpenChange={(expanded: boolean) => ctl.setFolderExpanded(folder.id, expanded)}
  >
    <div class="hover:bg-muted/50 flex w-full min-w-0 items-center gap-2 rounded-md px-1 py-1">
      <div class="flex min-w-0 flex-1 flex-wrap items-center gap-2">
        <Collapsible.Trigger
          class="text-muted-foreground hover:bg-muted data-[state=open]:[&_svg]:rotate-180 inline-flex size-8 shrink-0 items-center justify-center rounded-md bg-transparent"
          aria-label={ctl.isFolderExpanded(folder.id) ? "Collapse folder" : "Expand folder"}
          type="button"
        >
          <ChevronDownIcon class="size-4 shrink-0 transition-transform duration-200" />
        </Collapsible.Trigger>
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            <Button
              type="button"
              variant="outline"
              size="sm"
              class="text-muted-foreground h-8 shrink-0 gap-1 px-2"
              aria-label="Folder icon"
            >
              <MumbleFolderIcon iconKey={folder.iconKey ?? null} class="size-4 shrink-0" />
              <ChevronDownIcon class="size-3.5 shrink-0 opacity-70" aria-hidden="true" />
            </Button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content align="start" class="w-56 max-h-[min(70vh,24rem)]">
            <DropdownMenu.Item
              onclick={() => {
                folder.iconKey = null;
                void ctl.saveFolder(folder);
              }}
            >
              <MumbleFolderIcon iconKey={null} class="mr-2 size-4 shrink-0" />
              Default
            </DropdownMenu.Item>
            {#each MUMBLE_FOLDER_ICON_OPTIONS as opt (opt.key)}
              <DropdownMenu.Item
                onclick={() => {
                  folder.iconKey = opt.key;
                  void ctl.saveFolder(folder);
                }}
              >
                <MumbleFolderIcon iconKey={opt.key} class="mr-2 size-4 shrink-0" />
                {opt.label}
              </DropdownMenu.Item>
            {/each}
          </DropdownMenu.Content>
        </DropdownMenu.Root>
        <Input
          class="h-8 min-w-0 max-w-xs flex-1"
          bind:value={folder.name}
          onblur={() => void ctl.saveFolder(folder, { silent: true })}
          onkeydown={(e) => ctl.onEnterSubmit(e, () => void ctl.saveFolder(folder, { silent: true }))}
        />
        <span class="text-muted-foreground text-xs">order</span>
        <Input
          class="h-8 w-16 shrink-0"
          type="number"
          bind:value={folder.displayOrder}
          onblur={() => void ctl.saveFolder(folder, { silent: true })}
          onkeydown={(e) =>
            ctl.onEnterSubmit(e, () => void ctl.saveFolder(folder, { silent: true }))}
        />
      </div>
      <div class="ml-auto flex shrink-0 items-center">
        <DropdownMenu.Root>
          <DropdownMenu.Trigger>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              class="text-muted-foreground size-8"
              aria-label="Folder actions"
            >
              <MoreVerticalIcon class="size-4" aria-hidden="true" />
            </Button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Content align="end" class="w-44">
            <DropdownMenu.Item onclick={() => ctl.beginSubfolderDraft(folder.serverGroupId, folder.id)}>
              <PlusIcon class="mr-2 size-4" aria-hidden="true" />
              Add subfolder
            </DropdownMenu.Item>
            <DropdownMenu.Item onclick={() => ctl.beginLinkDraft(folder.serverGroupId, folder.id)}>
              <LinkIcon class="mr-2 size-4" aria-hidden="true" />
              Add link
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            <DropdownMenu.Item variant="destructive" onclick={() => ctl.openDeleteFolder(folder)}>
              Delete folder
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      </div>
    </div>
    <Collapsible.Content class="min-w-0 pl-1 pt-1">
      <MumbleFolderDraftPanel gid={folder.serverGroupId} parentId={folder.id} depth={depth + 1} />
      <MumbleLinkDraftPanel gid={folder.serverGroupId} fid={folder.id} />
      {#each linksHere as link (link.id)}
        <MumbleLinkRow {link} depth={depth + 1} />
      {/each}
      {#each children as sub (sub.id)}
        <MumbleFolderBlock folder={sub} depth={depth + 1} />
      {/each}
    </Collapsible.Content>
  </Collapsible.Root>
</div>
