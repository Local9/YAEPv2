<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { Profile, ThumbnailConfig, ThumbnailSetting } from "$models/domain";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import BookmarkIcon from "@lucide/svelte/icons/bookmark";
  import ImageIcon from "@lucide/svelte/icons/image";
  import ListIcon from "@lucide/svelte/icons/list";
  import SaveIcon from "@lucide/svelte/icons/save";
  import SlidersHorizontalIcon from "@lucide/svelte/icons/sliders-horizontal";

  let profiles = $state<Profile[]>([]);
  let activeProfileId = $state<number | null>(null);
  let defaultConfig = $state<ThumbnailConfig | null>(null);
  let settings = $state<ThumbnailSetting[]>([]);
  let windowTitle = $state("");
  let saveMessage = $state("");
  let error = $state("");

  const inputClass =
    "w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground shadow-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring";

  function cloneDefault(): ThumbnailConfig | null {
    if (!defaultConfig) return null;
    return { ...defaultConfig };
  }

  async function refresh() {
    profiles = await backend.getProfiles();
    activeProfileId = profiles.find((p) => p.isActive)?.id ?? null;
    if (activeProfileId == null) return;
    defaultConfig = await backend.getThumbnailDefaultConfig(activeProfileId);
    settings = await backend.getThumbnailSettings(activeProfileId);
  }

  async function saveDefault() {
    if (activeProfileId == null || defaultConfig == null) return;
    try {
      await backend.setThumbnailDefaultConfig(activeProfileId, defaultConfig);
      saveMessage = "Default config saved";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function addOrUpdateWindowOverride() {
    if (activeProfileId == null || !windowTitle.trim()) return;
    const config = cloneDefault();
    if (!config) return;
    try {
      await backend.saveThumbnailSetting(activeProfileId, windowTitle.trim(), config);
      windowTitle = "";
      saveMessage = "Override saved";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  onMount(refresh);
</script>

<section
  class="rounded-lg border border-border bg-card p-4 text-card-foreground shadow-sm"
>
  <div class="mb-4 flex items-start gap-3">
    <ImageIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
    <div>
      <h2 class="text-lg font-semibold tracking-tight">Thumbnail Settings</h2>
      <p class="mt-1 text-sm text-muted-foreground">
        Edit default and per-window-title thumbnail config.
      </p>
    </div>
  </div>

  {#if defaultConfig}
    <div class="mb-2 flex items-center gap-2 text-sm font-medium text-muted-foreground">
      <SlidersHorizontalIcon class="size-4 shrink-0" aria-hidden="true" />
      <span>Default layout</span>
    </div>
    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
      <label class="grid gap-1.5 text-sm font-medium">
        <span class="text-muted-foreground">Width</span>
        <input class={inputClass} type="number" bind:value={defaultConfig.width} />
      </label>
      <label class="grid gap-1.5 text-sm font-medium">
        <span class="text-muted-foreground">Height</span>
        <input class={inputClass} type="number" bind:value={defaultConfig.height} />
      </label>
      <label class="grid gap-1.5 text-sm font-medium">
        <span class="text-muted-foreground">X</span>
        <input class={inputClass} type="number" bind:value={defaultConfig.x} />
      </label>
      <label class="grid gap-1.5 text-sm font-medium">
        <span class="text-muted-foreground">Y</span>
        <input class={inputClass} type="number" bind:value={defaultConfig.y} />
      </label>
      <label class="grid gap-1.5 text-sm font-medium">
        <span class="text-muted-foreground">Opacity</span>
        <input
          class={inputClass}
          type="number"
          step="0.05"
          min="0.1"
          max="1"
          bind:value={defaultConfig.opacity}
        />
      </label>
      <label class="grid gap-1.5 text-sm font-medium">
        <span class="text-muted-foreground">Border color</span>
        <input class={inputClass} bind:value={defaultConfig.focusBorderColor} />
      </label>
      <label class="grid gap-1.5 text-sm font-medium">
        <span class="text-muted-foreground">Border thickness</span>
        <input
          class={inputClass}
          type="number"
          bind:value={defaultConfig.focusBorderThickness}
        />
      </label>
      <label
        class="flex cursor-pointer items-center gap-2 self-end text-sm font-medium sm:col-span-2 lg:col-span-1"
      >
        <input
          class="size-4 rounded border border-input text-primary focus-visible:ring-2 focus-visible:ring-ring"
          type="checkbox"
          bind:checked={defaultConfig.showTitleOverlay}
        />
        <span class="text-muted-foreground">Show title</span>
      </label>
    </div>
    <div class="mt-4 flex flex-wrap items-center gap-2">
      <button
        type="button"
        class="inline-flex items-center gap-2 rounded-md bg-primary px-3 py-2 text-sm font-medium text-primary-foreground shadow-sm transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        onclick={saveDefault}
      >
        <SaveIcon class="size-4 shrink-0" aria-hidden="true" />
        Save default
      </button>
      {#if saveMessage}
        <span class="text-sm text-muted-foreground">{saveMessage}</span>
      {/if}
    </div>
  {/if}

  {#if error}
    <p
      class="mt-4 flex items-start gap-2 rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive"
      role="alert"
    >
      <AlertCircleIcon class="mt-0.5 size-4 shrink-0" aria-hidden="true" />
      <span>{error}</span>
    </p>
  {/if}

  <hr class="my-6 border-t border-border" />

  <div class="mb-3 flex items-center gap-2 text-sm font-medium text-muted-foreground">
    <BookmarkIcon class="size-4 shrink-0" aria-hidden="true" />
    <h3 class="text-base font-semibold text-foreground">Per-title override</h3>
  </div>
  <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
    <input
      class={`${inputClass} sm:max-w-md sm:flex-1`}
      bind:value={windowTitle}
      placeholder="EVE - CharacterName"
    />
    <button
      type="button"
      class="inline-flex shrink-0 items-center justify-center gap-2 rounded-md border border-input bg-background px-3 py-2 text-sm font-medium shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      onclick={addOrUpdateWindowOverride}
    >
      <SaveIcon class="size-4 shrink-0" aria-hidden="true" />
      Save override from default
    </button>
  </div>
  <ul class="mt-4 space-y-2 text-sm">
    {#each settings as setting (setting.windowTitle)}
      <li class="flex items-start gap-2 rounded-md border border-border/60 bg-muted/30 px-3 py-2">
        <ListIcon class="mt-0.5 size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
        <span>
          <code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{setting.windowTitle}</code>
          <span class="ml-2 text-muted-foreground">
            ({setting.config.width}x{setting.config.height})
          </span>
        </span>
      </li>
    {/each}
  </ul>
</section>
