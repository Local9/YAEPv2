import { getCurrentWebview } from "@tauri-apps/api/webview";

/**
 * WebView2 + bits-ui (Select, DropdownMenu, Menubar): after choosing an item, scroll-lock /
 * focus routing can leave the host webview ignoring mouse input until reload, while global
 * hotkeys still work. Nudge webview focus and drop stale body styles once overlays are closed.
 */
const OPEN_OVERLAY_SELECTOR = [
  '[data-slot="select-content"][data-state="open"]',
  '[data-slot="dropdown-menu-content"][data-state="open"]',
  '[data-slot="dropdown-menu-sub-content"][data-state="open"]',
  '[data-slot="menubar-content"][data-state="open"]',
  '[data-slot="menubar-sub-content"][data-state="open"]',
  '[data-slot="context-menu-content"][data-state="open"]',
  '[data-slot="context-menu-sub-content"][data-state="open"]',
  '[data-slot="popover-content"][data-state="open"]',
  '[data-slot="dialog-overlay"][data-state="open"]',
  '[data-slot="alert-dialog-overlay"][data-state="open"]',
  '[data-slot="sheet-overlay"][data-state="open"]'
].join(",");

function hasOpenOverlay(): boolean {
  return document.querySelector(OPEN_OVERLAY_SELECTOR) !== null;
}

function clearStaleBodyScrollLock(): void {
  if (hasOpenOverlay()) return;
  document.body.style.removeProperty("pointer-events");
  document.body.style.removeProperty("overflow");
  document.body.style.removeProperty("padding-right");
  document.body.style.removeProperty("margin-right");
}

function nudgeWebviewFocus(): void {
  void getCurrentWebview().setFocus().catch(() => {
    /* not running inside Tauri or webview unavailable */
  });
}

/** Debounced: bits-ui often finishes closing after the event; avoid stacking many timers during drag. */
let recoverTimerId: ReturnType<typeof window.setTimeout> | null = null;

function scheduleWebviewInputRecovery(): void {
  if (recoverTimerId !== null) {
    window.clearTimeout(recoverTimerId);
  }
  recoverTimerId = window.setTimeout(() => {
    recoverTimerId = null;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        clearStaleBodyScrollLock();
        nudgeWebviewFocus();
      });
    });
  }, 40);
}

export function init(): void {
  // Any click can be an outside-dismiss or end of drag; release-only paths missed Escape / backdrop.
  document.addEventListener("pointerup", () => scheduleWebviewInputRecovery(), true);

  document.addEventListener(
    "keydown",
    (e) => {
      if (e.key === "Escape") {
        scheduleWebviewInputRecovery();
      }
    },
    true
  );

  document.addEventListener(
    "change",
    (e) => {
      const t = e.target;
      if (t instanceof HTMLSelectElement) {
        scheduleWebviewInputRecovery();
        return;
      }
      if (t instanceof HTMLInputElement && (t.type === "checkbox" || t.type === "radio")) {
        scheduleWebviewInputRecovery();
      }
    },
    true
  );

  // Alt-tab and similar: WebView2 can keep stale hit-testing until focus moves.
  window.addEventListener("focus", () => scheduleWebviewInputRecovery());
}
