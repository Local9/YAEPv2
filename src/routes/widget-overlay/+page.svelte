<script lang="ts">
  import "../thumbnail-overlay/overlay-surface.css";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import BrowserWidget from "$lib/components/widget-overlay/browser-widget.svelte";
  import {
    DEFAULT_BROWSER_QUICK_LINKS,
    type WidgetBrowserFrame,
    type WidgetOverlaySettings
  } from "$models/domain";

  const DEFAULT_BROWSER: WidgetBrowserFrame = {
    url: "",
    x: 400,
    y: 48,
    width: 480,
    height: 360
  };

  function finiteOr(n: unknown, fallback: number): number {
    return typeof n === "number" && Number.isFinite(n) ? n : fallback;
  }

  function normalizeBrowser(b: WidgetOverlaySettings["layout"]["browser"] | undefined): WidgetBrowserFrame {
    if (!b) return { ...DEFAULT_BROWSER };
    const url = typeof b.url === "string" ? b.url.trim() : "";
    return {
      url,
      x: finiteOr(b.x, DEFAULT_BROWSER.x),
      y: finiteOr(b.y, DEFAULT_BROWSER.y),
      width: finiteOr(b.width, DEFAULT_BROWSER.width),
      height: finiteOr(b.height, DEFAULT_BROWSER.height)
    };
  }

  function mergeOverlaySettings(loaded: WidgetOverlaySettings): WidgetOverlaySettings {
    const browserQuickLinks =
      Array.isArray(loaded.browserQuickLinks) && loaded.browserQuickLinks.length > 0
        ? loaded.browserQuickLinks
        : DEFAULT_BROWSER_QUICK_LINKS;
    const browserDefaultUrl =
      loaded.browserDefaultUrl != null && String(loaded.browserDefaultUrl).trim()
        ? String(loaded.browserDefaultUrl).trim()
        : null;
    let browser = normalizeBrowser(loaded.layout.browser);
    if (!browser.url && browserDefaultUrl) {
      browser = { ...browser, url: browserDefaultUrl };
    }
    return {
      ...loaded,
      browserQuickLinks,
      browserDefaultUrl,
      widgetsSuppressed: loaded.widgetsSuppressed ?? false,
      browserAlwaysDisplayed: loaded.browserAlwaysDisplayed ?? false,
      toggleHotkey: loaded.toggleHotkey ?? "",
      layout: {
        ...loaded.layout,
        browser
      }
    };
  }

  function browserWidgetRenderedVisible(s: WidgetOverlaySettings): boolean {
    return s.showBrowserWidget && (!s.widgetsSuppressed || s.browserAlwaysDisplayed);
  }

  let settings = $state<WidgetOverlaySettings | null>(null);
  let browserCardEl = $state<HTMLElement | undefined>(undefined);

  function physicalRect(el: HTMLElement) {
    const dpr = window.devicePixelRatio ?? 1;
    const r = el.getBoundingClientRect();
    return {
      x: Math.round(r.left * dpr),
      y: Math.round(r.top * dpr),
      width: Math.round(r.width * dpr),
      height: Math.round(r.height * dpr)
    };
  }

  async function pushHitRegions() {
    if (!settings) {
      try {
        await invoke("widget_overlay_update_hit_regions", { rects: [] });
      } catch {
        /* dev without Tauri */
      }
      return;
    }
    const rects: ReturnType<typeof physicalRect>[] = [];
    if (browserWidgetRenderedVisible(settings) && browserCardEl) {
      rects.push(physicalRect(browserCardEl));
    }
    try {
      await invoke("widget_overlay_update_hit_regions", { rects });
    } catch {
      /* dev without Tauri */
    }
  }

  async function persistLayout() {
    if (settings) {
      try {
        await backend.widgetOverlaySaveLayout(settings.layout);
      } catch {
        /* dev */
      }
    }
    await pushHitRegions();
  }

  /** Pin and other overlay settings (merges layout from DB on the backend when needed). */
  async function persistOverlaySettings() {
    if (!settings) return;
    try {
      await backend.widgetOverlaySaveSettings(settings);
    } catch {
      /* dev */
    }
    await pushHitRegions();
  }

  async function reloadSettingsFromBackend() {
    try {
      const loaded = await invoke<WidgetOverlaySettings>("widget_overlay_get_settings");
      settings = mergeOverlaySettings(loaded);
    } catch {
      settings = {
        enabled: true,
        visible: true,
        monitorIndex: 0,
        showBrowserWidget: true,
        widgetsSuppressed: false,
        browserAlwaysDisplayed: false,
        toggleHotkey: "",
        browserQuickLinks: [...DEFAULT_BROWSER_QUICK_LINKS],
        browserDefaultUrl: null,
        layout: {
          browser: { ...DEFAULT_BROWSER }
        }
      };
    }
    await pushHitRegions();
  }

  onMount(() => {
    let unlistenSettings: UnlistenFn | undefined;
    void listen("widget-overlay-settings-changed", () => {
      void reloadSettingsFromBackend();
    }).then((u) => {
      unlistenSettings = u;
    });

    void reloadSettingsFromBackend();

    const t = window.setInterval(() => void pushHitRegions(), 400);
    return () => {
      unlistenSettings?.();
      window.clearInterval(t);
    };
  });
</script>

<div class="widget-overlay-root">
  {#if settings && browserWidgetRenderedVisible(settings)}
    <BrowserWidget
      bind:browser={settings.layout.browser}
      bind:pinned={settings.browserAlwaysDisplayed}
      quickLinks={settings.browserQuickLinks}
      bind:rootEl={browserCardEl}
      onPersist={persistLayout}
      onPinnedPersist={persistOverlaySettings}
    />
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

  .widget-overlay-root {
    position: fixed;
    inset: 0;
    pointer-events: none;
  }
</style>
