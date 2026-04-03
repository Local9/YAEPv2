<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import GripVerticalIcon from "@lucide/svelte/icons/grip-vertical";
  import PinIcon from "@lucide/svelte/icons/pin";
  import type { Snippet } from "svelte";
  import type { WidgetLayoutRect } from "$models/domain";

  /** Bindable geometry plus optional fields (e.g. `url`) preserved across drag/resize. */
  type WidgetFrameModel = WidgetLayoutRect & Record<string, unknown>;

  const DEFAULT_MIN_W = 220;
  const DEFAULT_MIN_H = 160;

  let {
    title,
    frame = $bindable(),
    pinned = $bindable(false),
    rootEl = $bindable<HTMLElement | undefined>(undefined),
    minWidth = DEFAULT_MIN_W,
    minHeight = DEFAULT_MIN_H,
    showPin = true,
    shellAriaLabel,
    onPersist,
    onPinnedPersist,
    toolbar,
    children
  }: {
    title: string;
    frame: WidgetFrameModel;
    pinned?: boolean;
    rootEl?: HTMLElement | undefined;
    minWidth?: number;
    minHeight?: number;
    showPin?: boolean;
    /** Defaults to `title` if unset. */
    shellAriaLabel?: string;
    onPersist: () => void | Promise<void>;
    /** Called after pin toggles; defaults to `onPersist` when omitted. */
    onPinnedPersist?: () => void | Promise<void>;
    toolbar?: Snippet;
    children: Snippet;
  } = $props();

  let drag = $state<{ dx: number; dy: number } | null>(null);
  let resize = $state<{ startX: number; startY: number; startW: number; startH: number } | null>(null);

  const label = $derived(shellAriaLabel?.trim() || title);

  function clampSize(w: number, h: number, x: number, y: number) {
    const maxW = Math.max(minWidth, window.innerWidth - x - 8);
    const maxH = Math.max(minHeight, window.innerHeight - y - 8);
    return {
      w: Math.min(Math.max(w, minWidth), maxW),
      h: Math.min(Math.max(h, minHeight), maxH)
    };
  }

  function onPointerMove(e: PointerEvent) {
    if (!drag) return;
    const nx = e.clientX - drag.dx;
    const ny = e.clientY - drag.dy;
    frame = { ...frame, x: nx, y: ny };
  }

  function onResizeMove(e: PointerEvent) {
    if (!resize) return;
    const nw = resize.startW + (e.clientX - resize.startX);
    const nh = resize.startH + (e.clientY - resize.startY);
    const { w, h } = clampSize(nw, nh, frame.x, frame.y);
    frame = { ...frame, width: w, height: h };
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

  async function endResize() {
    window.removeEventListener("pointermove", onResizeMove);
    window.removeEventListener("pointerup", endResize);
    window.removeEventListener("pointercancel", endResize);
    resize = null;
    try {
      await invoke("widget_overlay_set_dragging", { dragging: false });
    } catch {
      /* dev */
    }
    await onPersist();
  }

  function onTitleBarPointerDown(e: PointerEvent) {
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

  function onResizePointerDown(e: PointerEvent) {
    e.stopPropagation();
    e.preventDefault();
    resize = {
      startX: e.clientX,
      startY: e.clientY,
      startW: frame.width,
      startH: frame.height
    };
    void invoke("widget_overlay_set_dragging", { dragging: true }).catch(() => {});
    window.addEventListener("pointermove", onResizeMove);
    window.addEventListener("pointerup", endResize);
    window.addEventListener("pointercancel", endResize);
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

<div
  bind:this={rootEl}
  class="widget-shell absolute z-0 box-border touch-none select-none overflow-hidden rounded-lg border border-border bg-card text-card-foreground flex flex-col shadow-[0_1px_2px_oklch(0_0_0/0.12),0_8px_24px_oklch(0_0_0/0.2)]"
  style:left="{frame.x}px"
  style:top="{frame.y}px"
  style:width="{frame.width}px"
  style:height="{frame.height}px"
  role="application"
  aria-label={label}
>
  <div
    class="widget-title-bar flex shrink-0 items-center gap-2 px-2 py-[5px] min-h-[30px] box-border bg-muted text-foreground border-b border-border cursor-grab select-none active:cursor-grabbing"
    role="toolbar"
    tabindex="-1"
    onpointerdown={onTitleBarPointerDown}
  >
    <span
      class="widget-drag-handle flex shrink-0 items-center justify-center mr-0.5 text-muted-foreground opacity-[0.88] pointer-events-none"
      aria-hidden="true"
      title="Drag to move"
    >
      <GripVerticalIcon class="h-3.5 w-3.5" />
    </span>
    <span class="widget-title-text shrink-0 text-[12px] font-semibold tracking-[0.01em] text-foreground pointer-events-none">{title}</span>
    <span class="widget-title-flex flex-1 min-w-2 pointer-events-none"></span>
    {#if showPin}
      <button
        type="button"
        class="widget-pin flex shrink-0 items-center justify-center w-6 h-6 p-0 rounded-md border border-border bg-background text-muted-foreground cursor-pointer hover:bg-accent hover:text-accent-foreground aria-pressed:text-primary aria-pressed:border-[color-mix(in_oklch,var(--primary)_45%,var(--border))] aria-pressed:[background:color-mix(in_oklch,var(--primary)_12%,var(--background))]"
        title={pinned ? "Unpin widget (hide when widgets are toggled off)" : "Pin widget (stay visible when widgets are toggled off)"}
        aria-label={pinned ? "Unpin widget" : "Pin widget"}
        aria-pressed={pinned}
        onclick={togglePinned}
        onpointerdown={stopDragChain}
      >
        <PinIcon class="widget-pin-icon h-3.5 w-3.5" strokeWidth={pinned ? 2.25 : 1.75} />
      </button>
    {/if}
  </div>

  {#if toolbar}
    <div class="widget-toolbar-slot shrink-0">
      {@render toolbar()}
    </div>
  {/if}

  <div class="widget-body flex-1 min-h-0 flex flex-col overflow-hidden">
    {@render children()}
  </div>

  <button
    type="button"
    class="widget-resize-grip absolute right-[2px] bottom-[2px] z-2 w-[18px] h-[18px] p-0 border-0 rounded-[2px] bg-[linear-gradient(135deg,transparent_50%,color-mix(in_oklch,var(--muted-foreground)_55%,transparent)_50%)] hover:bg-[linear-gradient(135deg,transparent_45%,color-mix(in_oklch,var(--primary)_65%,transparent)_45%)] focus-visible:bg-[linear-gradient(135deg,transparent_45%,color-mix(in_oklch,var(--primary)_65%,transparent)_45%)] focus-visible:outline-none cursor-se-resize touch-none"
    aria-label="Resize widget"
    onpointerdown={onResizePointerDown}
  ></button>
</div>
