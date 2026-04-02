<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { Badge } from "$lib/components/ui/badge";

  interface Payload {
    overlayId: string;
    pid: number;
    focused: boolean;
    focusBorderColor: string;
    focusBorderThickness: number;
    decloakFlashColor: string;
    decloakFlashThickness: number;
    decloakFlashDurationMs: number;
    showTitleOverlay: boolean;
    title: string;
  }

  let overlayId = $state("");
  let pid = $state(0);
  let focused = $state(false);
  let focusBorderColor = $state("#d47800");
  let focusBorderThickness = $state(2);
  let decloakFlashColor = $state("#fcd34d");
  let decloakFlashThickness = $state(2);
  let decloakFlashDurationMs = $state(5000);
  let showTitleOverlay = $state(false);
  let title = $state("");
  let currentSystem = $state("");
  let isCloaked = $state<boolean | null>(null);
  let isDecloakFlashActive = $state(false);
  let decloakFlashTimeout: ReturnType<typeof setTimeout> | null = null;

  function triggerDecloakFlash() {
    if (decloakFlashTimeout) {
      clearTimeout(decloakFlashTimeout);
    }
    isDecloakFlashActive = true;
    decloakFlashTimeout = setTimeout(() => {
      isDecloakFlashActive = false;
      decloakFlashTimeout = null;
    }, Math.max(250, decloakFlashDurationMs));
  }

  function applyPayload(p: Payload) {
    if (p.overlayId !== overlayId) return;
    focused = p.focused;
    focusBorderColor = p.focusBorderColor;
    focusBorderThickness = Number(p.focusBorderThickness);
    decloakFlashColor = p.decloakFlashColor;
    decloakFlashThickness = Number(p.decloakFlashThickness);
    decloakFlashDurationMs = Number(p.decloakFlashDurationMs);
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
    let unlistenEve: UnlistenFn | undefined;
    const p = listen<Payload>("thumbnail-overlay:state", (event) => {
      applyPayload(event.payload);
    });
    p.then((u) => {
      unlisten = u;
    });
    void listen<{
      windowTitle?: string | null;
      listenerName?: string | null;
      system?: string | null;
      isCloaked?: boolean | null;
      decloakFlash?: boolean | null;
    }>("eve-thumbnail-status", (event) => {
      const payload = event.payload;
      const targetTitle = payload.windowTitle ?? payload.listenerName ?? "";
      if (!targetTitle || !title) return;
      const normalizeName = (value: string) =>
        value
          .toLowerCase()
          .replace(/^eve\s*-\s*/i, "")
          .trim();
      const normalizedOverlayTitle = normalizeName(title);
      const normalizedEventTitle = normalizeName(targetTitle);
      if (
        normalizedOverlayTitle !== normalizedEventTitle &&
        !normalizedOverlayTitle.includes(normalizedEventTitle) &&
        !normalizedEventTitle.includes(normalizedOverlayTitle)
      ) {
        return;
      }
      if (payload.system) currentSystem = payload.system;
      if (typeof payload.isCloaked === "boolean") isCloaked = payload.isCloaked;
      if (payload.decloakFlash === true) triggerDecloakFlash();
    }).then((u) => {
      unlistenEve = u;
    });
    return () => {
      unlisten?.();
      unlistenEve?.();
      if (decloakFlashTimeout) {
        clearTimeout(decloakFlashTimeout);
      }
    };
  });
</script>

<div class="pointer-events-none fixed inset-0 box-border">
  {#if showTitleOverlay && title}
    <Badge
      variant="secondary"
      class="pointer-events-none absolute left-1.5 right-1.5 top-1 inline-flex max-w-[calc(100%-0.75rem)] overflow-hidden text-xs text-white"
    >
      <span class="block overflow-hidden text-ellipsis whitespace-nowrap">
        {#if currentSystem}[{currentSystem}]&nbsp;{/if}{title}
      </span>
    </Badge>
  {/if}
  {#if focused}
    <div
      class="pointer-events-none absolute inset-0 box-border border-solid border-border [border-width:var(--thickness)] opacity-100"
      style:--border={focusBorderColor}
      style:--thickness={`${Math.min(12, Math.max(1, focusBorderThickness))}px`}
    ></div>
  {/if}
  {#if isDecloakFlashActive}
    <div
      class="pointer-events-none absolute inset-0 box-border animate-pulse border-solid opacity-100"
      style:--decloak-flash-color={decloakFlashColor}
      style:--decloak-flash-thickness={`${Math.max(1, decloakFlashThickness)}px`}
      style:border-color="var(--decloak-flash-color)"
      style:border-width="var(--decloak-flash-thickness)"
    ></div>
  {/if}
  {#if isCloaked != null}
    <div
      class="absolute bottom-1.5 right-1.5 h-3 w-3 {isCloaked
        ? 'bg-green-500'
        : 'bg-red-500'}"
    ></div>
  {/if}
</div>
