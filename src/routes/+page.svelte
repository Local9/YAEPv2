<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import type {
    AppReleaseCheck,
    HealthSnapshot,
    RuntimeThumbnailStateSnapshot
  } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Separator } from "$lib/components/ui/separator";
  import ActivityIcon from "@lucide/svelte/icons/activity";
  import InfoIcon from "@lucide/svelte/icons/info";
  import LayoutDashboardIcon from "@lucide/svelte/icons/layout-dashboard";
  import ListIcon from "@lucide/svelte/icons/list";
  import MousePointerClickIcon from "@lucide/svelte/icons/mouse-pointer-click";

  interface ThumbnailEvent {
    pid: number;
    windowTitle: string;
  }

  interface FocusEvent {
    pid: number | null;
    windowTitle: string | null;
  }

  let health = $state<HealthSnapshot | null>(null);
  let error = $state("");
  let releaseCheck = $state<AppReleaseCheck | null>(null);
  let releaseError = $state("");
  let activeThumbnails = $state<ThumbnailEvent[]>([]);
  let focused = $state<FocusEvent>({ pid: null, windowTitle: null });

  function userSafeRuntimeErrorMessage(): string {
    return "Unable to communicate with the backend right now.";
  }

  function userSafeReleaseErrorMessage(): string {
    return "Unable to check for updates right now.";
  }

  onMount(() => {
    const cleanup: Array<() => void> = [];
    void (async () => {
      cleanup.push(
        await listen<ThumbnailEvent>("thumbnailAdded", (event) => {
          if (activeThumbnails.some((x) => x.pid === event.payload.pid)) {
            return;
          }
          activeThumbnails = [...activeThumbnails, event.payload];
        }),
      );
      cleanup.push(
        await listen<ThumbnailEvent>("thumbnailRemoved", (event) => {
          activeThumbnails = activeThumbnails.filter((x) => x.pid !== event.payload.pid);
        }),
      );
      cleanup.push(
        await listen<ThumbnailEvent>("thumbnailUpdated", (event) => {
          activeThumbnails = activeThumbnails.map((x) =>
            x.pid === event.payload.pid ? event.payload : x,
          );
        }),
      );
      cleanup.push(
        await listen<FocusEvent>("focusChanged", (event) => {
          focused = event.payload;
        }),
      );

      try {
        const runtimeState: RuntimeThumbnailStateSnapshot = await backend.getRuntimeThumbnailState();
        activeThumbnails = runtimeState.thumbnails;
        focused = runtimeState.focused;
        health = await backend.health();
      } catch (err) {
        error = userSafeRuntimeErrorMessage();
      }

      try {
        releaseCheck = await backend.checkLatestRelease();
      } catch (err) {
        releaseError = userSafeReleaseErrorMessage();
      }
    })();

    return () => {
      for (const fn of cleanup) fn();
    };
  });

  async function activateWindow(pid: number) {
    try {
      await backend.activateWindowByPid(pid);
    } catch (err) {
      error = userSafeRuntimeErrorMessage();
    }
  }

  async function openLatestReleasePage() {
    if (!releaseCheck?.updateAvailable || !releaseCheck.releaseUrl) {
      return;
    }
    try {
      await backend.openExternalUrl(releaseCheck.releaseUrl);
    } catch (err) {
      releaseError = userSafeReleaseErrorMessage();
    }
  }
</script>

<div class="space-y-4">
  <Card class="shadow-sm">
    <CardHeader>
      <div class="flex items-start gap-3">
        <InfoIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
        <div>
          <CardTitle class="text-lg font-semibold tracking-tight">Application Updates</CardTitle>
          <CardDescription class="mt-1 text-pretty">
            Download the latest YAEP release when a newer version is available.
          </CardDescription>
        </div>
      </div>
    </CardHeader>
    <CardContent>
      {#if releaseCheck?.updateAvailable}
        <p class="text-sm">
          New version available:
          <strong class="font-medium text-foreground">{releaseCheck.latestVersion}</strong>
          (current {releaseCheck.currentVersion})
        </p>
        <Button type="button" class="mt-3" onclick={openLatestReleasePage}>Open Release Page</Button>
      {:else if releaseCheck}
        <p class="text-sm">You are up to date (version {releaseCheck.currentVersion}).</p>
      {:else if releaseError}
        <p class="text-sm text-destructive">{releaseError}</p>
      {:else}
        <p class="text-sm text-muted-foreground">Checking for updates...</p>
      {/if}
    </CardContent>
  </Card>

  <Card class="shadow-sm">
    <CardHeader>
      <div class="flex items-start gap-3">
        <LayoutDashboardIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
        <div>
          <CardTitle class="text-lg font-semibold tracking-tight">Dashboard</CardTitle>
          <CardDescription class="mt-1 text-pretty">
            Runtime status for thumbnails and focus activity.
          </CardDescription>
        </div>
      </div>
    </CardHeader>
    <CardContent>
      {#if health}
        <p class="text-sm">
          Backend status:
          <strong class="font-medium text-foreground">{health.backendReady ? "ready" : "not ready"}</strong>
        </p>
        <p class="text-sm text-muted-foreground">Active profile id: {health.activeProfileId ?? "none"}</p>
      {:else if error}
        <p class="text-sm text-destructive">Backend status: error ({error})</p>
      {:else}
        <p class="text-sm text-muted-foreground">Checking backend status...</p>
      {/if}

      <Separator class="my-6" orientation="horizontal" />

      <div class="mb-3 flex items-center gap-2 text-sm font-medium text-muted-foreground">
        <ActivityIcon class="size-4 shrink-0" aria-hidden="true" />
        <h3 class="text-base font-semibold text-foreground">Phase 3 Runtime Events</h3>
      </div>
      <p class="text-sm">Tracked runtime thumbnails: {activeThumbnails.length}</p>
      <p class="text-sm text-muted-foreground">Focused thumbnail: {focused.windowTitle ?? "none"}</p>
      <ul class="mt-3 space-y-2">
        {#each activeThumbnails as thumb (thumb.pid)}
          <li
            class="flex flex-wrap items-center gap-2 rounded-md border border-border/80 bg-muted/40 px-3 py-2 text-sm"
          >
            <ListIcon class="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
            <span class="min-w-0 flex-1">{thumb.windowTitle} (PID {thumb.pid})</span>
            <Button
              type="button"
              variant="secondary"
              size="sm"
              class="gap-1.5"
              onclick={() => activateWindow(thumb.pid)}
            >
              <MousePointerClickIcon class="size-3.5 shrink-0" aria-hidden="true" />
              Activate
            </Button>
          </li>
        {/each}
      </ul>
    </CardContent>
  </Card>
</div>
