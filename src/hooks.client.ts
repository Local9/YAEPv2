import { getCurrentWebview } from "@tauri-apps/api/webview";

/**
 * WebView2 + bits-ui (Select, DropdownMenu, Menubar): after choosing an item, scroll-lock /
 * focus routing can leave the host webview ignoring mouse input until reload, while global
 * hotkeys still work. Nudge webview focus and drop stale body styles once overlays are closed.
 */
const OPEN_OVERLAY_SELECTOR = [
  '[data-slot="select-content"][data-state="open"]',
  '[data-slot="dropdown-menu-content"][data-state="open"]',
  '[data-slot="menubar-content"][data-state="open"]',
  '[data-slot="context-menu-content"][data-state="open"]',
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

function afterFloatingUiPointerChoice(): void {
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      window.setTimeout(() => {
        clearStaleBodyScrollLock();
        nudgeWebviewFocus();
      }, 32);
    });
  });
}

function isFloatingUiChoiceTarget(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) return false;
  return Boolean(
    target.closest('[data-slot="select-item"]') ||
      target.closest('[data-slot="dropdown-menu-item"]') ||
      target.closest('[data-slot="menubar-item"]') ||
      target.closest('[data-slot="context-menu-item"]') ||
      target.closest('[data-slot="switch"]') ||
      target.closest('[data-slot="checkbox"]')
  );
}

export function init(): void {
  document.addEventListener(
    "pointerup",
    (e) => {
      if (isFloatingUiChoiceTarget(e.target)) {
        afterFloatingUiPointerChoice();
      }
    },
    true
  );

  document.addEventListener(
    "change",
    (e) => {
      const t = e.target;
      if (t instanceof HTMLSelectElement) {
        afterFloatingUiPointerChoice();
        return;
      }
      if (t instanceof HTMLInputElement && (t.type === "checkbox" || t.type === "radio")) {
        afterFloatingUiPointerChoice();
      }
    },
    true
  );
}
