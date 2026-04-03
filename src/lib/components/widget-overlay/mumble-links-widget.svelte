<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import type { MumbleFolder, MumbleLink, MumbleTreeSnapshot } from "$models/domain";
  import { formatMumbleServerGroupDisplayName } from "$lib/utils/mumble-display";
  import MumbleFolderIcon from "$lib/mumble/mumble-folder-icon.svelte";
  import * as Menubar from "$lib/components/ui/menubar";
  import GripVerticalIcon from "@lucide/svelte/icons/grip-vertical";
  import HeadphonesIcon from "@lucide/svelte/icons/headphones";
  import PinIcon from "@lucide/svelte/icons/pin";

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

  const MIN_SHELL_WIDTH = 96;
  const COMPACT_SHELL_HEIGHT = 30;

  let tree = $state<MumbleTreeSnapshot | null>(null);
  /** Which menubar menu is open (bits-ui menubar `value`). */
  let menubarValue = $state("");
  let drag = $state<{ dx: number; dy: number } | null>(null);
  let resizeWidth = $state<{ startX: number; startW: number } | null>(null);

  async function loadTree() {
    try {
      tree = await backend.getMumbleTree();
    } catch {
      tree = null;
    }
  }

  $effect(() => {
    if (menubarValue) void loadTree();
  });

  onMount(() => {
    void loadTree();
    let u1: UnlistenFn | undefined;
    let u2: UnlistenFn | undefined;
    void listen("widget-overlay-settings-changed", () => {
      void loadTree();
    }).then((fn) => {
      u1 = fn;
    });
    void listen("mumble-tree-changed", () => {
      void loadTree();
    }).then((fn) => {
      u2 = fn;
    });
    return () => {
      u1?.();
      u2?.();
    };
  });

  const sortedGroups = $derived(
    tree
      ? [...tree.groups].sort((a, b) => a.displayOrder - b.displayOrder || a.name.localeCompare(b.name))
      : []
  );

  const multipleServerGroups = $derived(sortedGroups.length > 1);

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

  /** Widget shows at most one subfolder level; deeper DB folders are omitted here. */
  function nestedSubfoldersIgnored(gid: number, subfolderId: number): boolean {
    return foldersForParent(gid, subfolderId).length > 0;
  }

  function onPointerMove(e: PointerEvent) {
    if (!drag) return;
    const nx = e.clientX - drag.dx;
    const ny = e.clientY - drag.dy;
    frame = { ...frame, x: nx, y: ny };
  }

  async function endDrag() {
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", endDrag);
    window.removeEventListener("pointercancel", endDrag);
    drag = null;
    try {
      await invoke("widget_overlay_set_dragging", { dragging: false });
    } catch {
      /* dev */
    }
    await onPersist();
  }

  function onGripPointerDown(e: PointerEvent) {
    e.preventDefault();
    drag = {
      dx: e.clientX - frame.x,
      dy: e.clientY - frame.y
    };
    void invoke("widget_overlay_set_dragging", { dragging: true }).catch(() => {});
    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", endDrag);
    window.addEventListener("pointercancel", endDrag);
  }

  function onResizeWidthMove(e: PointerEvent) {
    if (!resizeWidth) return;
    const nw = resizeWidth.startW + (e.clientX - resizeWidth.startX);
    const maxW = Math.max(MIN_SHELL_WIDTH, window.innerWidth - frame.x - 8);
    frame = {
      ...frame,
      width: Math.min(Math.max(MIN_SHELL_WIDTH, nw), maxW),
      height: COMPACT_SHELL_HEIGHT
    };
  }

  async function endResizeWidth() {
    window.removeEventListener("pointermove", onResizeWidthMove);
    window.removeEventListener("pointerup", endResizeWidth);
    window.removeEventListener("pointercancel", endResizeWidth);
    resizeWidth = null;
    try {
      await invoke("widget_overlay_set_dragging", { dragging: false });
    } catch {
      /* dev */
    }
    frame = { ...frame, height: COMPACT_SHELL_HEIGHT };
    await onPersist();
  }

  function onResizeWidthPointerDown(e: PointerEvent) {
    e.preventDefault();
    e.stopPropagation();
    resizeWidth = {
      startX: e.clientX,
      startW: frame.width
    };
    void invoke("widget_overlay_set_dragging", { dragging: true }).catch(() => {});
    window.addEventListener("pointermove", onResizeWidthMove);
    window.addEventListener("pointerup", endResizeWidth);
    window.addEventListener("pointercancel", endResizeWidth);
  }

  function stopDragChain(e: PointerEvent) {
    e.stopPropagation();
  }

  async function togglePinned(e: MouseEvent) {
    e.stopPropagation();
    pinned = !pinned;
    const run = onPinnedPersist ?? onPersist;
    await run();
  }
</script>

{#snippet subfolderLinksOnly(gid: number, subfolder: MumbleFolder)}
  <Menubar.Sub>
    <Menubar.SubTrigger class="w-full max-w-full min-w-0 cursor-default">
      <MumbleFolderIcon iconKey={subfolder.iconKey ?? null} class="size-3.5 shrink-0" />
      <span class="truncate">{subfolder.name}</span>
    </Menubar.SubTrigger>
    <Menubar.SubContent class="max-h-[min(18rem,85dvh)] overflow-y-auto overflow-x-hidden p-1" interactOutsideBehavior="ignore">
      {#each linksForFolder(gid, subfolder.id) as link (link.id)}
        <Menubar.Item onclick={() => openLink(link.id)}>{link.name}</Menubar.Item>
      {/each}
      {#if linksForFolder(gid, subfolder.id).length === 0}
        <Menubar.Item disabled>Empty folder</Menubar.Item>
      {/if}
      {#if nestedSubfoldersIgnored(gid, subfolder.id)}
        <Menubar.Item disabled>Nested folders: edit on Mumble Links page</Menubar.Item>
      {/if}
    </Menubar.SubContent>
  </Menubar.Sub>
{/snippet}

{#snippet rootFolderMenu(group: { id: number; name: string }, folder: MumbleFolder)}
  {@const rootLinks = linksForFolder(group.id, folder.id)}
  {@const childFolders = foldersForParent(group.id, folder.id)}
  <Menubar.Menu value="mumble-g{group.id}-f{folder.id}">
    <Menubar.Trigger
      class="border-input bg-secondary text-secondary-foreground hover:bg-muted aria-expanded:bg-muted mumble-folder-trigger min-h-6 leading-[1.2] max-w-full min-w-0 gap-1 rounded-md border px-1.5 py-0.5 text-xs font-medium shadow-xs"
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
        <div class="max-h-96 overflow-y-auto overflow-x-hidden px-1 py-1">
          {#each rootLinks as link (link.id)}
            <Menubar.Item onclick={() => openLink(link.id)}>{link.name}</Menubar.Item>
          {/each}
          {#if rootLinks.length === 0 && childFolders.length === 0}
            <Menubar.Item disabled>Empty folder</Menubar.Item>
          {/if}
        </div>
        {#each childFolders as sub (sub.id)}
          {@render subfolderLinksOnly(group.id, sub)}
        {/each}
      </div>
    </Menubar.Content>
  </Menubar.Menu>
{/snippet}

{#snippet rootLinksOnlyMenu(group: { id: number; name: string })}
  {@const rootLinks = linksForFolder(group.id, null)}
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
{/snippet}

<div
  bind:this={rootEl}
  class="mumble-chip-shell absolute z-0 box-border touch-none select-none pointer-events-auto rounded-md border border-border bg-card text-card-foreground flex flex-col justify-center shadow-[0_1px_2px_oklch(0_0_0/0.12),0_6px_18px_oklch(0_0_0/0.18)]"
  style:left="{frame.x}px"
  style:top="{frame.y}px"
  style:width="{Math.max(MIN_SHELL_WIDTH, frame.width)}px"
  style:height="{Math.max(COMPACT_SHELL_HEIGHT, frame.height)}px"
  role="application"
  aria-label="Mumble links"
>
  <div class="mumble-chip-row flex items-center gap-[3px] box-border p-[2px] pl-[4px] min-h-0 flex-1 min-w-0">
    <div
      class="mumble-chip-grip flex shrink-0 items-center justify-center w-[18px] h-[22px] rounded-[4px] text-muted-foreground cursor-grab select-none active:cursor-grabbing"
      role="presentation"
      title="Drag to move"
      onpointerdown={onGripPointerDown}
    >
      <GripVerticalIcon class="mumble-chip-grip-icon h-3 w-3" aria-hidden="true" />
    </div>

    <div class="mumble-chip-menu flex min-w-0 flex-1">
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
                {@render rootFolderMenu(group, folder)}
              {/each}
            {:else if rootLinks.length > 0}
              {@render rootLinksOnlyMenu(group)}
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
    </div>

    <button
      type="button"
      class="mumble-chip-pin flex shrink-0 items-center justify-center w-6 h-6 p-0 rounded-[4px] border border-border bg-background text-muted-foreground cursor-pointer hover:bg-accent hover:text-accent-foreground aria-pressed:text-primary aria-pressed:border-[color-mix(in_oklch,var(--primary)_45%,var(--border))] aria-pressed:[background:color-mix(in_oklch,var(--primary)_12%,var(--background))]"
      title={pinned ? "Unpin widget (hide when widgets are toggled off)" : "Pin widget (stay visible when widgets are toggled off)"}
      aria-label={pinned ? "Unpin widget" : "Pin widget"}
      aria-pressed={pinned}
      onclick={togglePinned}
      onpointerdown={stopDragChain}
    >
      <PinIcon class="mumble-chip-pin-icon h-3 w-3" strokeWidth={pinned ? 2.25 : 1.75} />
    </button>

    <button
      type="button"
      class="mumble-chip-resize-x"
      title="Drag to resize width"
      aria-label="Resize Mumble widget width"
      onpointerdown={onResizeWidthPointerDown}
    ></button>
  </div>
</div>

<style>
  /* Flatten default menubar chrome; one menu per root folder (or per-server fallback). */
  .mumble-chip-menu :global([data-slot="menubar"]) {
    height: auto;
    min-height: 0;
    min-width: 0;
    max-width: 100%;
    flex: 1;
    align-items: center;
    gap: 3px;
    border: none;
    background: transparent;
    padding: 0;
    box-shadow: none;
  }

  .mumble-chip-menu :global([data-slot="menubar-trigger"]) {
    max-width: 100%;
  }

  .mumble-chip-resize-x {
    flex-shrink: 0;
    width: 6px;
    align-self: stretch;
    margin: 2px 0;
    margin-right: 2px;
    padding: 0;
    border: none;
    border-radius: 3px;
    background: linear-gradient(
      90deg,
      transparent 0%,
      color-mix(in oklch, var(--muted-foreground) 35%, transparent) 35%,
      color-mix(in oklch, var(--muted-foreground) 35%, transparent) 65%,
      transparent 100%
    );
    cursor: ew-resize;
    touch-action: none;
  }

  .mumble-chip-resize-x:hover,
  .mumble-chip-resize-x:focus-visible {
    background: linear-gradient(
      90deg,
      transparent 0%,
      color-mix(in oklch, var(--primary) 50%, transparent) 40%,
      color-mix(in oklch, var(--primary) 50%, transparent) 60%,
      transparent 100%
    );
    outline: none;
  }
</style>
