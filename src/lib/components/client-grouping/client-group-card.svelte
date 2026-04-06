<script lang="ts">
  import type { ClientGroupDetail } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Collapsible from "$lib/components/ui/collapsible";
  import { Skeleton } from "$lib/components/ui/skeleton";
  import GripVerticalIcon from "@lucide/svelte/icons/grip-vertical";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import { orderedMemberTitles } from "$lib/client-grouping/client-grouping-helpers";
  import {
    CLIENT_GROUP_BACKWARD_CAPTURE,
    CLIENT_GROUP_FORWARD_CAPTURE,
    CYCLE_HOTKEY_CAPTURE_RING_CLASS,
    CYCLE_HOTKEY_INPUT_CLASS,
    type GroupHotkeyCaptureKind,
  } from "./client-group-hotkeys";

  interface Props {
    group: ClientGroupDetail;
    availableToAdd: string[];
    dragGroupId: number | null;
    dragTitle: string | null;
    dropBeforeIndex: number | null;
    isCapturingHotkey: (groupId: number, kind: GroupHotkeyCaptureKind) => boolean;
    onGroupCycleHotkeyPointerDown: (group: ClientGroupDetail, kind: GroupHotkeyCaptureKind) => void;
    onSaveHotkeysBlur: (group: ClientGroupDetail, kind: GroupHotkeyCaptureKind) => void;
    onRemoveGroup: (group: ClientGroupDetail) => void;
    onAddMember: (group: ClientGroupDetail, windowTitle: string) => void;
    onRemoveMember: (group: ClientGroupDetail, windowTitle: string) => void;
    onGripPointerDown: (e: PointerEvent, group: ClientGroupDetail, title: string, rowIndex: number) => void;
  }

  let {
    group,
    availableToAdd,
    dragGroupId,
    dragTitle,
    dropBeforeIndex,
    isCapturingHotkey,
    onGroupCycleHotkeyPointerDown,
    onSaveHotkeysBlur,
    onRemoveGroup,
    onAddMember,
    onRemoveMember,
    onGripPointerDown,
  }: Props = $props();

</script>

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
    <div class="mb-3 grid gap-3 sm:grid-cols-2">
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
          value={group.cycleForwardHotkey}
          placeholder={isCapturingHotkey(group.id, CLIENT_GROUP_FORWARD_CAPTURE)
            ? "Press chord, release key…"
            : "Click here, then press keys"}
          onpointerdown={() => void onGroupCycleHotkeyPointerDown(group, CLIENT_GROUP_FORWARD_CAPTURE)}
          onpaste={(e) => e.preventDefault()}
          onblur={() => onSaveHotkeysBlur(group, CLIENT_GROUP_FORWARD_CAPTURE)}
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
          value={group.cycleBackwardHotkey}
          placeholder={isCapturingHotkey(group.id, CLIENT_GROUP_BACKWARD_CAPTURE)
            ? "Press chord, release key…"
            : "Click here, then press keys"}
          onpointerdown={() => void onGroupCycleHotkeyPointerDown(group, CLIENT_GROUP_BACKWARD_CAPTURE)}
          onpaste={(e) => e.preventDefault()}
          onblur={() => onSaveHotkeysBlur(group, CLIENT_GROUP_BACKWARD_CAPTURE)}
        />
      </div>
    </div>
    <div class="mb-3 flex flex-wrap justify-end">
      <Button type="button" variant="destructive" onclick={() => void onRemoveGroup(group)}>
        <Trash2Icon class="size-4 shrink-0" aria-hidden="true" />
        Delete group
      </Button>
    </div>

    <div class="mb-2">
      <label class="text-muted-foreground mb-1 block text-xs" for="add-client-{group.id}">
        Add client to group
      </label>
      {#if availableToAdd.length === 0}
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
            if (v) void onAddMember(group, v);
          }}
        >
          <option value="">Choose a window title...</option>
          {#each availableToAdd as title (title)}
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
        data-member-list
        class="bg-muted/20 max-w-3xl rounded-lg border border-dashed border-border p-2"
        role="list"
      >
        {#snippet reorderDropSkeleton(spacingClass: string, windowTitle: string)}
          <div
            class="border-primary/45 bg-muted/15 flex items-center gap-2 rounded-md border border-dashed px-2 py-2 shadow-xs {spacingClass}"
          >
            <Skeleton class="size-6 shrink-0 rounded-sm" />
            <span class="text-foreground min-w-0 flex-1 truncate text-sm" title={windowTitle}
              >{windowTitle}</span>
            <Skeleton class="h-8 w-18 shrink-0 rounded-md" />
          </div>
        {/snippet}
        {#each memberTitles as title, i (title)}
          {#if dragGroupId === group.id && dragTitle != null && dropBeforeIndex === i}
            {@render reorderDropSkeleton("mb-1", dragTitle)}
          {/if}
          <div
            role="listitem"
            data-reorder-row={i}
            class="hover:bg-muted/40 flex items-center gap-2 rounded-md border border-transparent px-2 py-2 {dragGroupId ===
              group.id && dragTitle === title
              ? 'hidden'
              : ''}"
          >
            <span
              tabindex="-1"
              role="button"
              aria-grabbed={dragGroupId === group.id && dragTitle === title}
              class="text-muted-foreground inline-flex size-6 shrink-0 cursor-grab touch-none select-none active:cursor-grabbing"
              onpointerdown={(e) => onGripPointerDown(e, group, title, i)}
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
              onclick={() => void onRemoveMember(group, title)}
            >
              Remove
            </Button>
          </div>
        {/each}
        {#if dragGroupId === group.id && dragTitle != null && dropBeforeIndex === memberTitles.length}
          {@render reorderDropSkeleton("mt-1", dragTitle)}
        {/if}
      </div>
    {/if}
  </Collapsible.Content>
</Collapsible.Root>
