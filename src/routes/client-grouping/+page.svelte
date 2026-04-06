<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import type { ClientGroupDetail, Profile, ThumbnailSetting } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { toast } from "svelte-sonner";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import LayersIcon from "@lucide/svelte/icons/layers";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import { availableToAdd, orderedMemberTitles, reorderTitles } from "$lib/client-grouping/client-grouping-helpers";
  import {
    GROUP_HOTKEY_CAPTURE_FIELD,
    type GroupHotkeyCaptureKind,
  } from "$lib/components/client-grouping/client-group-hotkeys";
  import ClientGroupCreateDialog from "$lib/components/client-grouping/client-group-create-dialog.svelte";
  import ClientGroupCard from "$lib/components/client-grouping/client-group-card.svelte";

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
  let reorderListEl: HTMLElement | null = null;
  let reorderGroup: ClientGroupDetail | null = null;
  let reorderPointerId: number | null = null;
  let captureHotkey = $state<{ groupId: number; kind: GroupHotkeyCaptureKind } | null>(null);

  const availableToAddForGroup = (g: ClientGroupDetail) => availableToAdd(g, thumbnailSettings);
  const diagClientGrouping = (message: string): void => {
    void backend.frontendDiagLog("info", "client-grouping", message).catch(() => {});
  };

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

  function onSaveHotkeysBlur(group: ClientGroupDetail, kind: GroupHotkeyCaptureKind) {
    if (isCapturingHotkey(group.id, kind)) {
      stopHotkeyCapture();
      return;
    }
    void saveHotkeys(group);
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

  function clearMemberReorderState() {
    reorderListEl = null;
    reorderGroup = null;
    reorderPointerId = null;
    dragGroupId = null;
    dragTitle = null;
    dropBeforeIndex = null;
  }

  function updateDropIndexFromPointer(clientY: number) {
    if (reorderListEl == null || reorderGroup == null || dragTitle == null) return;
    const titles = orderedMemberTitles(reorderGroup);
    const fromIndex = titles.indexOf(dragTitle);
    if (fromIndex < 0) return;
    const memberCount = titles.length;
    const rows = [...reorderListEl.querySelectorAll<HTMLElement>("[data-reorder-row]")];
    const indexed = rows
      .map((el) => ({
        el,
        i: Number.parseInt(el.dataset.reorderRow ?? "", 10),
      }))
      .filter((x) => !Number.isNaN(x.i) && x.i !== fromIndex)
      .sort((a, b) => a.i - b.i);

    if (indexed.length > 0) {
      const firstTop = indexed[0].el.getBoundingClientRect().top;
      if (clientY < firstTop) {
        dropBeforeIndex = 0;
        return;
      }
    }

    for (const { el, i } of indexed) {
      const rect = el.getBoundingClientRect();
      if (rect.height < 2) continue;
      const mid = rect.top + rect.height / 2;
      if (clientY < mid) {
        dropBeforeIndex = i;
        return;
      }
    }
    dropBeforeIndex = memberCount;
  }

  function onGripPointerDown(
    e: PointerEvent,
    group: ClientGroupDetail,
    title: string,
    rowIndex: number,
  ) {
    if (e.button !== 0) return;
    diagClientGrouping(`pointer-start pointerId=${e.pointerId} rowIndex=${rowIndex} groupId=${group.id}`);
    e.preventDefault();
    const target = e.currentTarget;
    if (!(target instanceof HTMLElement)) return;
    const listEl = target.closest<HTMLElement>("[data-member-list]");
    if (!listEl) return;
    reorderPointerId = e.pointerId;
    reorderListEl = listEl;
    reorderGroup = group;
    dragGroupId = group.id;
    dragTitle = title;
    dropBeforeIndex = rowIndex;
  }

  function onWindowPointerMove(e: PointerEvent) {
    if (reorderPointerId == null || e.pointerId !== reorderPointerId) return;
    if (reorderListEl == null || reorderGroup == null) return;
    updateDropIndexFromPointer(e.clientY);
  }

  async function finishMemberReorder() {
    const g = reorderGroup;
    const title = dragTitle;
    const beforeIdx = dropBeforeIndex;
    clearMemberReorderState();
    if (activeProfileId == null || g == null || !title || beforeIdx == null) return;
    const titles = orderedMemberTitles(g);
    const fromIndex = titles.indexOf(title);
    if (fromIndex < 0) return;
    const next = reorderTitles(titles, fromIndex, beforeIdx);
    if (next.join("\0") === titles.join("\0")) return;
    try {
      await backend.reorderClientGroupMembers(activeProfileId, g.id, next);
      status = `Updated order for ${g.name}`;
      error = "";
      await refresh();
    } catch (err) {
      error = String(err);
    }
  }

  async function onWindowPointerUp(e: PointerEvent) {
    const matched = reorderPointerId != null && e.pointerId === reorderPointerId;
    diagClientGrouping(`pointer-up pointerId=${e.pointerId} matched=${matched}`);
    if (!matched) return;
    await finishMemberReorder();
  }

  async function onWindowPointerCancel(e: PointerEvent) {
    const matched = reorderPointerId != null && e.pointerId === reorderPointerId;
    diagClientGrouping(`pointer-cancel pointerId=${e.pointerId} matched=${matched}`);
    if (!matched) return;
    await finishMemberReorder();
  }

  onMount(() => {
    const clip = (value: string, max = 80): string =>
      value.length > max ? `${value.slice(0, max - 1)}…` : value;
    const targetSummary = (target: EventTarget | null): string => {
      if (!(target instanceof Element)) return "unknown";
      const tag = target.tagName.toLowerCase();
      const id = target.id ? `#${clip(target.id, 24)}` : "";
      const classToken = target.classList.item(0);
      const cls = classToken ? `.${clip(classToken, 32)}` : "";
      return `${tag}${id}${cls}`;
    };
    let lastCaptureLogAt = 0;
    const onDocumentCapturedPointerDown = (e: PointerEvent) => {
      const now = Date.now();
      if (now - lastCaptureLogAt < 500) return;
      lastCaptureLogAt = now;
      const path = e.composedPath().slice(0, 3).map((x) => targetSummary(x)).join(">");
      diagClientGrouping(`capture pointerdown target=${targetSummary(e.target)} path=${path}`);
    };
    const onDocumentCapturedClick = (e: MouseEvent) => {
      const now = Date.now();
      if (now - lastCaptureLogAt < 500) return;
      lastCaptureLogAt = now;
      const path = e.composedPath().slice(0, 3).map((x) => targetSummary(x)).join(">");
      diagClientGrouping(`capture click target=${targetSummary(e.target)} path=${path}`);
    };
    const onWindowError = (e: ErrorEvent) => {
      const text = e.message || "unknown";
      diagClientGrouping(`window-error ${clip(text, 140)}`);
    };
    const onWindowUnhandledRejection = (e: PromiseRejectionEvent) => {
      const reason =
        typeof e.reason === "string"
          ? e.reason
          : e.reason instanceof Error
            ? e.reason.message
            : String(e.reason);
      diagClientGrouping(`window-unhandledrejection ${clip(reason, 140)}`);
    };
    diagClientGrouping("mounted");
    void refresh();
    document.addEventListener("pointerdown", onDocumentCapturedPointerDown, true);
    document.addEventListener("click", onDocumentCapturedClick, true);
    window.addEventListener("error", onWindowError);
    window.addEventListener("unhandledrejection", onWindowUnhandledRejection);
    window.addEventListener("pointermove", onWindowPointerMove, true);
    window.addEventListener("pointerup", onWindowPointerUp, true);
    window.addEventListener("pointercancel", onWindowPointerCancel, true);
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
      document.removeEventListener("pointerdown", onDocumentCapturedPointerDown, true);
      document.removeEventListener("click", onDocumentCapturedClick, true);
      window.removeEventListener("error", onWindowError);
      window.removeEventListener("unhandledrejection", onWindowUnhandledRejection);
      window.removeEventListener("pointermove", onWindowPointerMove, true);
      window.removeEventListener("pointerup", onWindowPointerUp, true);
      window.removeEventListener("pointercancel", onWindowPointerCancel, true);
      clearMemberReorderState();
      unlistenCaptured?.();
      stopHotkeyCapture();
    };
  });

  $effect(() => {
    if (status) toast.success(status);
  });

  $effect(() => {
    if (error) toast.error(error);
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
          cycling among members that currently have live thumbnails (offline clients are skipped).
          Clients are matched by window title (same as thumbnails). Registered cycle hotkeys run only when the focused window is one of the clients currently
          tracked by the thumbnail service (same PIDs as live thumbnails).
        </CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
    {#if activeProfileId == null}
      <p class="text-muted-foreground mt-4 text-sm">Select an active profile to manage client groups.</p>
    {:else}
      <div class="mt-4 flex flex-wrap items-center gap-2 border-b border-border pb-4">
        <Button type="button" onclick={() => (createGroupDialogOpen = true)}>
          <PlusIcon class="size-4 shrink-0" aria-hidden="true" />
          Create group
        </Button>
      </div>

      <ClientGroupCreateDialog
        bind:open={createGroupDialogOpen}
        bind:groupName={newGroupName}
        onCreate={() => void createGroup()}
        onOpenChange={(open) => {
          if (!open) newGroupName = "";
        }}
      />

      <p class="text-muted-foreground mt-2 text-sm">
        Active profile:
        <span class="text-foreground font-medium">{profiles.find((p) => p.isActive)?.name ?? ""}</span>
      </p>

      <p class="text-muted-foreground mt-3 text-xs">
        Add clients from current thumbnail window titles. Press and drag the grip to reorder (pointer-based; the
        drop preview shows the client title and where it will land). Cycle hotkeys follow this order.
      </p>

      {#if groups.length === 0}
        <p class="text-muted-foreground mt-4 text-sm">No client groups for this profile.</p>
      {/if}

      <div class="mt-4 flex flex-col gap-3">
        {#each groups as group (group.id)}
          <ClientGroupCard
            {group}
            availableToAdd={availableToAddForGroup(group)}
            {dragGroupId}
            {dragTitle}
            {dropBeforeIndex}
            {isCapturingHotkey}
            {onGroupCycleHotkeyPointerDown}
            {onSaveHotkeysBlur}
            onRemoveGroup={removeGroup}
            onAddMember={addMember}
            onRemoveMember={removeMember}
            {onGripPointerDown}
          />
        {/each}
      </div>
    {/if}
  </CardContent>
</Card>
