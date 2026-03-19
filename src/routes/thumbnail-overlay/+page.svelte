<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";

  type Payload = {
    overlayId: string;
    pid: number;
    focused: boolean;
    focusBorderColor: string;
    focusBorderThickness: number;
    showTitleOverlay: boolean;
    title: string;
  };

  let overlayId = $state("");
  let pid = $state(0);
  let focused = $state(false);
  let focusBorderColor = $state("#d47800");
  let focusBorderThickness = $state(2);
  let showTitleOverlay = $state(false);
  let title = $state("");

  function applyPayload(p: Payload) {
    if (p.overlayId !== overlayId) return;
    focused = p.focused;
    focusBorderColor = p.focusBorderColor;
    focusBorderThickness = Number(p.focusBorderThickness);
    showTitleOverlay = p.showTitleOverlay;
    title = p.title;
  }

  onMount(() => {
    const u = new URL(window.location.href);
    overlayId = u.searchParams.get("overlayId") ?? "";
    pid = Number(u.searchParams.get("pid") ?? 0);

    void (async () => {
      try {
        const initial = await invoke<Payload | null>("get_thumbnail_overlay_state", {
          overlayId,
        });
        if (initial) applyPayload(initial);
      } catch {
        /* dev server / no Tauri */
      }
    })();

    let unlisten: UnlistenFn | undefined;
    const p = listen<Payload>("thumbnail-overlay:state", (event) => {
      applyPayload(event.payload);
    });
    p.then((u) => {
      unlisten = u;
    });
    return () => {
      unlisten?.();
    };
  });
</script>

<div class="thumb-overlay-root">
  {#if showTitleOverlay && title}
    <div class="thumb-overlay-title text-2xl text-white font-bold">{title}</div>
  {/if}
  {#if focused}
    <div
      class="thumb-overlay-frame"
      style:--border={focusBorderColor}
      style:--thickness={`${Math.min(12, Math.max(1, focusBorderThickness))}px`}
    ></div>
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0 !important;
    padding: 0 !important;
    background: transparent !important;
    background-color: transparent !important;
    overflow: hidden;
    width: 100%;
    height: 100%;
    min-height: 100%;
  }

  .thumb-overlay-root {
    position: relative;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    pointer-events: none;
  }

  .thumb-overlay-title {
    position: absolute;
    top: 4px;
    left: 6px;
    right: 6px;
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    pointer-events: none;
    opacity: 1;
  }

  .thumb-overlay-frame {
    position: absolute;
    inset: 0;
    border: var(--thickness) solid var(--border);
    box-sizing: border-box;
    pointer-events: none;
    opacity: 1;
  }
</style>
