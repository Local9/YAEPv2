<script lang="ts">
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import type {
    AppReleaseCheck,
    EveStaticDataDownloadDone,
    EveStaticDataDownloadProgress,
    EveStaticDataStatus,
    HealthSnapshot,
    RuntimeThumbnailStateSnapshot
  } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Progress } from "$lib/components/ui/progress/index.js";
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
  import HardDriveDownloadIcon from "@lucide/svelte/icons/hard-drive-download";

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
  let eveStatic = $state<EveStaticDataStatus | null>(null);
  let eveStaticError = $state("");
  let eveStaticLoading = $state(true);
  let eveStaticBusy = $state(false);
  let eveStaticImportBusy = $state(false);
  let eveStaticProgress = $state<EveStaticDataDownloadProgress | null>(null);
  let activeThumbnails = $state<ThumbnailEvent[]>([]);
  let focused = $state<FocusEvent>({ pid: null, windowTitle: null });

  function userSafeRuntimeErrorMessage(): string {
    return "Unable to communicate with the backend right now.";
  }

  function userSafeReleaseErrorMessage(): string {
    return "Unable to check for updates right now.";
  }

  function userSafeEveStaticErrorMessage(): string {
    return "Unable to load EVE static data status.";
  }

  /** Show the EVE static data card when install/update/import is needed, or while downloading/importing. */
  let showEveStaticCard = $derived(
    eveStaticBusy ||
      eveStaticImportBusy ||
      (!eveStaticLoading &&
        ((eveStaticError !== "" && eveStatic == null) ||
          (eveStatic != null &&
            (eveStatic.sdeCatalogUpdatePending ||
              (!eveStatic.installed && !eveStatic.offerDismissed) ||
              (eveStatic.installed && eveStatic.sdeSqliteTypesCount === 0)))))
  );

  function formatByteSize(n: number): string {
    if (!Number.isFinite(n) || n < 0) return "";
    if (n < 1024) return `${Math.round(n)} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
    return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  }

  function eveStaticProgressBarValue(p: EveStaticDataDownloadProgress | null): number | undefined {
    if (!p) return undefined;
    if (p.phase === "downloading") {
      if (p.totalBytes != null && p.totalBytes > 0) {
        return Math.min(100, (100 * p.bytesReceived) / p.totalBytes);
      }
      return undefined;
    }
    if (p.extractTotal != null && p.extractTotal > 0 && p.extractIndex != null) {
      return Math.min(100, (100 * (p.extractIndex + 1)) / p.extractTotal);
    }
    return undefined;
  }

  function eveStaticProgressDetail(p: EveStaticDataDownloadProgress | null): string {
    if (!p) return "Starting...";
    if (p.phase === "downloading") {
      if (p.totalBytes != null && p.totalBytes > 0) {
        return `${formatByteSize(p.bytesReceived)} / ${formatByteSize(p.totalBytes)}`;
      }
      return formatByteSize(p.bytesReceived) || "0 B";
    }
    if (p.extractTotal != null && p.extractIndex != null) {
      return `Entry ${p.extractIndex + 1} of ${p.extractTotal}`;
    }
    return "Extracting...";
  }

  async function downloadEveStaticData() {
    eveStaticError = "";
    eveStaticProgress = null;
    eveStaticBusy = true;
    try {
      await backend.eveStaticDataDownload();
    } catch (err) {
      eveStaticBusy = false;
      eveStaticProgress = null;
      eveStaticError = err instanceof Error ? err.message : String(err);
    }
  }

  async function dismissEveStaticOffer() {
    try {
      await backend.eveStaticDataDismissOffer();
      eveStatic = await backend.eveStaticDataStatus();
    } catch {
      eveStaticError = userSafeEveStaticErrorMessage();
    }
  }

  async function dismissSdeCatalogNotice() {
    try {
      await backend.eveStaticDataDismissCatalogNotice();
      eveStatic = await backend.eveStaticDataStatus();
    } catch {
      eveStaticError = userSafeEveStaticErrorMessage();
    }
  }

  async function importSdeIntoSqlite() {
    eveStaticError = "";
    eveStaticImportBusy = true;
    try {
      await backend.eveSdeImportFromLocal();
      eveStatic = await backend.eveStaticDataStatus();
    } catch (err) {
      eveStaticError = err instanceof Error ? err.message : String(err);
    } finally {
      eveStaticImportBusy = false;
    }
  }

  onMount(() => {
    const cleanup: Array<() => void> = [];
    void (async () => {
      cleanup.push(
        await listen<EveStaticDataDownloadProgress>("eve-static-data-progress", (event) => {
          const p = event.payload;
          eveStaticProgress = {
            phase: p.phase === "extracting" ? "extracting" : "downloading",
            bytesReceived: p.bytesReceived,
            totalBytes: p.totalBytes ?? null,
            extractIndex: p.extractIndex ?? null,
            extractTotal: p.extractTotal ?? null
          };
        })
      );
      cleanup.push(
        await listen<EveStaticDataDownloadDone>("eve-static-data-download-done", (event) => {
          eveStaticBusy = false;
          eveStaticProgress = null;
          if (event.payload.ok) {
            void (async () => {
              try {
                eveStatic = await backend.eveStaticDataStatus();
              } catch {
                eveStaticError = userSafeEveStaticErrorMessage();
              }
            })();
          } else {
            eveStaticError = event.payload.message ?? "Download failed.";
          }
        })
      );
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
      } catch {}

      try {
        eveStatic = await backend.eveStaticDataStatus();
      } catch {
        eveStaticError = userSafeEveStaticErrorMessage();
      } finally {
        eveStaticLoading = false;
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
      releaseError = "";
      await backend.openExternalUrl(releaseCheck.releaseUrl);
    } catch (err) {
      releaseError = userSafeReleaseErrorMessage();
    }
  }
</script>

<div class="space-y-4">
  {#if releaseCheck?.updateAvailable}
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
        <p class="text-sm">
          New version available:
          <strong class="font-medium text-foreground">{releaseCheck.latestVersion}</strong>
          (current {releaseCheck.currentVersion})
        </p>
        {#if releaseError}
          <p class="mt-2 text-sm text-destructive">{releaseError}</p>
        {/if}
        <Button type="button" class="mt-3" onclick={openLatestReleasePage}>Open Release Page</Button>
      </CardContent>
    </Card>
  {/if}

  {#if showEveStaticCard}
    <Card class="shadow-sm">
      <CardHeader>
        <div class="flex items-start gap-3">
          <HardDriveDownloadIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
          <div>
            <CardTitle class="text-lg font-semibold tracking-tight">EVE static data</CardTitle>
            <CardDescription class="mt-1 text-pretty">
              Local JSONL export from CCP for tools such as the PI Templates viewer (stored next to this application).
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {#if eveStaticBusy}
          <div class="mb-4 space-y-2" aria-live="polite">
            <div class="flex flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground">
              <span class="font-medium text-foreground">
                {eveStaticProgress?.phase === "extracting" ? "Extracting archive" : "Downloading"}
              </span>
              <span>{eveStaticProgressDetail(eveStaticProgress)}</span>
            </div>
            <Progress value={eveStaticProgressBarValue(eveStaticProgress)} max={100} />
          </div>
        {/if}
        {#if eveStaticError && eveStatic == null && !eveStaticLoading}
          <p class="text-sm text-destructive">{eveStaticError}</p>
        {:else if eveStatic}
          {#if eveStatic.sdeCatalogUpdatePending}
            <div class="mb-4 rounded-md border border-border bg-muted/40 p-3">
              <p class="text-sm font-medium text-foreground">New EVE static data available</p>
              <p class="mt-1 text-pretty text-sm text-muted-foreground">
                CCP published a new SDE build. Download the latest archive to refresh your local copy.
              </p>
              <div class="mt-3 flex flex-wrap gap-2">
                <Button
                  type="button"
                  disabled={eveStaticBusy || eveStaticImportBusy}
                  onclick={() => void downloadEveStaticData()}
                >
                  {eveStaticBusy ? "Downloading..." : "Download latest"}
                </Button>
                <Button
                  type="button"
                  variant="secondary"
                  disabled={eveStaticBusy || eveStaticImportBusy}
                  onclick={() => void dismissSdeCatalogNotice()}
                >
                  Dismiss
                </Button>
              </div>
            </div>
          {/if}
          {#if eveStatic.installed && eveStatic.sdeSqliteTypesCount === 0}
            <div class="mb-4 rounded-md border border-border bg-muted/40 p-3">
              <p class="text-sm font-medium text-foreground">SQLite type catalog is empty</p>
              <p class="mt-1 text-pretty text-sm text-muted-foreground">
                Local SDE JSONL is installed, but types and groups are not loaded into the database yet. Import from disk to
                enable PI Templates and other features that resolve type names (this can take a short while).
              </p>
              <div class="mt-3 flex flex-wrap gap-2">
                <Button
                  type="button"
                  disabled={eveStaticBusy || eveStaticImportBusy}
                  onclick={() => void importSdeIntoSqlite()}
                >
                  {eveStaticImportBusy ? "Importing..." : "Import into database"}
                </Button>
              </div>
            </div>
          {/if}
          {#if !eveStatic.installed && !eveStatic.offerDismissed}
            <p class="text-sm text-pretty">
              Download the latest archive from developers.eveonline.com? This may take a while depending on your connection.
            </p>
            <div class="mt-3 flex flex-wrap gap-2">
              <Button
                type="button"
                disabled={eveStaticBusy || eveStaticImportBusy}
                onclick={() => void downloadEveStaticData()}
              >
                {eveStaticBusy ? "Downloading..." : "Yes, download"}
              </Button>
              <Button
                type="button"
                variant="secondary"
                disabled={eveStaticBusy || eveStaticImportBusy}
                onclick={() => void dismissEveStaticOffer()}
              >
                Not now
              </Button>
            </div>
          {/if}
          {#if eveStaticError}
            <p class="mt-2 text-sm text-destructive">{eveStaticError}</p>
          {/if}
        {/if}
      </CardContent>
    </Card>
  {/if}

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
