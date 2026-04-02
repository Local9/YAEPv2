<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import type { MumbleFolder, MumbleLink, MumbleTreeSnapshot } from "$models/domain";
  import { deriveMumbleLinkName, isAllowedMumbleLinkUrl } from "$lib/utils/mumble-url";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { toast } from "svelte-sonner";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from "$lib/components/ui/card";
  import * as Collapsible from "$lib/components/ui/collapsible";
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import ChevronDownIcon from "@lucide/svelte/icons/chevron-down";
  import FolderIcon from "@lucide/svelte/icons/folder";
  import LinkIcon from "@lucide/svelte/icons/link";
  import MoreVerticalIcon from "@lucide/svelte/icons/more-vertical";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import RadioIcon from "@lucide/svelte/icons/radio";

  const MUMBLE_LINK_HOTKEY_CAPTURE = "mumbleLink";
  const HOTKEY_INPUT_CLASS = "min-w-[10rem] cursor-pointer select-none";
  const HOTKEY_CAPTURE_RING_CLASS = "ring-ring ring-2 ring-offset-2 ring-offset-background";

  interface HotkeyCapturedPayload {
    value: string;
    captureType: string;
    targetId: number | null;
  }

  let tree = $state<MumbleTreeSnapshot | null>(null);
  let status = $state("");
  let error = $state("");
  let captureHotkeyLinkId = $state<number | null>(null);
  let linkDraft = $state<{ serverGroupId: number; folderId: number | null } | null>(null);
  let linkDraftUrl = $state("");
  let linkDraftName = $state("");
  let folderDraft = $state<{ serverGroupId: number; parentFolderId: number | null } | null>(null);
  let folderDraftName = $state("");
  let confirmDelete = $state<{
    kind: "folder" | "link";
    id: number;
    label: string;
  } | null>(null);
  let confirmOpen = $state(false);
  /** Folder id -> expanded; omitted ids default to expanded (true). */
  let folderExpandedById = $state<Record<number, boolean>>({});

  function isFolderExpanded(folderId: number): boolean {
    return folderExpandedById[folderId] !== false;
  }

  function setFolderExpanded(folderId: number, expanded: boolean) {
    folderExpandedById = { ...folderExpandedById, [folderId]: expanded };
  }

  function userSafeMumbleErrorMessage(): string {
    return "Unable to save Mumble link changes right now. Please try again.";
  }

  function onEnterSubmit(
    e: KeyboardEvent,
    action: () => void | Promise<void>,
    options?: { stopBubble?: boolean }
  ) {
    if (e.key !== "Enter") return;
    e.preventDefault();
    if (options?.stopBubble) e.stopPropagation();
    void action();
  }

  async function refresh() {
    try {
      tree = await backend.getMumbleTree();
      error = "";
    } catch {
      error = userSafeMumbleErrorMessage();
      tree = null;
    }
  }

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

  function maxFolderOrder(gid: number, parentId: number | null): number {
    const list = foldersForParent(gid, parentId);
    return list.reduce((m, f) => Math.max(m, f.displayOrder), -1);
  }

  function maxLinkOrder(gid: number, folderId: number | null): number {
    const list = linksForFolder(gid, folderId);
    return list.reduce((m, l) => Math.max(m, l.displayOrder), -1);
  }

  function openDeleteFolder(f: MumbleFolder) {
    confirmDelete = { kind: "folder", id: f.id, label: f.name };
    confirmOpen = true;
  }

  function openDeleteLink(l: MumbleLink) {
    confirmDelete = { kind: "link", id: l.id, label: l.name };
    confirmOpen = true;
  }

  async function executeDelete() {
    const c = confirmDelete;
    if (!c) return;
    try {
      if (c.kind === "folder") await backend.deleteMumbleFolder(c.id);
      else await backend.deleteMumbleLink(c.id);
      status = "Deleted";
      confirmOpen = false;
      confirmDelete = null;
      await refresh();
    } catch {
      error = userSafeMumbleErrorMessage();
    }
  }

  async function saveFolder(f: MumbleFolder, opts?: { silent?: boolean }) {
    try {
      await backend.updateMumbleFolder(f.id, f.name.trim(), f.displayOrder);
      if (!opts?.silent) status = "Folder saved";
      await refresh();
    } catch {
      error = userSafeMumbleErrorMessage();
    }
  }

  async function saveLink(link: MumbleLink, opts?: { silent?: boolean }) {
    const url = link.url.trim();
    if (!isAllowedMumbleLinkUrl(url)) {
      error = "Link URL must start with mumble:// or https://";
      return;
    }
    try {
      await backend.updateMumbleLink(
        link.id,
        link.name.trim(),
        url,
        link.displayOrder,
        link.hotkey.trim(),
        link.serverGroupId,
        link.folderId ?? null
      );
      if (!opts?.silent) status = "Link saved";
      await refresh();
    } catch {
      error = userSafeMumbleErrorMessage();
    }
  }

  function beginRootFolderDraft(gid: number) {
    folderDraft = { serverGroupId: gid, parentFolderId: null };
    folderDraftName = "";
  }

  function beginSubfolderDraft(gid: number, parentFolderId: number) {
    folderDraft = { serverGroupId: gid, parentFolderId };
    folderDraftName = "";
  }

  function cancelFolderDraft() {
    folderDraft = null;
    folderDraftName = "";
  }

  async function submitFolderDraft() {
    if (!folderDraft || !folderDraftName.trim()) return;
    const ord = maxFolderOrder(folderDraft.serverGroupId, folderDraft.parentFolderId) + 1;
    try {
      await backend.createMumbleFolder(
        folderDraft.serverGroupId,
        folderDraft.parentFolderId,
        folderDraftName.trim(),
        ord
      );
      status = "Folder created";
      cancelFolderDraft();
      await refresh();
    } catch {
      error = userSafeMumbleErrorMessage();
    }
  }

  function beginLinkDraft(gid: number, folderId: number | null) {
    linkDraft = { serverGroupId: gid, folderId };
    linkDraftUrl = "";
    linkDraftName = "";
  }

  function cancelLinkDraft() {
    linkDraft = null;
    linkDraftUrl = "";
    linkDraftName = "";
  }

  function onLinkDraftUrlInput(v: string) {
    linkDraftUrl = v;
    const d = deriveMumbleLinkName(v);
    if (d) linkDraftName = d;
  }

  async function submitLinkDraft() {
    if (!linkDraft) return;
    const url = linkDraftUrl.trim();
    const name = linkDraftName.trim();
    if (!name || !isAllowedMumbleLinkUrl(url)) {
      error = "Enter a valid URL and name";
      return;
    }
    const ord = maxLinkOrder(linkDraft.serverGroupId, linkDraft.folderId) + 1;
    try {
      await backend.createMumbleLink(name, url, ord, "", linkDraft.serverGroupId, linkDraft.folderId);
      status = "Link created";
      cancelLinkDraft();
      await refresh();
    } catch {
      error = userSafeMumbleErrorMessage();
    }
  }

  function stopLinkHotkeyCapture() {
    captureHotkeyLinkId = null;
    void backend.hotkeysCaptureStop();
  }

  async function startLinkHotkeyCapture(linkId: number) {
    captureHotkeyLinkId = linkId;
    error = "";
    try {
      await backend.hotkeysCaptureStart(MUMBLE_LINK_HOTKEY_CAPTURE, linkId);
    } catch {
      error = userSafeMumbleErrorMessage();
      captureHotkeyLinkId = null;
    }
  }

  onMount(() => {
    void refresh();
    let unlistenCaptured: UnlistenFn | undefined;
    void listen<HotkeyCapturedPayload>("hotkeyCaptured", (event) => {
      const p = event.payload;
      if (p.captureType !== MUMBLE_LINK_HOTKEY_CAPTURE || p.targetId == null) return;
      captureHotkeyLinkId = null;
      const lid = p.targetId;
      const link = tree?.links.find((l) => l.id === lid);
      if (!link) {
        void refresh();
        return;
      }
      void (async () => {
        try {
          await backend.updateMumbleLink(
            lid,
            link.name.trim(),
            link.url.trim(),
            link.displayOrder,
            p.value.trim(),
            link.serverGroupId,
            link.folderId ?? null
          );
          status = "Hotkey saved";
          await refresh();
        } catch {
          error = userSafeMumbleErrorMessage();
        }
      })();
    }).then((u) => {
      unlistenCaptured = u;
    });
    return () => {
      unlistenCaptured?.();
      stopLinkHotkeyCapture();
    };
  });

  $effect(() => {
    if (status) toast.success(status);
  });

  $effect(() => {
    if (error) toast.error(error);
  });
</script>

{#snippet linkDraftPanel(gid: number, fid: number | null)}
  {#if linkDraft && linkDraft.serverGroupId === gid && linkDraft.folderId === fid}
    <div
      class="border-border bg-muted/30 mt-2 flex flex-col gap-2 rounded-md border p-3"
      style:margin-left={`${(fid ? 1 : 0) * 12 + 8}px`}
    >
      <p class="text-muted-foreground text-xs font-medium">New link</p>
      <Input
        class="font-mono text-sm"
        placeholder="mumble://..."
        value={linkDraftUrl}
        oninput={(e) => onLinkDraftUrlInput((e.currentTarget as HTMLInputElement).value)}
        onkeydown={(e) => onEnterSubmit(e, () => void submitLinkDraft())}
      />
      <Input
        bind:value={linkDraftName}
        placeholder="Display name"
        onkeydown={(e) => onEnterSubmit(e, () => void submitLinkDraft())}
      />
      <div class="flex flex-wrap gap-2">
        <Button type="button" size="sm" onclick={() => void submitLinkDraft()}>Add link</Button>
        <Button type="button" variant="outline" size="sm" onclick={cancelLinkDraft}>Cancel</Button>
      </div>
    </div>
  {/if}
{/snippet}

{#snippet folderDraftPanel(gid: number, parentId: number | null, depth: number)}
  {#if folderDraft && folderDraft.serverGroupId === gid && folderDraft.parentFolderId === parentId}
    <div
      class="border-border bg-muted/30 mt-2 flex flex-col gap-2 rounded-md border p-3"
      style:margin-left={`${depth * 16 + 8}px`}
    >
      <p class="text-muted-foreground text-xs font-medium">New folder</p>
      <Input
        bind:value={folderDraftName}
        placeholder="Folder name"
        onkeydown={(e) => onEnterSubmit(e, () => void submitFolderDraft())}
      />
      <div class="flex flex-wrap gap-2">
        <Button type="button" size="sm" onclick={() => void submitFolderDraft()}>Add folder</Button>
        <Button type="button" variant="outline" size="sm" onclick={cancelFolderDraft}>Cancel</Button>
      </div>
    </div>
  {/if}
{/snippet}

{#snippet linkRow(link: MumbleLink, depth: number)}
  <div
    class="border-border flex flex-col gap-2 border-b py-2 sm:flex-row sm:flex-wrap sm:items-center sm:gap-2"
    style:margin-left={`${depth * 16}px`}
  >
    <LinkIcon class="text-muted-foreground size-4 shrink-0 sm:mt-0.5" aria-hidden="true" />
    <div class="flex min-w-0 flex-1 flex-wrap items-center gap-2">
      <Input
        class="min-w-32 flex-1 sm:max-w-xs"
        bind:value={link.name}
        onblur={() => void saveLink(link, { silent: true })}
        onkeydown={(e) => onEnterSubmit(e, () => void saveLink(link, { silent: true }))}
      />
      <Input
        class="min-w-48 flex-[2] font-mono text-sm"
        bind:value={link.url}
        onblur={() => void saveLink(link, { silent: true })}
        onkeydown={(e) => onEnterSubmit(e, () => void saveLink(link, { silent: true }))}
      />
      <Input
        class="{HOTKEY_INPUT_CLASS} w-36 {captureHotkeyLinkId === link.id
          ? HOTKEY_CAPTURE_RING_CLASS
          : ''}"
        readonly
        value={link.hotkey}
        placeholder="Hotkey"
        onpointerdown={() => void startLinkHotkeyCapture(link.id)}
      />
      <Input
        class="w-20 shrink-0"
        type="number"
        bind:value={link.displayOrder}
        onblur={() => void saveLink(link, { silent: true })}
        onkeydown={(e) => onEnterSubmit(e, () => void saveLink(link, { silent: true }))}
      />
    </div>
    <div class="ml-auto flex shrink-0 items-center self-end sm:self-center">
      <DropdownMenu.Root>
        <DropdownMenu.Trigger>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            class="text-muted-foreground size-8"
            aria-label="Link actions"
          >
            <MoreVerticalIcon class="size-4" aria-hidden="true" />
          </Button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Content align="end">
          <DropdownMenu.Item variant="destructive" onclick={() => openDeleteLink(link)}>
            Delete link
          </DropdownMenu.Item>
        </DropdownMenu.Content>
      </DropdownMenu.Root>
    </div>
  </div>
{/snippet}

{#snippet folderBlock(folder: MumbleFolder, depth: number)}
  {@const children = foldersForParent(folder.serverGroupId, folder.id)}
  {@const linksHere = linksForFolder(folder.serverGroupId, folder.id)}
  <div class="min-w-0" style:margin-left={`${depth * 16}px`}>
  <Collapsible.Root
    class="mt-2 min-w-0 pl-2"
    open={isFolderExpanded(folder.id)}
    onOpenChange={(expanded: boolean) => setFolderExpanded(folder.id, expanded)}
  >
    <div
      class="hover:bg-muted/50 flex w-full min-w-0 items-center gap-2 rounded-md px-1 py-1"
    >
      <div class="flex min-w-0 flex-1 flex-wrap items-center gap-2">
        <Collapsible.Trigger
          class="text-muted-foreground hover:bg-muted data-[state=open]:[&_svg]:rotate-180 inline-flex size-8 shrink-0 items-center justify-center rounded-md bg-transparent"
          aria-label={isFolderExpanded(folder.id) ? "Collapse folder" : "Expand folder"}
          type="button"
        >
          <ChevronDownIcon class="size-4 shrink-0 transition-transform duration-200" />
        </Collapsible.Trigger>
        <FolderIcon class="text-muted-foreground size-4 shrink-0" aria-hidden="true" />
        <Input
          class="h-8 min-w-0 max-w-xs flex-1"
          bind:value={folder.name}
          onblur={() => void saveFolder(folder, { silent: true })}
          onkeydown={(e) => onEnterSubmit(e, () => void saveFolder(folder, { silent: true }))}
        />
        <span class="text-muted-foreground text-xs">order</span>
        <Input
          class="h-8 w-16 shrink-0"
          type="number"
          bind:value={folder.displayOrder}
          onblur={() => void saveFolder(folder, { silent: true })}
          onkeydown={(e) => onEnterSubmit(e, () => void saveFolder(folder, { silent: true }))}
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
          <DropdownMenu.Content align="start" class="w-44">
            <DropdownMenu.Item
              onclick={() => beginSubfolderDraft(folder.serverGroupId, folder.id)}
            >
              <PlusIcon class="mr-2 size-4" aria-hidden="true" />
              Add subfolder
            </DropdownMenu.Item>
            <DropdownMenu.Item onclick={() => beginLinkDraft(folder.serverGroupId, folder.id)}>
              <LinkIcon class="mr-2 size-4" aria-hidden="true" />
              Add link
            </DropdownMenu.Item>
            <DropdownMenu.Separator />
            <DropdownMenu.Item variant="destructive" onclick={() => openDeleteFolder(folder)}>
              Delete folder
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Root>
      </div>
    </div>
    <Collapsible.Content class="min-w-0 pl-1 pt-1">
      {@render folderDraftPanel(folder.serverGroupId, folder.id, depth + 1)}
      {@render linkDraftPanel(folder.serverGroupId, folder.id)}
      {#each linksHere as link (link.id)}
        {@render linkRow(link, depth + 1)}
      {/each}
      {#each children as sub (sub.id)}
        {@render folderBlock(sub, depth + 1)}
      {/each}
    </Collapsible.Content>
  </Collapsible.Root>
  </div>
{/snippet}

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <RadioIcon class="text-muted-foreground mt-0.5 size-5 shrink-0" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Mumble Links</CardTitle>
        <CardDescription
          >Organize folders and links. Paste a URL to fill the name. Hotkeys are optional and can be set per
          link.</CardDescription
        >
      </div>
    </div>
  </CardHeader>
  <CardContent>
    {#if tree}
      <div class="mt-2 space-y-6">
        {#each sortedGroups as group (group.id)}
          <Collapsible.Root class="border-border rounded-lg border" open>
            <Collapsible.Trigger
              class="hover:bg-muted/40 flex w-full items-center gap-2 px-3 py-2 text-left"
            >
              <ChevronDownIcon
                class="size-4 shrink-0 transition-transform [[data-state=open]_&]:rotate-180"
              />
              {#if multipleServerGroups}
                <span class="text-foreground min-w-0 flex-1 font-medium">{group.name}</span>
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
                    <DropdownMenu.Item onclick={() => beginRootFolderDraft(group.id)}>
                      <PlusIcon class="mr-2 size-4" aria-hidden="true" />
                      Add folder
                    </DropdownMenu.Item>
                    <DropdownMenu.Item onclick={() => beginLinkDraft(group.id, null)}>
                      <LinkIcon class="mr-2 size-4" aria-hidden="true" />
                      Add link
                    </DropdownMenu.Item>
                  </DropdownMenu.Content>
                </DropdownMenu.Root>
              </div>
            </Collapsible.Trigger>
            <Collapsible.Content class="px-3 py-3">
              {@render folderDraftPanel(group.id, null, 0)}
              {@render linkDraftPanel(group.id, null)}
              {#each linksForFolder(group.id, null) as link (link.id)}
                {@render linkRow(link, 0)}
              {/each}
              {#each foldersForParent(group.id, null) as folder (folder.id)}
                {@render folderBlock(folder, 0)}
              {/each}
            </Collapsible.Content>
          </Collapsible.Root>
        {/each}
      </div>
    {:else}
      <p class="text-muted-foreground mt-4 text-sm">Loading tree…</p>
    {/if}
  </CardContent>
</Card>

<AlertDialog.Root bind:open={confirmOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Delete {confirmDelete?.label ?? ""}?</AlertDialog.Title>
      <AlertDialog.Description>
        {#if confirmDelete?.kind === "folder"}
          This removes the folder, its subfolders, and all links inside. This cannot be undone.
        {:else}
          This removes the link. This cannot be undone.
        {/if}
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel
        onclick={() => {
          confirmOpen = false;
          confirmDelete = null;
        }}
      >
        Cancel
      </AlertDialog.Cancel>
      <AlertDialog.Action variant="destructive" onclick={() => void executeDelete()}>Delete</AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
