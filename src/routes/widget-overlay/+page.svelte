<script lang="ts">
  import "../thumbnail-overlay/overlay-surface.css";
  import { onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import BrowserWidget from "$lib/components/widget-overlay/browser-widget.svelte";
  import MumbleLinksWidget from "$lib/components/widget-overlay/mumble-links-widget.svelte";
  import WidgetWrapper from "$lib/components/widget-overlay/widget-wrapper.svelte";
  import { type WidgetSnapshot, type WidgetOverlaySettings } from "$models/domain";
  import {
    createDefaultWidgetOverlaySettings,
    isWidgetOverlayWidgetVisible,
    mergeWidgetOverlaySettings,
    widgetOverlayHitRectsFromDom,
    widgetOverlayOpenMenuHitRects,
    withWidgetOverlayDefaultBrowserUrl
  } from "$services/widgets";

  let settings = $state<WidgetOverlaySettings | null>(null);
  let browserCardEl = $state<HTMLElement | undefined>(undefined);
  let fleetMotdCardEl = $state<HTMLElement | undefined>(undefined);
  let intelFeedCardEl = $state<HTMLElement | undefined>(undefined);
  let mumbleLinksCardEl = $state<HTMLElement | undefined>(undefined);
  let fleetMotd = $state("");
  let intelLines = $state<
    {
      prefix: string;
      parts: { type: "text" | "link"; value: string }[];
      backgroundColor: string;
      foregroundColor: string;
      contrastRatio: number;
      isSpike: boolean;
    }[]
  >([]);
  let intelFeedScrollEl = $state<HTMLElement | undefined>(undefined);

  function parseHexColor(color: string): [number, number, number] | null {
    const hex = color.trim().replace(/^#/, "");
    if (!/^[0-9a-fA-F]{6}$/.test(hex)) return null;
    const r = Number.parseInt(hex.slice(0, 2), 16);
    const g = Number.parseInt(hex.slice(2, 4), 16);
    const b = Number.parseInt(hex.slice(4, 6), 16);
    return [r, g, b];
  }

  function relativeLuminance(rgb: [number, number, number]): number {
    const convert = (v: number) => {
      const c = v / 255;
      return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
    };
    const [r, g, b] = rgb.map(convert);
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
  }

  function contrastRatio(foregroundHex: string, backgroundHex: string): number {
    const fg = parseHexColor(foregroundHex) ?? [255, 255, 255];
    const bg = parseHexColor(backgroundHex) ?? [31, 41, 55];
    const l1 = relativeLuminance(fg);
    const l2 = relativeLuminance(bg);
    const lighter = Math.max(l1, l2);
    const darker = Math.min(l1, l2);
    return (lighter + 0.05) / (darker + 0.05);
  }

  function pickReadableForeground(backgroundHex: string): { color: string; ratio: number } {
    const whiteRatio = contrastRatio("#ffffff", backgroundHex);
    const blackRatio = contrastRatio("#000000", backgroundHex);
    if (whiteRatio >= blackRatio) return { color: "#ffffff", ratio: whiteRatio };
    return { color: "#000000", ratio: blackRatio };
  }

  function normalizeSafeUrl(raw: string): string | null {
    const trimmed = raw.trim();
    if (!/^https?:\/\//i.test(trimmed)) return null;
    try {
      const parsed = new URL(trimmed);
      if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return null;
      return parsed.toString();
    } catch {
      return null;
    }
  }

  function tokenizeMessage(message: string): { type: "text" | "link"; value: string }[] {
    const urlRegex = /https?:\/\/[^\s]+/gi;
    const parts: { type: "text" | "link"; value: string }[] = [];
    let lastIndex = 0;
    for (const match of message.matchAll(urlRegex)) {
      const start = match.index ?? 0;
      if (start > lastIndex) {
        parts.push({ type: "text", value: message.slice(lastIndex, start) });
      }
      const rawUrl = match[0];
      const safeUrl = normalizeSafeUrl(rawUrl.replace(/[),.;!?]+$/, ""));
      if (safeUrl) {
        parts.push({ type: "link", value: safeUrl });
      } else {
        parts.push({ type: "text", value: rawUrl });
      }
      lastIndex = start + rawUrl.length;
    }
    if (lastIndex < message.length) {
      parts.push({ type: "text", value: message.slice(lastIndex) });
    }
    if (parts.length === 0) {
      parts.push({ type: "text", value: message });
    }
    return parts;
  }

  function onIntelLinkClick(event: MouseEvent, url: string) {
    event.preventDefault();
    void backend.openExternalUrl(url);
  }

  async function scrollIntelToBottom() {
    await tick();
    if (!intelFeedScrollEl) return;
    intelFeedScrollEl.scrollTop = intelFeedScrollEl.scrollHeight;
  }

  function applyWidgetSnapshot(snapshot: WidgetSnapshot) {
    fleetMotd = snapshot.fleetMotd ?? "";
    const lines = snapshot.intelLines ?? [];
    intelLines = lines.map(
      (line) => {
        const backgroundColor = line.backgroundColor || "#1f2937";
        const readable = pickReadableForeground(backgroundColor);
        const prefix = `[${line.timestamp || "Unknown"}] - ${line.channelName} - `;
        return {
          prefix,
          parts: tokenizeMessage(line.message),
          backgroundColor,
          foregroundColor: readable.color,
          contrastRatio: readable.ratio,
          isSpike: /\bspike\b/i.test(line.message)
        };
      }
    );
    if (lines.length > 0) {
      void scrollIntelToBottom();
    }
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
    const rects = [
      ...widgetOverlayHitRectsFromDom(settings, {
        browser: browserCardEl,
        fleetMotd: fleetMotdCardEl,
        intelFeed: intelFeedCardEl,
        mumbleLinks: mumbleLinksCardEl
      }),
      ...widgetOverlayOpenMenuHitRects()
    ];
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
      settings = withWidgetOverlayDefaultBrowserUrl(mergeWidgetOverlaySettings(loaded));
    } catch {
      settings = withWidgetOverlayDefaultBrowserUrl(createDefaultWidgetOverlaySettings());
    }
    await pushHitRegions();
  }

  onMount(() => {
    let unlistenSettings: UnlistenFn | undefined;
    let unlistenWidgetUpdate: UnlistenFn | undefined;
    void listen("widget-overlay-settings-changed", () => {
      void reloadSettingsFromBackend();
    }).then((u) => {
      unlistenSettings = u;
    });

    void reloadSettingsFromBackend();
    void backend.widgetGetSnapshot().then((snapshot) => {
      applyWidgetSnapshot(snapshot);
    });
    void listen<{ snapshot: WidgetSnapshot }>("widget:update", (event) => {
      applyWidgetSnapshot(event.payload.snapshot);
    }).then((u) => {
      unlistenWidgetUpdate = u;
    });

    const t = window.setInterval(() => void pushHitRegions(), 400);
    return () => {
      unlistenSettings?.();
      unlistenWidgetUpdate?.();
      window.clearInterval(t);
    };
  });

  $effect(() => {
    const count = intelLines.length;
    if (!intelFeedScrollEl || count === 0) return;
    void scrollIntelToBottom();
  });
</script>

<div class="widget-overlay-root">
  {#if settings && isWidgetOverlayWidgetVisible("browser", settings)}
    <BrowserWidget
      bind:browser={settings.layout.browser}
      bind:pinned={settings.browserAlwaysDisplayed}
      quickLinks={settings.browserQuickLinks}
      bind:rootEl={browserCardEl}
      onPersist={persistLayout}
      onPinnedPersist={persistOverlaySettings}
    />
  {/if}
  {#if settings && isWidgetOverlayWidgetVisible("fleetMotd", settings)}
    <WidgetWrapper
      title="Fleet MOTD"
      shellAriaLabel="Fleet MOTD widget"
      bind:frame={settings!.layout.fleetMotd}
      bind:pinned={settings!.fleetMotdAlwaysDisplayed}
      bind:rootEl={fleetMotdCardEl}
      onPersist={persistLayout}
      onPinnedPersist={persistOverlaySettings}
      minWidth={320}
      minHeight={120}
    >
      {#snippet children()}
        <div class="fleet-motd">
          {#if fleetMotd}
            <div class="value">{fleetMotd}</div>
          {:else}
            <div class="empty">No Fleet MOTD detected yet.</div>
          {/if}
        </div>
      {/snippet}
    </WidgetWrapper>
  {/if}
  {#if settings && isWidgetOverlayWidgetVisible("mumbleLinks", settings)}
    <MumbleLinksWidget
      bind:frame={settings!.layout.mumbleLinks}
      bind:pinned={settings!.mumbleLinksAlwaysDisplayed}
      bind:rootEl={mumbleLinksCardEl}
      onPersist={persistLayout}
      onPinnedPersist={persistOverlaySettings}
    />
  {/if}
  {#if settings && isWidgetOverlayWidgetVisible("intelFeed", settings)}
    <WidgetWrapper
      title="Intel Feed"
      shellAriaLabel="Intel feed widget"
      bind:frame={settings!.layout.intelFeed}
      bind:pinned={settings!.intelFeedAlwaysDisplayed}
      bind:rootEl={intelFeedCardEl}
      onPersist={persistLayout}
      onPinnedPersist={persistOverlaySettings}
      minWidth={420}
      minHeight={180}
    >
      {#snippet children()}
        <div class="intel-feed" bind:this={intelFeedScrollEl}>
          {#if intelLines.length === 0}
            <div class="empty">No intel messages yet.</div>
          {:else}
            <div class="lines">
              {#each intelLines as intelLine, index (index)}
                <div
                  class="line {intelLine.isSpike ? 'line-spike' : ''}"
                  style:background-color={intelLine.backgroundColor}
                  style:color={intelLine.foregroundColor}
                >
                  <span>{intelLine.prefix}</span>
                  {#each intelLine.parts as part, partIndex (partIndex)}
                    {#if part.type === "link"}
                      <a
                        class="intel-link"
                        href={part.value}
                        onclick={(event) => onIntelLinkClick(event, part.value)}
                      >
                        {part.value}
                      </a>
                    {:else}
                      <span>{part.value}</span>
                    {/if}
                  {/each}
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/snippet}
    </WidgetWrapper>
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
  .fleet-motd {
    flex: 1;
    overflow: auto;
    padding: 10px 12px;
    background: hsl(var(--background));
  }
  .fleet-motd .value {
    font-size: 12px;
    white-space: pre-wrap;
  }
  .fleet-motd .empty {
    font-size: 12px;
    color: hsl(var(--muted-foreground));
  }
  .intel-feed {
    flex: 1;
    overflow: auto;
    padding: 10px 12px;
    background: hsl(var(--background));
  }
  .intel-feed .lines {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .intel-feed .line {
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    border-radius: 4px;
    padding: 4px 6px;
  }
  .intel-feed .line.line-spike {
    animation: intel-spike-pulse 1.1s ease-in-out infinite;
  }
  @keyframes intel-spike-pulse {
    0% {
      box-shadow: 0 0 0 0 rgb(239 68 68 / 0.75);
      transform: scale(1);
    }
    60% {
      box-shadow: 0 0 0 7px rgb(239 68 68 / 0);
      transform: scale(1.005);
    }
    100% {
      box-shadow: 0 0 0 0 rgb(239 68 68 / 0);
      transform: scale(1);
    }
  }
  .intel-feed .intel-link {
    text-decoration: underline;
    text-underline-offset: 2px;
    color: inherit;
  }
  .intel-feed .empty {
    font-size: 12px;
    color: hsl(var(--muted-foreground));
  }
</style>
