<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import type { MumbleFolder, MumbleLink, MumbleTreeSnapshot } from "$models/domain";
  import WidgetWrapper from "$lib/components/widget-overlay/widget-wrapper.svelte";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import { Button } from "$lib/components/ui/button";
  import HeadphonesIcon from "@lucide/svelte/icons/headphones";

  type WidgetFrameModel = { x: number; y: number; width: number; height: number } & Record<string, unknown>;

  let {
    frame = $bindable<WidgetFrameModel>(),
    pinned = $bindable(false),
    rootEl = $bindable<HTMLElement | undefined>(undefined),
    onPersist,
    onPinnedPersist
  }: {
    frame: WidgetFrameModel;
    pinned?: boolean;
    rootEl?: HTMLElement | undefined;
    onPersist: () => void | Promise<void>;
    onPinnedPersist?: () => void | Promise<void>;
  } = $props();

  let tree = $state<MumbleTreeSnapshot | null>(null);
  let menuOpen = $state(false);

  async function loadTree() {
    try {
      tree = await backend.getMumbleTree();
    } catch {
      tree = null;
    }
  }

  $effect(() => {
    if (menuOpen) void loadTree();
  });

  onMount(() => {
    void loadTree();
    let u: UnlistenFn | undefined;
    void listen("widget-overlay-settings-changed", () => {
      void loadTree();
    }).then((fn) => {
      u = fn;
    });
    return () => u?.();
  });

  const sortedGroups = $derived(
    tree
      ? [...tree.groups].sort((a, b) => a.displayOrder - b.displayOrder || a.name.localeCompare(b.name))
      : []
  );

  function foldersForParent(gid: number, parentId: number | null): MumbleFolder[] {
    if (!tree) return [];
    return tree.folders
      .filter((f) => f.serverGroupId === gid && (f.parentFolderId ?? null) === parentId)
      .sort((a, b) => a.displayOrder - b.displayOrder || a.name.localeCompare(b.name));
  }

  function linksForFolder(gid: number, folderId: number | null): MumbleLink[] {
    if (!tree) return [];
    return tree.links
      .filter((l) => l.serverGroupId === gid && (l.folderId ?? null) === folderId)
      .sort((a, b) => a.displayOrder - b.displayOrder || a.name.localeCompare(b.name));
  }

  function openLink(linkId: number) {
    void backend.openMumbleLink(linkId);
  }
</script>

{#snippet folderBranch(folder: MumbleFolder)}
  {@const linksHere = linksForFolder(folder.serverGroupId, folder.id)}
  {@const subfolders = foldersForParent(folder.serverGroupId, folder.id)}
  <DropdownMenu.Sub>
    <DropdownMenu.SubTrigger class="cursor-default">{folder.name}</DropdownMenu.SubTrigger>
    <DropdownMenu.SubContent class="max-h-72 overflow-y-auto">
      {#each linksHere as link (link.id)}
        <DropdownMenu.Item onclick={() => openLink(link.id)}>{link.name}</DropdownMenu.Item>
      {/each}
      {#each subfolders as sub (sub.id)}
        {@render folderBranch(sub)}
      {/each}
      {#if linksHere.length === 0 && subfolders.length === 0}
        <DropdownMenu.Item disabled>Empty folder</DropdownMenu.Item>
      {/if}
    </DropdownMenu.SubContent>
  </DropdownMenu.Sub>
{/snippet}

<WidgetWrapper
  title="Mumble"
  shellAriaLabel="Mumble links widget"
  bind:frame
  bind:pinned
  bind:rootEl
  {onPersist}
  onPinnedPersist={onPinnedPersist ?? onPersist}
  minWidth={160}
  minHeight={72}
>
  {#snippet children()}
    <div class="flex flex-col gap-2 p-2">
      <DropdownMenu.Root bind:open={menuOpen}>
        <DropdownMenu.Trigger>
          <Button variant="secondary" class="h-9 w-full gap-2 text-sm" type="button">
            <HeadphonesIcon class="size-4 shrink-0" aria-hidden="true" />
            Links
          </Button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Content class="max-h-96 w-56 overflow-y-auto" align="start">
          {#if sortedGroups.length === 0}
            <DropdownMenu.Item disabled>No servers configured</DropdownMenu.Item>
          {:else}
            {#each sortedGroups as group (group.id)}
              <DropdownMenu.Sub>
                <DropdownMenu.SubTrigger class="cursor-default">{group.name}</DropdownMenu.SubTrigger>
                <DropdownMenu.SubContent class="max-h-72 overflow-y-auto">
                  {#each linksForFolder(group.id, null) as link (link.id)}
                    <DropdownMenu.Item onclick={() => openLink(link.id)}>{link.name}</DropdownMenu.Item>
                  {/each}
                  {#each foldersForParent(group.id, null) as folder (folder.id)}
                    {@render folderBranch(folder)}
                  {/each}
                  {#if linksForFolder(group.id, null).length === 0 && foldersForParent(group.id, null).length === 0}
                    <DropdownMenu.Item disabled>Empty server</DropdownMenu.Item>
                  {/if}
                </DropdownMenu.SubContent>
              </DropdownMenu.Sub>
            {/each}
          {/if}
        </DropdownMenu.Content>
      </DropdownMenu.Root>
    </div>
  {/snippet}
</WidgetWrapper>
