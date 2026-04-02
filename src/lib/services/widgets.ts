import type { WidgetBrowserFrame, WidgetLayoutRect, WidgetOverlaySettings } from "$models/domain";
import { DEFAULT_BROWSER_QUICK_LINKS } from "$models/domain";

/** Widget shells tracked for hit-testing on the overlay window. */
export type WidgetOverlayWidgetId = "browser" | "fleetMotd" | "intelFeed" | "mumbleLinks";

export const DEFAULT_WIDGET_BROWSER_FRAME: WidgetBrowserFrame = {
  url: "",
  x: 400,
  y: 48,
  width: 480,
  height: 360
};

export const DEFAULT_WIDGET_LAYOUT_PARTIAL = {
  fleetMotd: { x: 24, y: 24, width: 420, height: 180 } satisfies WidgetLayoutRect,
  intelFeed: { x: 24, y: 220, width: 560, height: 280 } satisfies WidgetLayoutRect,
  mumbleLinks: { x: 24, y: 520, width: 200, height: 30 } satisfies WidgetLayoutRect
} as const;

function finiteOr(n: unknown, fallback: number): number {
  return typeof n === "number" && Number.isFinite(n) ? n : fallback;
}

/** Normalizes browser frame geometry and URL string from persisted or partial data. */
export function normalizeWidgetBrowserFrame(
  b: WidgetBrowserFrame | Partial<WidgetBrowserFrame> | undefined
): WidgetBrowserFrame {
  if (!b) return { ...DEFAULT_WIDGET_BROWSER_FRAME };
  const url = typeof b.url === "string" ? b.url.trim() : "";
  return {
    url,
    x: finiteOr(b.x, DEFAULT_WIDGET_BROWSER_FRAME.x),
    y: finiteOr(b.y, DEFAULT_WIDGET_BROWSER_FRAME.y),
    width: finiteOr(b.width, DEFAULT_WIDGET_BROWSER_FRAME.width),
    height: finiteOr(b.height, DEFAULT_WIDGET_BROWSER_FRAME.height)
  };
}

/**
 * Merges loaded overlay settings with app defaults (flags, quick links, layout fallbacks, browser geometry).
 * Does not inject `browserDefaultUrl` into `layout.browser.url` — use `withWidgetOverlayDefaultBrowserUrl` on the overlay route when needed.
 */
export function mergeWidgetOverlaySettings(loaded: WidgetOverlaySettings): WidgetOverlaySettings {
  const browserQuickLinks =
    Array.isArray(loaded.browserQuickLinks) && loaded.browserQuickLinks.length > 0
      ? loaded.browserQuickLinks.map((l) => ({
          id: l.id,
          url: l.url,
          title: l.title
        }))
      : [...DEFAULT_BROWSER_QUICK_LINKS];
  const browserDefaultUrl =
    loaded.browserDefaultUrl != null && String(loaded.browserDefaultUrl).trim()
      ? String(loaded.browserDefaultUrl).trim()
      : null;
  const browser = normalizeWidgetBrowserFrame(loaded.layout?.browser);
  return {
    ...loaded,
    browserQuickLinks,
    browserDefaultUrl,
    widgetsSuppressed: loaded.widgetsSuppressed ?? false,
    browserAlwaysDisplayed: loaded.browserAlwaysDisplayed ?? false,
    showFleetMotdWidget: loaded.showFleetMotdWidget ?? true,
    showIntelFeedWidget: loaded.showIntelFeedWidget ?? true,
    showMumbleLinksWidget: loaded.showMumbleLinksWidget ?? true,
    fleetMotdAlwaysDisplayed: loaded.fleetMotdAlwaysDisplayed ?? false,
    intelFeedAlwaysDisplayed: loaded.intelFeedAlwaysDisplayed ?? false,
    mumbleLinksAlwaysDisplayed: loaded.mumbleLinksAlwaysDisplayed ?? false,
    toggleHotkey: loaded.toggleHotkey ?? "",
    layout: {
      ...loaded.layout,
      browser,
      fleetMotd: loaded.layout.fleetMotd ?? { ...DEFAULT_WIDGET_LAYOUT_PARTIAL.fleetMotd },
      intelFeed: loaded.layout.intelFeed ?? { ...DEFAULT_WIDGET_LAYOUT_PARTIAL.intelFeed },
      mumbleLinks: loaded.layout.mumbleLinks ?? { ...DEFAULT_WIDGET_LAYOUT_PARTIAL.mumbleLinks }
    }
  };
}

/** When the saved page URL is empty, use the configured default URL (overlay runtime). */
export function withWidgetOverlayDefaultBrowserUrl(settings: WidgetOverlaySettings): WidgetOverlaySettings {
  const url = settings.layout.browser.url.trim();
  const def = settings.browserDefaultUrl?.trim();
  if (url || !def) return settings;
  return {
    ...settings,
    layout: {
      ...settings.layout,
      browser: { ...settings.layout.browser, url: def }
    }
  };
}

export function isWidgetOverlayWidgetVisible(id: WidgetOverlayWidgetId, s: WidgetOverlaySettings): boolean {
  switch (id) {
    case "browser":
      return s.showBrowserWidget && (!s.widgetsSuppressed || s.browserAlwaysDisplayed);
    case "fleetMotd":
      return s.showFleetMotdWidget && (!s.widgetsSuppressed || s.fleetMotdAlwaysDisplayed);
    case "intelFeed":
      return s.showIntelFeedWidget && (!s.widgetsSuppressed || s.intelFeedAlwaysDisplayed);
    case "mumbleLinks":
      return s.showMumbleLinksWidget && (!s.widgetsSuppressed || s.mumbleLinksAlwaysDisplayed);
  }
}

export type WidgetOverlayHitRect = { x: number; y: number; width: number; height: number };

export function physicalWidgetHitRect(el: HTMLElement): WidgetOverlayHitRect {
  const dpr = window.devicePixelRatio ?? 1;
  const r = el.getBoundingClientRect();
  return {
    x: Math.round(r.left * dpr),
    y: Math.round(r.top * dpr),
    width: Math.round(r.width * dpr),
    height: Math.round(r.height * dpr)
  };
}

export type WidgetOverlayRootElements = Partial<Record<WidgetOverlayWidgetId, HTMLElement | undefined>>;

/** Builds physical hit rects for visible widgets that currently have a mounted root element. */
export function widgetOverlayHitRectsFromDom(
  settings: WidgetOverlaySettings,
  roots: WidgetOverlayRootElements
): WidgetOverlayHitRect[] {
  const rects: WidgetOverlayHitRect[] = [];
  const ids: WidgetOverlayWidgetId[] = ["browser", "fleetMotd", "intelFeed", "mumbleLinks"];
  for (const id of ids) {
    if (!isWidgetOverlayWidgetVisible(id, settings)) continue;
    const el = roots[id];
    if (el) rects.push(physicalWidgetHitRect(el));
  }
  return rects;
}

/**
 * Menus rendered with a portal are not under the widget shell in the DOM, so their pixels are outside
 * the widget hit rects. The overlay window uses region hit-testing on Windows; without these rects,
 * clicks fall through the open menu.
 */
export function widgetOverlayOpenMenuHitRects(): WidgetOverlayHitRect[] {
  if (typeof document === "undefined") return [];
  const nodes = document.querySelectorAll<HTMLElement>(
    '[data-slot="dropdown-menu-content"], [data-slot="dropdown-menu-sub-content"], [data-slot="context-menu-content"], [data-slot="context-menu-sub-content"], [data-slot="menubar-content"], [data-slot="menubar-sub-content"]'
  );
  const rects: WidgetOverlayHitRect[] = [];
  for (const el of nodes) {
    if (!el.isConnected) continue;
    const r = el.getBoundingClientRect();
    if (r.width < 1 || r.height < 1) continue;
    rects.push(physicalWidgetHitRect(el));
  }
  return rects;
}

/** Fallback when the overlay cannot load settings from Tauri (dev or error). */
export function createDefaultWidgetOverlaySettings(): WidgetOverlaySettings {
  return {
    enabled: true,
    visible: true,
    monitorIndex: 0,
    showBrowserWidget: true,
    showFleetMotdWidget: true,
    showIntelFeedWidget: true,
    showMumbleLinksWidget: true,
    widgetsSuppressed: false,
    browserAlwaysDisplayed: false,
    fleetMotdAlwaysDisplayed: false,
    intelFeedAlwaysDisplayed: false,
    mumbleLinksAlwaysDisplayed: false,
    toggleHotkey: "",
    browserQuickLinks: [...DEFAULT_BROWSER_QUICK_LINKS],
    browserDefaultUrl: null,
    layout: {
      browser: { ...DEFAULT_WIDGET_BROWSER_FRAME },
      ...DEFAULT_WIDGET_LAYOUT_PARTIAL
    }
  };
}
