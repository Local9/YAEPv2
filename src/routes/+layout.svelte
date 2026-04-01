<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { ModeWatcher } from "mode-watcher";
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
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import UsersIcon from "@lucide/svelte/icons/users";
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import ThemeToggle from "$lib/components/theme-toggle.svelte";
  import { Toaster } from "$lib/components/ui/sonner";

  let { children }: LayoutProps = $props();
  let appLoading = $state(true);
  let appLoadError = $state("");

  const sections = [
    { href: "/", label: "Dashboard", Icon: LayoutDashboardIcon },
    { href: "/profiles", label: "Profiles", Icon: UsersIcon },
    { href: "/thumbnail-settings", label: "Thumbnail Settings", Icon: ImageIcon },
    { href: "/client-grouping", label: "Client Grouping", Icon: LayersIcon },
    { href: "/grid-layout", label: "Grid Layout", Icon: Grid3x3Icon },
    { href: "/process-management", label: "Process Management", Icon: CpuIcon },
    { href: "/mumble-links", label: "Mumble Links", Icon: RadioIcon },
    { href: "/eve-profiles", label: "EVE Profiles", Icon: Gamepad2Icon },
    { href: "/settings", label: "Settings", Icon: SettingsIcon },
  ] as const;

  let isThumbnailOverlay = $derived(page.url.pathname.startsWith("/thumbnail-overlay"));

  function navItemActive(href: string, pathname: string): boolean {
    if (href === "/") return pathname === "/";
    return pathname === href || pathname.startsWith(`${href}/`);
  }

  onMount(() => {
    if (isThumbnailOverlay) {
      appLoading = false;
      document.getElementById("boot-splash")?.remove();
      return;
    }

    void (async () => {
      try {
        await backend.appReady();
      } catch {
        appLoadError = "Unable to start runtime thumbnails.";
      } finally {
        appLoading = false;
        document.getElementById("boot-splash")?.remove();
      }
    })();
  });
</script>

{#if isThumbnailOverlay}
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
  <Toaster />
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
              {#each sections as { href, label, Icon } (href)}
                <Sidebar.MenuItem>
                  <Sidebar.MenuButton
                    isActive={navItemActive(href, page.url.pathname)}
                    tooltipContent={label}
                  >
                    {#snippet child({ props })}
                      <a {href} {...props}>
                        <Icon aria-hidden="true" />
                        <span>{label}</span>
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
