<script lang="ts">
  import "../../app.css";
  import { onMount } from "svelte";
  import { ModeWatcher, setMode } from "mode-watcher";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import type { LayoutProps } from "./$types";

  let { data, children }: LayoutProps = $props();

  const modeWatcherDefault = $derived(data.themePref === "Light" ? "light" : "dark");

  onMount(() => {
    let unlisten: UnlistenFn | undefined;
    void listen<{ theme: string }>("app-theme-changed", (e) => {
      const t = e.payload.theme;
      setMode(t === "Light" ? "light" : "dark");
    }).then((u) => {
      unlisten = u;
    });
    return () => {
      unlisten?.();
    };
  });
</script>

<ModeWatcher defaultMode={modeWatcherDefault} />
{@render children?.()}
