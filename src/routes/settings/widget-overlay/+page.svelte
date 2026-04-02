<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { backend } from "$services/backend";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Select from "$lib/components/ui/select";
  import { toast } from "svelte-sonner";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle
  } from "$lib/components/ui/card";
  import * as Dialog from "$lib/components/ui/dialog";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
  import SaveIcon from "@lucide/svelte/icons/save";
  import LayoutPanelIcon from "@lucide/svelte/icons/layout-panel-top";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import { Input } from "$lib/components/ui/input";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow
  } from "$lib/components/ui/table";
  import {
    DEFAULT_BROWSER_QUICK_LINKS,
    type MonitorInfoDto,
    type WidgetOverlaySettings
  } from "$models/domain";

  const WIDGET_OVERLAY_TOGGLE_CAPTURE = "widgetOverlayToggle";
  const WIDGET_HOTKEY_INPUT_CLASS = "min-w-[10rem] cursor-pointer select-none";
  const WIDGET_HOTKEY_CAPTURE_RING_CLASS = "ring-ring ring-2 ring-offset-2 ring-offset-background";

  interface HotkeyCapturedPayload {
    value: string;
    captureType: string;
    targetId: number | null;
  }

  let monitors = $state<MonitorInfoDto[]>([]);
  let settings = $state<WidgetOverlaySettings | null>(null);
  let monitorPicker = $state("0");
  /** Select value: empty string means no default auto-open URL. */
  let browserDefaultPicker = $state("");
  let saveStatus = $state("");
  let error = $state("");
  let browserQuickLinksDialogOpen = $state(false);
  let captureWidgetToggleHotkey = $state(false);

  const monitorSelectItems = $derived(
    monitors.map((m) => ({
      value: String(m.index),
      label: `${m.name || `Display ${m.index}`}${m.isPrimary ? " (primary)" : ""}`
    }))
  );

  const monitorTriggerLabel = $derived.by(() => {
    const m = monitors.find((x) => x.index === Number.parseInt(monitorPicker, 10));
    if (!m) return "Select a display";
    return `${m.name || `Display ${m.index}`}${m.isPrimary ? " (primary)" : ""}`;
  });

  const browserDefaultItems = $derived(
    settings
      ? [
          { value: "", label: "None (show shortcuts when the saved page URL is empty)" },
          ...settings.browserQuickLinks.map((l) => ({
            value: l.url.trim(),
            label: l.title.trim() || l.url.trim()
          }))
        ]
      : []
  );

  const browserDefaultTriggerLabel = $derived.by(() => {
    if (!settings) return "";
    const v = browserDefaultPicker.trim();
    if (!v) return "None (show shortcuts when the saved page URL is empty)";
    const hit = settings.browserQuickLinks.find((l) => l.url.trim() === v);
    return hit ? hit.title : "None (show shortcuts when the saved page URL is empty)";
  });

  function formatMonitorLabel(m: MonitorInfoDto): string {
    return `${m.name || `Display ${m.index}`}${m.isPrimary ? " (primary)" : ""}`;
  }

  function userSafeErrorMessage(): string {
    return "Unable to save widget overlay settings right now. Please try again.";
  }

  function normalizeLoadedSettings(loaded: WidgetOverlaySettings): WidgetOverlaySettings {
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
        mumbleLinks: loaded.layout.mumbleLinks ?? { x: 24, y: 520, width: 200, height: 88 }
      }
    };
  }

  function isEscapeHotkeyValue(rawHotkey: string): boolean {
    const tokens = rawHotkey
      .split("+")
      .map((token) => token.trim().toLowerCase())
      .filter((token) => token.length > 0);
    if (tokens.length === 0) return false;
    const keyToken = tokens[tokens.length - 1];
    return keyToken === "escape" || keyToken === "esc";
  }

  async function onWidgetToggleHotkeyPointerDown() {
    if (!settings?.enabled) return;
    captureWidgetToggleHotkey = true;
    error = "";
    try {
      await backend.hotkeysCaptureStart(WIDGET_OVERLAY_TOGGLE_CAPTURE);
    } catch {
      error = userSafeErrorMessage();
      captureWidgetToggleHotkey = false;
    }
  }

  function stopWidgetToggleHotkeyCapture() {
    captureWidgetToggleHotkey = false;
    void backend.hotkeysCaptureStop();
  }

  async function refresh() {
    try {
      monitors = await backend.listMonitors();
      const raw = await backend.widgetOverlayGetSettings();
      settings = normalizeLoadedSettings(raw);
      if (settings) {
        monitorPicker = String(settings.monitorIndex);
        browserDefaultPicker = settings.browserDefaultUrl ?? "";
      }
      error = "";
    } catch {
      error = userSafeErrorMessage();
    }
  }

  function mergeSettingsForPersist(): WidgetOverlaySettings | null {
    if (!settings) return null;
    const parsed = Number.parseInt(monitorPicker, 10);
    let browserDefaultUrl = browserDefaultPicker.trim() || null;
    if (
      browserDefaultUrl &&
      !settings.browserQuickLinks.some((l) => l.url.trim() === browserDefaultUrl)
    ) {
      browserDefaultUrl = null;
    }
    return {
      ...settings,
      monitorIndex: Number.isFinite(parsed) ? parsed : settings.monitorIndex,
      browserDefaultUrl
    };
  }

  /** Persists current editor state to SQLite and syncs the overlay window. */
  async function persistToBackend(options?: { successToast?: boolean }): Promise<boolean> {
    const merged = mergeSettingsForPersist();
    if (!merged) return false;
    try {
      await backend.widgetOverlaySaveSettings(merged);
      settings = merged;
      browserDefaultPicker = merged.browserDefaultUrl ?? "";
      error = "";
      if (options?.successToast) {
        saveStatus = "Widget overlay settings saved";
      }
      return true;
    } catch {
      error = userSafeErrorMessage();
      await refresh();
      return false;
    }
  }

  async function save() {
    await persistToBackend({ successToast: true });
  }

  async function onBrowserWidgetEnabledChange(v: boolean | "indeterminate") {
    if (!settings || v === "indeterminate") return;
    settings = { ...settings, showBrowserWidget: v === true };
    await persistToBackend({ successToast: false });
  }

  async function onBrowserAlwaysDisplayedChange(v: boolean | "indeterminate") {
    if (!settings || v === "indeterminate") return;
    settings = { ...settings, browserAlwaysDisplayed: v === true };
    await persistToBackend({ successToast: false });
  }

  async function onFleetMotdWidgetEnabledChange(v: boolean | "indeterminate") {
    if (!settings || v === "indeterminate") return;
    settings = { ...settings, showFleetMotdWidget: v === true };
    await persistToBackend({ successToast: false });
  }

  async function onFleetMotdAlwaysDisplayedChange(v: boolean | "indeterminate") {
    if (!settings || v === "indeterminate") return;
    settings = { ...settings, fleetMotdAlwaysDisplayed: v === true };
    await persistToBackend({ successToast: false });
  }

  async function onIntelFeedWidgetEnabledChange(v: boolean | "indeterminate") {
    if (!settings || v === "indeterminate") return;
    settings = { ...settings, showIntelFeedWidget: v === true };
    await persistToBackend({ successToast: false });
  }

  async function onIntelFeedAlwaysDisplayedChange(v: boolean | "indeterminate") {
    if (!settings || v === "indeterminate") return;
    settings = { ...settings, intelFeedAlwaysDisplayed: v === true };
    await persistToBackend({ successToast: false });
  }

  async function onMumbleLinksWidgetEnabledChange(v: boolean | "indeterminate") {
    if (!settings || v === "indeterminate") return;
    settings = { ...settings, showMumbleLinksWidget: v === true };
    await persistToBackend({ successToast: false });
  }

  async function onMumbleLinksAlwaysDisplayedChange(v: boolean | "indeterminate") {
    if (!settings || v === "indeterminate") return;
    settings = { ...settings, mumbleLinksAlwaysDisplayed: v === true };
    await persistToBackend({ successToast: false });
  }

  function addQuickLink() {
    if (!settings) return;
    settings = {
      ...settings,
      browserQuickLinks: [
        ...settings.browserQuickLinks,
        {
          id: `link-${globalThis.crypto.randomUUID()}`,
          url: "https://",
          title: "New shortcut"
        }
      ]
    };
  }

  function removeQuickLink(id: string) {
    if (!settings) return;
    const removed = settings.browserQuickLinks.find((l) => l.id === id);
    const next = settings.browserQuickLinks.filter((l) => l.id !== id);
    let def = settings.browserDefaultUrl;
    const rm = removed?.url.trim() ?? "";
    if (removed && def && def.trim() === rm) {
      def = null;
      browserDefaultPicker = "";
    }
    settings = { ...settings, browserQuickLinks: next, browserDefaultUrl: def };
  }

  onMount(() => {
    void refresh();
    let unlistenHotkeyCaptured: UnlistenFn | undefined;
    void listen<HotkeyCapturedPayload>("hotkeyCaptured", (event) => {
      const payload = event.payload;
      if (payload.captureType !== WIDGET_OVERLAY_TOGGLE_CAPTURE) return;
      if (payload.targetId != null) return;
      stopWidgetToggleHotkeyCapture();
      if (!settings) return;
      if (payload.value.trim() === "" || isEscapeHotkeyValue(payload.value)) {
        settings = { ...settings, toggleHotkey: "" };
        void persistToBackend({ successToast: false });
        return;
      }
      settings = { ...settings, toggleHotkey: payload.value };
      void persistToBackend({ successToast: false });
    }).then((u) => {
      unlistenHotkeyCaptured = u;
    });
    return () => {
      unlistenHotkeyCaptured?.();
      stopWidgetToggleHotkeyCapture();
    };
  });

  $effect(() => {
    if (!settings) return;
    const p = browserDefaultPicker.trim();
    if (p && !settings.browserQuickLinks.some((l) => l.url.trim() === p)) {
      browserDefaultPicker = "";
      settings = { ...settings, browserDefaultUrl: null };
    }
  });

  $effect(() => {
    if (settings && !settings.showBrowserWidget) {
      browserQuickLinksDialogOpen = false;
    }
  });

  $effect(() => {
    if (saveStatus) toast.success(saveStatus);
  });

  $effect(() => {
    if (error) toast.error(error);
  });
</script>

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <LayoutPanelIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Widget overlay</CardTitle>
        <CardDescription>
          Fullscreen transparent overlay for draggable widgets. Choose a display and which widgets appear. The overlay
          stays above other windows; only widget regions receive clicks (Windows). Use the tray command &quot;Toggle
          widget visibility&quot; or the hotkey below to hide or show widgets while keeping the overlay window open.
        </CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
    {#if settings}
      <div class="mt-4 grid max-w-4xl gap-4">
        <Field>
          <FieldLabel>Enable overlay</FieldLabel>
          <FieldContent class="flex flex-row items-center gap-2">
            <Checkbox
              class="cursor-pointer"
              checked={settings.enabled}
              onCheckedChange={(v) => {
                const en = v === true;
                settings = { ...settings!, enabled: en, visible: en ? true : false };
              }}
            />
            <span class="text-sm text-muted-foreground">Turn on the widget overlay feature (required for it to appear)</span>
          </FieldContent>
        </Field>

        {#if settings.enabled}
          <Field>
            <FieldLabel>Show overlay window</FieldLabel>
            <FieldContent class="flex flex-row items-center gap-2">
              <Checkbox
                class="cursor-pointer"
                checked={settings.visible}
                onCheckedChange={(v) => {
                  settings = { ...settings!, visible: v === true };
                }}
              />
              <span class="text-sm text-muted-foreground">Whether the overlay window is shown (saved with Save)</span>
            </FieldContent>
          </Field>

          <Field>
            <FieldLabel>Toggle widget visibility hotkey</FieldLabel>
            <FieldContent class="flex max-w-md flex-col gap-2">
              <Input
                class="{WIDGET_HOTKEY_INPUT_CLASS} {captureWidgetToggleHotkey
                  ? WIDGET_HOTKEY_CAPTURE_RING_CLASS
                  : ''}"
                readonly
                autocomplete="off"
                spellcheck={false}
                inputmode="none"
                aria-readonly="true"
                bind:value={settings.toggleHotkey}
                placeholder={captureWidgetToggleHotkey
                  ? "Press chord, release key…"
                  : "Click here, then press keys"}
                onpointerdown={() => void onWidgetToggleHotkeyPointerDown()}
                onkeydown={(e) => {
                  if (e.key !== "Escape") return;
                  e.preventDefault();
                  if (captureWidgetToggleHotkey) {
                    stopWidgetToggleHotkeyCapture();
                  }
                  settings = { ...settings!, toggleHotkey: "" };
                  void persistToBackend({ successToast: false });
                }}
                onpaste={(e) => e.preventDefault()}
                onblur={(e) => {
                  if (captureWidgetToggleHotkey) {
                    stopWidgetToggleHotkeyCapture();
                    return;
                  }
                  settings = { ...settings!, toggleHotkey: (e.currentTarget as HTMLInputElement).value.trim() };
                  void persistToBackend({ successToast: false });
                }}
              />
              <p class="text-xs text-muted-foreground">
                Hides or shows widgets while the overlay window stays open. Clear the field or press Escape while
                capturing to remove the hotkey. Saved when you leave the field or finish capture.
              </p>
            </FieldContent>
          </Field>
        {/if}

        <Field>
          <FieldLabel>Display</FieldLabel>
          <FieldContent>
            <Select.Root type="single" bind:value={monitorPicker} items={monitorSelectItems}>
              <Select.Trigger class="w-full max-w-md">
                <span data-slot="select-value">{monitorTriggerLabel}</span>
              </Select.Trigger>
              <Select.Content>
                {#each monitors as m (m.index)}
                  <Select.Item value={String(m.index)} label={formatMonitorLabel(m)}>
                    {formatMonitorLabel(m)}
                  </Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
            <p class="mt-1 text-xs text-muted-foreground">The overlay spans the full monitor rectangle.</p>
          </FieldContent>
        </Field>

        <div class="flex flex-wrap items-center gap-2">
          <Button type="button" onclick={() => void save()}>
            <SaveIcon class="size-4" aria-hidden="true" />
            Save
          </Button>
        </div>

        <Field>
          <FieldLabel>Widgets</FieldLabel>
          <FieldContent class="flex flex-col gap-2">
            <p class="text-xs text-muted-foreground">
              Enable each widget to show or hide it on the overlay immediately. &quot;Always displayed&quot; keeps that
              widget visible when widget visibility is toggled off (hotkey or tray). Use Configuration to edit options
              where available.
            </p>
            <div class="max-w-full overflow-x-auto rounded-md border border-border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead class="w-24">Enabled</TableHead>
                    <TableHead class="w-36">Always displayed</TableHead>
                    <TableHead class="min-w-40">Widget name</TableHead>
                    <TableHead class="min-w-48">Configuration</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell class="align-middle">
                      <Checkbox
                        class="cursor-pointer"
                        checked={settings.showBrowserWidget}
                        onCheckedChange={(v) => void onBrowserWidgetEnabledChange(v)}
                      />
                    </TableCell>
                    <TableCell class="align-middle">
                      <Checkbox
                        class="cursor-pointer"
                        checked={settings.browserAlwaysDisplayed}
                        disabled={!settings.showBrowserWidget}
                        onCheckedChange={(v) => void onBrowserAlwaysDisplayedChange(v)}
                      />
                    </TableCell>
                    <TableCell class="align-middle font-medium">Browser</TableCell>
                    <TableCell class="align-middle">
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        disabled={!settings.showBrowserWidget}
                        onclick={() => {
                          browserQuickLinksDialogOpen = true;
                        }}
                      >
                        Edit quick links…
                      </Button>
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="align-middle">
                      <Checkbox
                        class="cursor-pointer"
                        checked={settings.showFleetMotdWidget}
                        onCheckedChange={(v) => void onFleetMotdWidgetEnabledChange(v)}
                      />
                    </TableCell>
                    <TableCell class="align-middle">
                      <Checkbox
                        class="cursor-pointer"
                        checked={settings.fleetMotdAlwaysDisplayed}
                        disabled={!settings.showFleetMotdWidget}
                        onCheckedChange={(v) => void onFleetMotdAlwaysDisplayedChange(v)}
                      />
                    </TableCell>
                    <TableCell class="align-middle font-medium">Fleet MOTD</TableCell>
                    <TableCell class="align-middle text-muted-foreground">
                      Built-in widget
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="align-middle">
                      <Checkbox
                        class="cursor-pointer"
                        checked={settings.showIntelFeedWidget}
                        onCheckedChange={(v) => void onIntelFeedWidgetEnabledChange(v)}
                      />
                    </TableCell>
                    <TableCell class="align-middle">
                      <Checkbox
                        class="cursor-pointer"
                        checked={settings.intelFeedAlwaysDisplayed}
                        disabled={!settings.showIntelFeedWidget}
                        onCheckedChange={(v) => void onIntelFeedAlwaysDisplayedChange(v)}
                      />
                    </TableCell>
                    <TableCell class="align-middle font-medium">Intel Feed</TableCell>
                    <TableCell class="align-middle text-muted-foreground">
                      Shows last 50 intel lines (newest at bottom)
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell class="align-middle">
                      <Checkbox
                        class="cursor-pointer"
                        checked={settings.showMumbleLinksWidget}
                        onCheckedChange={(v) => void onMumbleLinksWidgetEnabledChange(v)}
                      />
                    </TableCell>
                    <TableCell class="align-middle">
                      <Checkbox
                        class="cursor-pointer"
                        checked={settings.mumbleLinksAlwaysDisplayed}
                        disabled={!settings.showMumbleLinksWidget}
                        onCheckedChange={(v) => void onMumbleLinksAlwaysDisplayedChange(v)}
                      />
                    </TableCell>
                    <TableCell class="align-middle font-medium">Mumble links</TableCell>
                    <TableCell class="align-middle">
                      <Button type="button" variant="outline" size="sm" onclick={() => goto("/mumble-links")}>
                        Edit links…
                      </Button>
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </div>
          </FieldContent>
        </Field>

        <Dialog.Root bind:open={browserQuickLinksDialogOpen}>
          <Dialog.Content class="flex max-h-[min(90vh,720px)] max-w-2xl flex-col gap-0 p-0 sm:max-w-2xl">
            <Dialog.Header class="shrink-0 px-6 pt-6">
              <Dialog.Title>Browser widget</Dialog.Title>
              <Dialog.Description>
                Quick links and default page. Save widget overlay settings on the main page to apply changes.
              </Dialog.Description>
            </Dialog.Header>
            <div class="min-h-0 flex-1 overflow-y-auto px-6 pb-2">
              <div class="grid gap-4 py-2">
                <Field>
                  <FieldLabel>Open by default (optional)</FieldLabel>
                  <FieldContent>
                    <Select.Root type="single" bind:value={browserDefaultPicker} items={browserDefaultItems}>
                      <Select.Trigger class="w-full max-w-md">
                        <span data-slot="select-value">{browserDefaultTriggerLabel}</span>
                      </Select.Trigger>
                      <Select.Content>
                        {#each browserDefaultItems as item (item.value === "" ? "none" : item.value)}
                          <Select.Item value={item.value} label={item.label}>
                            {item.label}
                          </Select.Item>
                        {/each}
                      </Select.Content>
                    </Select.Root>
                    <p class="mt-1 text-xs text-muted-foreground">
                      If set, this URL loads when the overlay opens while the saved browser URL is empty. It does not
                      replace the shortcut list; Home still returns there.
                    </p>
                  </FieldContent>
                </Field>

                <Field>
                  <FieldLabel>Quick links</FieldLabel>
                  <FieldContent class="flex flex-col gap-3">
                    <p class="text-xs text-muted-foreground">
                      Shortcuts when no page is open. Row order is top to bottom (then left to right in the home grid).
                    </p>
                    <div class="max-w-full overflow-x-auto rounded-md border border-border">
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead class="min-w-32">Title</TableHead>
                            <TableHead class="min-w-48">URL</TableHead>
                            <TableHead class="w-18 text-right">Remove</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {#each settings.browserQuickLinks as link (link.id)}
                            <TableRow>
                              <TableCell class="align-middle">
                                <Input
                                  class="h-9 max-w-md"
                                  type="text"
                                  bind:value={link.title}
                                  placeholder="Label"
                                />
                              </TableCell>
                              <TableCell class="align-middle">
                                <Input
                                  class="h-9 max-w-xl font-mono text-xs"
                                  type="text"
                                  bind:value={link.url}
                                  placeholder="https://"
                                  spellcheck={false}
                                />
                              </TableCell>
                              <TableCell class="align-middle text-right">
                                <Button
                                  type="button"
                                  variant="outline"
                                  size="icon"
                                  class="shrink-0"
                                  title="Remove shortcut"
                                  aria-label="Remove shortcut"
                                  onclick={() => removeQuickLink(link.id)}
                                  disabled={settings.browserQuickLinks.length <= 1}
                                >
                                  <Trash2Icon class="size-4" aria-hidden="true" />
                                </Button>
                              </TableCell>
                            </TableRow>
                          {/each}
                        </TableBody>
                      </Table>
                    </div>
                    <Button type="button" variant="secondary" class="w-fit gap-2" onclick={addQuickLink}>
                      <PlusIcon class="size-4" aria-hidden="true" />
                      Add shortcut
                    </Button>
                  </FieldContent>
                </Field>
              </div>
            </div>
            <Dialog.Footer class="shrink-0 border-t px-6 py-4">
              <Dialog.Close>
                {#snippet child({ props })}
                  <Button type="button" variant="secondary" {...props}>Close</Button>
                {/snippet}
              </Dialog.Close>
            </Dialog.Footer>
          </Dialog.Content>
        </Dialog.Root>
      </div>
    {:else}
      <p class="text-sm text-muted-foreground">Loading…</p>
    {/if}
  </CardContent>
</Card>
