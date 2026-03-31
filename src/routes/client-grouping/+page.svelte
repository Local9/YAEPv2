<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import type { ClientGroupDetail, Profile, ThumbnailSetting } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import * as Collapsible from "$lib/components/ui/collapsible";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import ArrowRightIcon from "@lucide/svelte/icons/arrow-right";
  import CheckCircle2Icon from "@lucide/svelte/icons/check-circle-2";
  import GripVerticalIcon from "@lucide/svelte/icons/grip-vertical";
  import LayersIcon from "@lucide/svelte/icons/layers";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";

  const CLIENT_GROUP_FORWARD_CAPTURE = "clientGroupCycleForward";
  const CLIENT_GROUP_BACKWARD_CAPTURE = "clientGroupCycleBackward";

  const GROUP_HOTKEY_CAPTURE_FIELD = {
    [CLIENT_GROUP_FORWARD_CAPTURE]: "cycleForwardHotkey",
    [CLIENT_GROUP_BACKWARD_CAPTURE]: "cycleBackwardHotkey",
  } as const;

  type GroupHotkeyCaptureKind =
    | typeof CLIENT_GROUP_FORWARD_CAPTURE
    | typeof CLIENT_GROUP_BACKWARD_CAPTURE;

  const CYCLE_HOTKEY_INPUT_CLASS = "min-w-[10rem] cursor-pointer select-none";
  const CYCLE_HOTKEY_CAPTURE_RING_CLASS =
    "ring-ring ring-2 ring-offset-2 ring-offset-background";

  interface HotkeyCapturedPayload {
    value: string;
    captureType: string;
    targetId: number | null;
  }

  let profiles = $state<Profile[]>([]);
  let activeProfileId = $state<number | null>(null);
  let groups = $state<ClientGroupDetail[]>([]);
  let thumbnailSettings = $state<ThumbnailSetting[]>([]);
  let status = $state("");
  let error = $state("");
  let newGroupName = $state("");
  let createGroupDialogOpen = $state(false);

  let dragGroupId = $state<number | null>(null);
  let dragTitle = $state<string | null>(null);
  let dropBeforeIndex = $state<number | null>(null);
  /** Which group is capturing a cycle hotkey (forward or backward), if any. */
  let captureHotkey = $state<{ groupId: number; kind: GroupHotkeyCaptureKind } | null>(null);

  function orderedMemberTitles(g: ClientGroupDetail): string[] {
    return [...g.members]
      .sort(
        (a, b) =>
          a.displayOrder - b.displayOrder || a.windowTitle.localeCompare(b.windowTitle),
      )
      .map((m) => m.windowTitle);
  }

  function availableToAdd(g: ClientGroupDetail): string[] {
    const inGroup = new Set(g.members.map((m) => m.windowTitle));
    return thumbnailSettings.map((t) => t.windowTitle).filter((t) => !inGroup.has(t));
  }

  async function refresh() {
    profiles = await backend.getProfiles();
    const active = profiles.find((p) => p.isActive);
    if (!active) {
      groups = [];
      thumbnailSettings = [];
      activeProfileId = null;
      return;
    }
    activeProfileId = active.id;
    try {
      [groups, thumbnailSettings] = await Promise.all([
        backend.getClientGroupsDetailed(active.id),
        backend.getThumbnailSettings(active.id),
      ]);
    } catch (e) {
      error = String(e);
      groups = [];
      thumbnailSettings = [];
    }
  }

  async function saveHotkeys(group: ClientGroupDetail) {
    if (activeProfileId == null) return;
    try {
      await backend.updateClientGroupHotkeys(
        group.id,
        group.cycleForwardHotkey,
        group.cycleBackwardHotkey,
      );
      status = `Saved hotkeys for ${group.name}`;
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  function stopHotkeyCapture() {
    captureHotkey = null;
    void backend.hotkeysCaptureStop();
  }

  async function onGroupCycleHotkeyPointerDown(
    group: ClientGroupDetail,
    kind: GroupHotkeyCaptureKind,
  ) {
    captureHotkey = { groupId: group.id, kind };
    error = "";
    try {
      await backend.hotkeysCaptureStart(kind, group.id);
    } catch (e) {
      error = String(e);
      captureHotkey = null;
    }
  }

  function isCapturingHotkey(groupId: number, kind: GroupHotkeyCaptureKind): boolean {
    return captureHotkey?.groupId === groupId && captureHotkey?.kind === kind;
  }

  async function cycle(group: ClientGroupDetail, direction: "forward" | "backward") {
    try {
      await backend.cycleClientGroup(group.id, direction);
      status = `Cycled ${group.name} (${direction})`;
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function createGroup() {
    if (activeProfileId == null) return;
    const name = newGroupName.trim();
    if (!name) {
      error = "Enter a group name";
      return;
    }
    try {
      await backend.createClientGroup(activeProfileId, name);
      newGroupName = "";
      createGroupDialogOpen = false;
      status = `Created group ${name}`;
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function removeGroup(group: ClientGroupDetail) {
    if (activeProfileId == null) return;
    if (
      !confirm(
        `Delete group "${group.name}"? Clients are removed from the group only; thumbnails are not deleted.`,
      )
    ) {
      return;
    }
    try {
      await backend.deleteClientGroup(activeProfileId, group.id);
      status = `Deleted group ${group.name}`;
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function addMember(group: ClientGroupDetail, windowTitle: string) {
    if (activeProfileId == null || !windowTitle.trim()) return;
    try {
      await backend.addClientGroupMember(activeProfileId, group.id, windowTitle);
      status = `Added client to ${group.name}`;
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function removeMember(group: ClientGroupDetail, windowTitle: string) {
    if (activeProfileId == null) return;
    try {
      await backend.removeClientGroupMember(activeProfileId, group.id, windowTitle);
      status = `Removed client from ${group.name}`;
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  function reorderTitles(list: string[], fromIndex: number, toBeforeIndex: number): string[] {
    const next = [...list];
    const [item] = next.splice(fromIndex, 1);
    let dest = toBeforeIndex;
    if (fromIndex < dest) dest -= 1;
    next.splice(dest, 0, item);
    return next;
  }

  async function onMemberDrop(group: ClientGroupDetail, e?: DragEvent) {
    const titleFromTransfer = e?.dataTransfer?.getData("text/plain").trim() ?? "";
    const effectiveTitle = titleFromTransfer || dragTitle || "";
    if (activeProfileId == null || !effectiveTitle || dragGroupId !== group.id) {
      clearDragState();
      return;
    }
    const titles = orderedMemberTitles(group);
    const fromIndex = titles.indexOf(effectiveTitle);
    const beforeIdx = dropBeforeIndex;
    clearDragState();
    if (fromIndex < 0 || beforeIdx == null) return;
    const next = reorderTitles(titles, fromIndex, beforeIdx);
    if (next.join("\0") === titles.join("\0")) return;
    try {
      await backend.reorderClientGroupMembers(activeProfileId, group.id, next);
      status = `Updated order for ${group.name}`;
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  function onRowDragEnter(e: DragEvent) {
    e.preventDefault();
  }

  function onRowDragOver(e: DragEvent, group: ClientGroupDetail, rowIndex: number) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    if (dragGroupId !== group.id) return;
    const row = e.currentTarget as HTMLElement;
    const rect = row.getBoundingClientRect();
    const mid = rect.top + rect.height / 2;
    dropBeforeIndex = e.clientY < mid ? rowIndex : rowIndex + 1;
  }

  function onListDragEnter(e: DragEvent) {
    e.preventDefault();
  }

  function onListDragOver(e: DragEvent, group: ClientGroupDetail, memberCount: number) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    if (dragGroupId !== group.id) return;
    if (memberCount === 0) {
      dropBeforeIndex = 0;
      return;
    }
    const el = e.currentTarget as HTMLElement;
    const rect = el.getBoundingClientRect();
    if (e.clientY >= rect.bottom - 8) {
      dropBeforeIndex = memberCount;
    }
  }

  function clearDragState() {
    dragGroupId = null;
    dragTitle = null;
    dropBeforeIndex = null;
  }

  function onRowDragEnd() {
    requestAnimationFrame(() => clearDragState());
  }

  onMount(() => {
    void refresh();
    let unlistenCaptured: UnlistenFn | undefined;
    void listen<HotkeyCapturedPayload>("hotkeyCaptured", (event) => {
      const p = event.payload;
      if (p.targetId == null) return;
      const field =
        GROUP_HOTKEY_CAPTURE_FIELD[p.captureType as keyof typeof GROUP_HOTKEY_CAPTURE_FIELD];
      if (!field) return;
      const gid = p.targetId;
      captureHotkey = null;
      groups = groups.map((gr) => (gr.id === gid ? { ...gr, [field]: p.value } : gr));
      const updated = groups.find((g) => g.id === gid);
      if (updated) void saveHotkeys(updated);
    }).then((u) => {
      unlistenCaptured = u;
    });
    return () => {
      unlistenCaptured?.();
      stopHotkeyCapture();
    };
  });
</script>

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <LayersIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Client Grouping</CardTitle>
        <CardDescription>
          Organize thumbnail clients into groups. Order in each group defines next / previous hotkey
          cycling. Clients are matched by window title (same as thumbnails). Registered cycle hotkeys run only when the focused window is one of the clients currently
          tracked by the thumbnail service (same PIDs as live thumbnails); the Cycle next / prev
          buttons always work from this page.
        </CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
    {#if status}
      <Alert class="border-primary/30 bg-primary/5">
        <CheckCircle2Icon class="size-4 text-primary" aria-hidden="true" />
        <AlertTitle>Status</AlertTitle>
        <AlertDescription>{status}</AlertDescription>
      </Alert>
    {/if}
    {#if error}
      <Alert variant="destructive">
        <AlertCircleIcon class="size-4 shrink-0" aria-hidden="true" />
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    {/if}

    {#if activeProfileId == null}
      <p class="text-muted-foreground mt-4 text-sm">Select an active profile to manage client groups.</p>
    {:else}
      <div class="mt-4 flex flex-wrap items-center gap-2 border-b border-border pb-4">
        <Button type="button" onclick={() => (createGroupDialogOpen = true)}>
          <PlusIcon class="size-4 shrink-0" aria-hidden="true" />
          Create group
        </Button>
      </div>

      <Dialog.Root
        bind:open={createGroupDialogOpen}
        onOpenChange={(open) => {
          if (!open) newGroupName = "";
        }}
      >
        <Dialog.Content class="sm:max-w-md">
          <Dialog.Header>
            <Dialog.Title>Create group</Dialog.Title>
            <Dialog.Description>
              Enter a name for the new client group. Clients can be added after it is created.
            </Dialog.Description>
          </Dialog.Header>
          <div class="grid gap-2">
            <label class="text-muted-foreground text-xs font-medium" for="new-group-name-dialog">
              Group name
            </label>
            <Input
              id="new-group-name-dialog"
              bind:value={newGroupName}
              placeholder="Group name"
              onkeydown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  void createGroup();
                }
              }}
            />
          </div>
          <Dialog.Footer>
            <Dialog.Close>
              {#snippet child({ props })}
                <Button variant="outline" {...props}>Cancel</Button>
              {/snippet}
            </Dialog.Close>
            <Button type="button" onclick={() => void createGroup()}>
              <PlusIcon class="size-4 shrink-0" aria-hidden="true" />
              Create
            </Button>
          </Dialog.Footer>
        </Dialog.Content>
      </Dialog.Root>

      <p class="text-muted-foreground mt-2 text-sm">
        Active profile:
        <span class="text-foreground font-medium">{profiles.find((p) => p.isActive)?.name ?? ""}</span>
      </p>

      <p class="text-muted-foreground mt-3 text-xs">
        Add clients from current thumbnail window titles. Drag by the grip to reorder; the target row shows a top
        or bottom border where the item will land. Cycle hotkeys follow this order.
      </p>

      {#if groups.length === 0}
        <p class="text-muted-foreground mt-4 text-sm">No client groups for this profile.</p>
      {/if}

      <div class="mt-4 flex flex-col gap-3">
        {#each groups as group (group.id)}
          <Collapsible.Root class="border-border bg-card w-full rounded-lg border shadow-xs">
            <div class="flex flex-wrap items-center gap-2 px-3 py-2">
              <Collapsible.Trigger
                class="text-primary hover:bg-muted/60 rounded-md px-2 py-1 text-left text-sm font-medium underline-offset-4 hover:underline"
              >
                {group.name}
                <span class="text-muted-foreground font-normal">
                  ({group.members.length} client{group.members.length === 1 ? "" : "s"})
                </span>
              </Collapsible.Trigger>
            </div>
            <Collapsible.Content class="border-border/80 border-t px-3 pb-3 pt-2">
                      <div class="mb-3 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                        <div>
                          <span class="text-muted-foreground mb-1 block text-xs">Forward hotkey</span>
                          <Input
                            class="{CYCLE_HOTKEY_INPUT_CLASS} {isCapturingHotkey(group.id, CLIENT_GROUP_FORWARD_CAPTURE)
                              ? CYCLE_HOTKEY_CAPTURE_RING_CLASS
                              : ''}"
                            readonly
                            autocomplete="off"
                            spellcheck={false}
                            inputmode="none"
                            title="Click the field, then press the shortcut. Typing is disabled; keys are captured by the app."
                            aria-readonly="true"
                            bind:value={group.cycleForwardHotkey}
                            placeholder={isCapturingHotkey(group.id, CLIENT_GROUP_FORWARD_CAPTURE)
                              ? "Press chord, release key…"
                              : "Click here, then press keys"}
                            onpointerdown={() =>
                              void onGroupCycleHotkeyPointerDown(group, CLIENT_GROUP_FORWARD_CAPTURE)}
                            onpaste={(e) => e.preventDefault()}
                            onblur={() => {
                              if (isCapturingHotkey(group.id, CLIENT_GROUP_FORWARD_CAPTURE)) {
                                stopHotkeyCapture();
                              }
                              void saveHotkeys(group);
                            }}
                          />
                        </div>
                        <div>
                          <span class="text-muted-foreground mb-1 block text-xs">Backward hotkey</span>
                          <Input
                            class="{CYCLE_HOTKEY_INPUT_CLASS} {isCapturingHotkey(group.id, CLIENT_GROUP_BACKWARD_CAPTURE)
                              ? CYCLE_HOTKEY_CAPTURE_RING_CLASS
                              : ''}"
                            readonly
                            autocomplete="off"
                            spellcheck={false}
                            inputmode="none"
                            title="Click the field, then press the shortcut. Typing is disabled; keys are captured by the app."
                            aria-readonly="true"
                            bind:value={group.cycleBackwardHotkey}
                            placeholder={isCapturingHotkey(group.id, CLIENT_GROUP_BACKWARD_CAPTURE)
                              ? "Press chord, release key…"
                              : "Click here, then press keys"}
                            onpointerdown={() =>
                              void onGroupCycleHotkeyPointerDown(group, CLIENT_GROUP_BACKWARD_CAPTURE)}
                            onpaste={(e) => e.preventDefault()}
                            onblur={() => {
                              if (isCapturingHotkey(group.id, CLIENT_GROUP_BACKWARD_CAPTURE)) {
                                stopHotkeyCapture();
                              }
                              void saveHotkeys(group);
                            }}
                          />
                        </div>
                        <div class="flex flex-wrap items-end gap-2">
                          <Button type="button" variant="outline" onclick={() => void cycle(group, "forward")}>
                            <ArrowRightIcon class="size-4 shrink-0" aria-hidden="true" />
                            Cycle next
                          </Button>
                          <Button type="button" variant="outline" onclick={() => void cycle(group, "backward")}>
                            <ArrowLeftIcon class="size-4 shrink-0" aria-hidden="true" />
                            Cycle prev
                          </Button>
                          <Button
                            type="button"
                            variant="destructive"
                            class="ml-auto"
                            onclick={() => void removeGroup(group)}
                          >
                            <Trash2Icon class="size-4 shrink-0" aria-hidden="true" />
                            Delete group
                          </Button>
                        </div>
                      </div>

                      <div class="mb-2">
                        <label class="text-muted-foreground mb-1 block text-xs" for="add-client-{group.id}">
                          Add client to group
                        </label>
                        {#if availableToAdd(group).length === 0}
                          <p class="text-muted-foreground text-xs">
                            All thumbnail clients are already in this group, or there are no thumbnails yet.
                          </p>
                        {:else}
                          <select
                            id="add-client-{group.id}"
                            class="border-input bg-background h-9 w-full max-w-xl rounded-md border px-3 text-sm shadow-xs outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
                            onchange={(e) => {
                              const v = (e.currentTarget as HTMLSelectElement).value;
                              (e.currentTarget as HTMLSelectElement).value = "";
                              if (v) void addMember(group, v);
                            }}
                          >
                            <option value="">Choose a window title...</option>
                            {#each availableToAdd(group) as title (title)}
                              <option value={title}>{title}</option>
                            {/each}
                          </select>
                        {/if}
                      </div>

                      {#if group.members.length === 0}
                        <p class="text-muted-foreground text-sm">No clients in this group yet.</p>
                      {:else}
                        {@const memberTitles = orderedMemberTitles(group)}
                        <div
                          class="bg-muted/20 max-w-3xl rounded-lg border border-dashed border-border p-2"
                          role="list"
                          ondragenter={onListDragEnter}
                          ondragover={(e) => onListDragOver(e, group, group.members.length)}
                          ondrop={(e) => {
                            e.preventDefault();
                            void onMemberDrop(group, e);
                          }}
                        >
                          {#each memberTitles as title, i (title)}
                            <div
                              role="listitem"
                              class="hover:bg-muted/40 flex items-center gap-2 rounded-md border border-transparent px-2 py-2 transition-[border-color] {dragGroupId ===
                                group.id && dragTitle === title
                                ? 'opacity-50'
                                : ''} {dragGroupId === group.id && dropBeforeIndex === i
                                ? 'border-t-primary border-t-2'
                                : ''} {dragGroupId === group.id &&
                              dropBeforeIndex === memberTitles.length &&
                              i === memberTitles.length - 1
                                ? 'border-b-primary border-b-2'
                                : ''}"
                              ondragenter={onRowDragEnter}
                              ondragover={(e) => onRowDragOver(e, group, i)}
                              ondrop={(e) => {
                                e.preventDefault();
                                e.stopPropagation();
                                void onMemberDrop(group, e);
                              }}
                            >
                              <span
                                tabindex="-1"
                                role="button"
                                aria-grabbed={dragGroupId === group.id && dragTitle === title}
                                class="text-muted-foreground inline-flex size-6 shrink-0 cursor-grab touch-none select-none active:cursor-grabbing"
                                draggable="true"
                                ondragstart={(e) => {
                                  e.stopPropagation();
                                  dragGroupId = group.id;
                                  dragTitle = title;
                                  dropBeforeIndex = i;
                                  const dt = e.dataTransfer;
                                  if (dt) {
                                    dt.setData("text/plain", title);
                                    dt.effectAllowed = "move";
                                  }
                                }}
                                ondragend={onRowDragEnd}
                              >
                                <GripVerticalIcon class="size-4" aria-hidden="true" />
                              </span>
                              <span class="min-w-0 flex-1 truncate text-sm" title={title}>{title}</span>
                              <Button
                                type="button"
                                variant="ghost"
                                size="sm"
                                draggable={false}
                                class="shrink-0 text-destructive hover:text-destructive"
                                onclick={() => void removeMember(group, title)}
                              >
                                Remove
                              </Button>
                            </div>
                          {/each}
                        </div>
                      {/if}
            </Collapsible.Content>
          </Collapsible.Root>
        {/each}
      </div>
    {/if}
  </CardContent>
</Card>
