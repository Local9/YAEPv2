<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { ModeWatcher, mode, setMode } from "mode-watcher";
  import { page } from "$app/state";
  import type { LayoutProps } from "./$types";
  import { backend } from "$services/backend";
  import CpuIcon from "@lucide/svelte/icons/cpu";
  import Gamepad2Icon from "@lucide/svelte/icons/gamepad-2";
  import Grid3x3Icon from "@lucide/svelte/icons/grid-3x3";
  import ImageIcon from "@lucide/svelte/icons/image";
  import LayersIcon from "@lucide/svelte/icons/layers";
  import LayoutDashboardIcon from "@lucide/svelte/icons/layout-dashboard";
  import RadioIcon from "@lucide/svelte/icons/radio";
  import LayoutPanelIcon from "@lucide/svelte/icons/layout-panel-top";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import UsersIcon from "@lucide/svelte/icons/users";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import ThemeToggle from "$lib/components/theme-toggle.svelte";
  import { Toaster } from "$lib/components/ui/sonner";

  let { children }: LayoutProps = $props();
  let appLoading = $state(true);
  let appLoadError = $state("");
  /** Avoid duplicate SQLite/event writes when mode has not changed. */
  let lastSyncedTheme = $state<"Dark" | "Light" | null>(null);

  type NavSection = {
    href: string;
    label: string;
    Icon: typeof LayoutDashboardIcon;
    exact?: boolean;
  };

  const sections: NavSection[] = [
    { href: "/", label: "Dashboard", Icon: LayoutDashboardIcon },
    { href: "/profiles", label: "Profiles", Icon: UsersIcon },
    { href: "/thumbnail-settings", label: "Thumbnail Settings", Icon: ImageIcon },
    { href: "/client-grouping", label: "Client Grouping", Icon: LayersIcon },
    { href: "/grid-layout", label: "Grid Layout", Icon: Grid3x3Icon },
    { href: "/process-management", label: "Process Management", Icon: CpuIcon },
    { href: "/mumble-links", label: "Mumble Links", Icon: RadioIcon },
    { href: "/eve-profiles", label: "EVE Profiles", Icon: Gamepad2Icon },
    { href: "/settings", label: "Settings", Icon: SettingsIcon, exact: true },
    { href: "/settings/eve-logs", label: "EVE Logs", Icon: SettingsIcon },
    { href: "/settings/widget-overlay", label: "Widget overlay", Icon: LayoutPanelIcon }
  ];

  let isThumbnailOverlay = $derived(page.url.pathname.startsWith("/thumbnail-overlay"));
  let isWidgetOverlay = $derived(page.url.pathname.startsWith("/widget-overlay"));

  function navItemActive(href: string, pathname: string, exact?: boolean): boolean {
    if (exact) return pathname === href;
    if (href === "/") return pathname === "/";
    return pathname === href || pathname.startsWith(`${href}/`);
  }

  function activeElementSummary(): string {
    const el = document.activeElement;
    if (!(el instanceof HTMLElement)) return "none";
    const tag = el.tagName.toLowerCase();
    const id = el.id ? `#${el.id}` : "";
    const cls = typeof el.className === "string"
      ? `.${el.className.split(/\s+/).filter(Boolean).slice(0, 2).join(".")}`
      : "";
    return `${tag}${id}${cls}`;
  }

  function onNavClick(navHref: string, e: MouseEvent): void {
    const before = window.location.pathname;
    const isPrimary = e.button === 0;
    void backend
      .frontendDiagLog(
        "info",
        "sidebar-nav",
        `click href=${navHref} before=${before} defaultPrevented=${e.defaultPrevented} button=${e.button} ctrl=${e.ctrlKey} meta=${e.metaKey} shift=${e.shiftKey}`
      )
      .catch(() => {});
    if (!isPrimary || e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) return;
    window.setTimeout(() => {
      const after = window.location.pathname;
      if (after !== before) return;
      const bodyPointer = document.body.style.pointerEvents || "(unset)";
      const bodyOverflow = document.body.style.overflow || "(unset)";
      void backend
        .frontendDiagLog(
          "warn",
          "sidebar-nav",
          `no-route-change href=${navHref} before=${before} after=${after} defaultPrevented=${e.defaultPrevented} body.pointerEvents=${bodyPointer} body.overflow=${bodyOverflow} active=${activeElementSummary()}`
        )
        .catch(() => {});
    }, 350);
  }

  onMount(() => {
    if (isThumbnailOverlay || isWidgetOverlay) {
      appLoading = false;
      document.getElementById("boot-splash")?.remove();
      return;
    }

    const failsafeMs = 15_000;
    const failsafeId = window.setTimeout(() => {
      appLoading = false;
      document.getElementById("boot-splash")?.remove();
    }, failsafeMs);

    void (async () => {
      try {
        try {
          const diagPath = await backend.frontendDiagFilePath();
          await backend.frontendDiagLog("info", "boot", `frontend mounted; diagnostics file=${diagPath}`);
        } catch {
          /* diagnostics are best-effort */
        }
        try {
          const t = await backend.getAppSetting("Theme");
          if (t === "Light") setMode("light");
          else if (t === "Dark") setMode("dark");
        } catch {
          /* ignore */
        }
        try {
          await backend.appReady();
        } catch {
          appLoadError = "Unable to start runtime thumbnails.";
        }
      } finally {
        window.clearTimeout(failsafeId);
        appLoading = false;
        document.getElementById("boot-splash")?.remove();
      }
    })();
  });

  $effect(() => {
    if (isThumbnailOverlay || isWidgetOverlay || appLoading) return;
    const m = mode.current;
    if (m !== "light" && m !== "dark") return;
    const theme: "Dark" | "Light" = m === "light" ? "Light" : "Dark";
    if (theme === lastSyncedTheme) return;
    lastSyncedTheme = theme;
    void backend.setAppSetting("Theme", theme).catch(() => {});
  });
</script>

{#if isThumbnailOverlay || isWidgetOverlay}
  {@render children?.()}
{:else if appLoading}
  <div class="flex min-h-screen flex-col items-center justify-center gap-4 bg-background text-foreground">
    <div class="size-8 animate-spin rounded-full border-2 border-muted border-t-primary"></div>
    <p class="text-sm text-muted-foreground">Loading YAEP...</p>
  </div>
{:else if appLoadError}
  <div class="flex min-h-screen flex-col items-center justify-center gap-3 bg-background px-6 text-center">
    <p class="text-sm text-destructive">{appLoadError}</p>
    <p class="text-xs text-muted-foreground">
      You can keep using the app, but thumbnail runtime features may be unavailable.
    </p>
  </div>
{:else}
  <ModeWatcher />
  <Toaster richColors />
  <Sidebar.Provider>
    <Sidebar.Root collapsible="icon" variant="sidebar">
      <Sidebar.Header>
        <Sidebar.Menu>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton size="lg">
              {#snippet child({ props })}
                <a href="/" {...props}>
                  <div
                    class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground"
                  >
                    <LayoutDashboardIcon class="size-4" aria-hidden="true" />
                  </div>
                  <div class="grid flex-1 text-start text-sm leading-tight">
                    <span class="truncate font-semibold">YAEP - Yet Another EVE Preview</span>
                    <span class="truncate text-xs text-sidebar-foreground/70">Control panel</span>
                  </div>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
        </Sidebar.Menu>
      </Sidebar.Header>
      <Sidebar.Content>
        <Sidebar.Group>
          <Sidebar.GroupLabel>Navigation</Sidebar.GroupLabel>
          <Sidebar.GroupContent>
            <Sidebar.Menu>
              {#each sections as nav (nav.href)}
                <Sidebar.MenuItem>
                  <Sidebar.MenuButton
                    isActive={navItemActive(nav.href, page.url.pathname, nav.exact)}
                    tooltipContent={nav.label}
                  >
                    {#snippet child({ props })}
                      <a href={nav.href} {...props} onclick={(e) => onNavClick(nav.href, e)}>
                        <nav.Icon aria-hidden="true" />
                        <span>{nav.label}</span>
                      </a>
                    {/snippet}
                  </Sidebar.MenuButton>
                </Sidebar.MenuItem>
              {/each}
            </Sidebar.Menu>
          </Sidebar.GroupContent>
        </Sidebar.Group>
      </Sidebar.Content>
      <Sidebar.Rail />
    </Sidebar.Root>
    <Sidebar.Inset>
      <header class="flex h-14 shrink-0 items-center gap-2 border-b border-border px-4">
        <Sidebar.Trigger />
        <span class="truncate font-semibold md:hidden">YAEP - Yet Another EVE Preview</span>
        <ThemeToggle />
      </header>
      <div class="flex flex-1 flex-col gap-4 p-4 md:p-6">
        {@render children?.()}
      </div>
    </Sidebar.Inset>
  </Sidebar.Provider>
{/if}
