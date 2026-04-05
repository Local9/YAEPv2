<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import RadioIcon from "@lucide/svelte/icons/radio";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from "$lib/components/ui/card";
  import { MumbleLinksController } from "./mumble-links-controller.svelte.js";
  import { setMumbleLinksPageContext } from "./mumble-links-context";
  import MumbleLinksDeleteDialog from "./mumble-links-delete-dialog.svelte";
  import MumbleLinksTree from "./mumble-links-tree.svelte";

  interface HotkeyCapturedPayload {
    value: string;
    captureType: string;
    targetId: number | null;
  }

  const ctl = new MumbleLinksController();
  setMumbleLinksPageContext(ctl);

  onMount(() => {
    void ctl.refresh();
    let unlistenCaptured: UnlistenFn | undefined;
    void listen<HotkeyCapturedPayload>("hotkeyCaptured", (event) => {
      const p = event.payload;
      if (p.captureType !== ctl.hotkeyCaptureType || p.targetId == null) return;
      ctl.captureHotkeyLinkId = null;
      void ctl.applyCapturedHotkey(p.targetId, p.value);
    }).then((u) => {
      unlistenCaptured = u;
    });
    return () => {
      unlistenCaptured?.();
      ctl.stopLinkHotkeyCapture();
    };
  });
</script>

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <RadioIcon class="text-muted-foreground mt-0.5 size-5 shrink-0" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Mumble Links</CardTitle>
        <CardDescription
          >Organize folders and links. Use the icon control to the left of each folder name to change
          its icon. Paste a URL to fill the name. Hotkeys are optional and can be set per link.</CardDescription
        >
      </div>
    </div>
  </CardHeader>
  <CardContent>
    {#if ctl.tree}
      <MumbleLinksTree />
    {:else}
      <p class="text-muted-foreground mt-4 text-sm">Loading tree…</p>
    {/if}
  </CardContent>
</Card>

<MumbleLinksDeleteDialog />
