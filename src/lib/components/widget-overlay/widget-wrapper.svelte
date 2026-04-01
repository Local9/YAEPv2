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
  class="widget-shell touch-none select-none"
  style:left="{frame.x}px"
  style:top="{frame.y}px"
  style:width="{frame.width}px"
  style:height="{frame.height}px"
  role="application"
  aria-label={label}
>
  <div
    class="widget-title-bar"
    role="toolbar"
    tabindex="-1"
    onpointerdown={onTitleBarPointerDown}
  >
    <span class="widget-drag-handle" aria-hidden="true" title="Drag to move">
      <GripVerticalIcon />
    </span>
    <span class="widget-title-text">{title}</span>
    <span class="widget-title-flex"></span>
    {#if showPin}
      <button
        type="button"
        class="widget-pin"
        title={pinned ? "Unpin widget (hide when widgets are toggled off)" : "Pin widget (stay visible when widgets are toggled off)"}
        aria-label={pinned ? "Unpin widget" : "Pin widget"}
        aria-pressed={pinned}
        onclick={togglePinned}
        onpointerdown={stopDragChain}
      >
        <PinIcon class="widget-pin-icon" strokeWidth={pinned ? 2.25 : 1.75} />
      </button>
    {/if}
  </div>

  {#if toolbar}
    <div class="widget-toolbar-slot">
      {@render toolbar()}
    </div>
  {/if}

  <div class="widget-body">
    {@render children()}
  </div>

  <button
    type="button"
    class="widget-resize-grip"
    aria-label="Resize widget"
    onpointerdown={onResizePointerDown}
  ></button>
</div>

<style>
  .widget-shell {
    position: absolute;
    z-index: 0;
    box-sizing: border-box;
    pointer-events: auto;
    overflow: hidden;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--card);
    color: var(--card-foreground);
    display: flex;
    flex-direction: column;
    box-shadow:
      0 1px 2px oklch(0 0 0 / 0.12),
      0 8px 24px oklch(0 0 0 / 0.2);
  }

  .widget-title-bar {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    min-height: 40px;
    box-sizing: border-box;
    background: var(--muted);
    color: var(--foreground);
    border-bottom: 1px solid var(--border);
    cursor: grab;
    user-select: none;
  }

  .widget-title-bar:active {
    cursor: grabbing;
  }

  .widget-drag-handle {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    margin-right: 2px;
    color: var(--muted-foreground);
    opacity: 0.88;
    pointer-events: none;
  }

  .widget-drag-handle :global(svg) {
    width: 1rem;
    height: 1rem;
  }

  .widget-title-text {
    flex-shrink: 0;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.01em;
    color: var(--foreground);
    pointer-events: none;
  }

  .widget-title-flex {
    flex: 1;
    min-width: 8px;
    pointer-events: none;
  }

  .widget-pin {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    padding: 0;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: var(--background);
    color: var(--muted-foreground);
    cursor: pointer;
  }

  .widget-pin:hover {
    background: var(--accent);
    color: var(--accent-foreground);
  }

  .widget-pin[aria-pressed="true"] {
    color: var(--primary);
    border-color: color-mix(in oklch, var(--primary) 45%, var(--border));
    background: color-mix(in oklch, var(--primary) 12%, var(--background));
  }

  .widget-pin :global(.widget-pin-icon) {
    width: 1rem;
    height: 1rem;
  }

  .widget-toolbar-slot {
    flex-shrink: 0;
  }

  .widget-body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .widget-resize-grip {
    position: absolute;
    right: 2px;
    bottom: 2px;
    z-index: 2;
    width: 18px;
    height: 18px;
    padding: 0;
    border: none;
    border-radius: 2px;
    background: linear-gradient(
      135deg,
      transparent 50%,
      color-mix(in oklch, var(--muted-foreground) 55%, transparent) 50%
    );
    cursor: se-resize;
    touch-action: none;
  }

  .widget-resize-grip:hover,
  .widget-resize-grip:focus-visible {
    background: linear-gradient(
      135deg,
      transparent 45%,
      color-mix(in oklch, var(--primary) 65%, transparent) 45%
    );
    outline: none;
  }
</style>
