<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import * as Collapsible from "$lib/components/ui/collapsible";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import LinkIcon from "@lucide/svelte/icons/link";
  import MoreVerticalIcon from "@lucide/svelte/icons/more-vertical";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import { formatMumbleServerGroupDisplayName } from "$lib/utils/mumble-display";
  import MumbleFolderBlock from "./mumble-folder-block.svelte";
  import MumbleFolderDraftPanel from "./mumble-folder-draft-panel.svelte";
  import MumbleLinkDraftPanel from "./mumble-link-draft-panel.svelte";
  import MumbleLinkRow from "./mumble-link-row.svelte";
  import { getMumbleLinksPageContext } from "./mumble-links-context";

  const ctl = getMumbleLinksPageContext();
</script>

<div class="mt-2 space-y-6">
  {#each ctl.sortedGroups as group (group.id)}
    <Collapsible.Root class="border-border rounded-lg border" open>
      <Collapsible.Trigger
        class="hover:bg-muted/40 flex w-full items-center gap-2 px-3 py-2 text-left"
      >
        <ChevronDownIcon
          class="size-4 shrink-0 transition-transform in-data-[state=open]:rotate-180"
        />
        {#if ctl.multipleServerGroups}
          <span class="text-foreground min-w-0 flex-1 font-medium"
            >{formatMumbleServerGroupDisplayName(group.name)}</span
          >
        {:else}
          <span class="min-w-0 flex-1"></span>
        {/if}
        <div
          class="ml-auto shrink-0"
          onclick={(e) => e.stopPropagation()}
          onkeydown={(e) => e.stopPropagation()}
          role="presentation"
        >
          <DropdownMenu.Root>
            <DropdownMenu.Trigger>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                class="text-muted-foreground size-8"
                aria-label="Server actions"
              >
                <MoreVerticalIcon class="size-4" aria-hidden="true" />
              </Button>
            </DropdownMenu.Trigger>
            <DropdownMenu.Content align="end">
              <DropdownMenu.Item onclick={() => ctl.beginRootFolderDraft(group.id)}>
                <PlusIcon class="mr-2 size-4" aria-hidden="true" />
                Add folder
              </DropdownMenu.Item>
              <DropdownMenu.Item onclick={() => ctl.beginLinkDraft(group.id, null)}>
                <LinkIcon class="mr-2 size-4" aria-hidden="true" />
                Add link
              </DropdownMenu.Item>
            </DropdownMenu.Content>
          </DropdownMenu.Root>
        </div>
      </Collapsible.Trigger>
      <Collapsible.Content class="px-3 py-3">
        <MumbleFolderDraftPanel gid={group.id} parentId={null} depth={0} />
        <MumbleLinkDraftPanel gid={group.id} fid={null} />
        {#each ctl.linksForFolder(group.id, null) as link (link.id)}
          <MumbleLinkRow {link} depth={0} />
        {/each}
        {#each ctl.foldersForParent(group.id, null) as folder (folder.id)}
          <MumbleFolderBlock {folder} depth={0} />
        {/each}
      </Collapsible.Content>
    </Collapsible.Root>
  {/each}
</div>
