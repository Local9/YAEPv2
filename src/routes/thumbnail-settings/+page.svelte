<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { Profile, ThumbnailConfig, ThumbnailSetting } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Input } from "$lib/components/ui/input";
  import { toast } from "svelte-sonner";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
  import { Separator } from "$lib/components/ui/separator";
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

  $effect(() => {
    if (saveMessage) toast.success(saveMessage);
  });

  $effect(() => {
    if (error) toast.error(error);
  });
</script>

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <ImageIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Thumbnail Settings</CardTitle>
        <CardDescription>Edit default and per-window-title thumbnail config.</CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
    {#if defaultConfig}
      <div class="mb-2 flex items-center gap-2 text-sm font-medium text-muted-foreground">
        <SlidersHorizontalIcon class="size-4 shrink-0" aria-hidden="true" />
        <span>Default layout</span>
      </div>
      <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
        <Field>
          <FieldLabel class="text-muted-foreground">Width</FieldLabel>
          <FieldContent>
            <Input type="number" bind:value={defaultConfig.width} />
          </FieldContent>
        </Field>
        <Field>
          <FieldLabel class="text-muted-foreground">Height</FieldLabel>
          <FieldContent>
            <Input type="number" bind:value={defaultConfig.height} />
          </FieldContent>
        </Field>
        <Field>
          <FieldLabel class="text-muted-foreground">X</FieldLabel>
          <FieldContent>
            <Input type="number" bind:value={defaultConfig.x} />
          </FieldContent>
        </Field>
        <Field>
          <FieldLabel class="text-muted-foreground">Y</FieldLabel>
          <FieldContent>
            <Input type="number" bind:value={defaultConfig.y} />
          </FieldContent>
        </Field>
        <Field>
          <FieldLabel class="text-muted-foreground">Opacity</FieldLabel>
          <FieldContent>
            <Input
              type="number"
              step="0.05"
              min="0.1"
              max="1"
              bind:value={defaultConfig.opacity}
            />
          </FieldContent>
        </Field>
        <Field>
          <FieldLabel class="text-muted-foreground">Border color</FieldLabel>
          <FieldContent>
            <Input bind:value={defaultConfig.focusBorderColor} />
          </FieldContent>
        </Field>
        <Field>
          <FieldLabel class="text-muted-foreground">Border thickness</FieldLabel>
          <FieldContent>
            <Input type="number" bind:value={defaultConfig.focusBorderThickness} />
          </FieldContent>
        </Field>
        <Field orientation="horizontal" class="cursor-pointer self-end sm:col-span-2 lg:col-span-1">
          <FieldContent>
            <Checkbox bind:checked={defaultConfig.showTitleOverlay} />
          </FieldContent>
          <FieldLabel class="text-muted-foreground">Show title</FieldLabel>
        </Field>
      </div>
      <div class="mt-4 flex flex-wrap items-center gap-2">
        <Button type="button" onclick={saveDefault} class="gap-2">
          <SaveIcon class="size-4 shrink-0" aria-hidden="true" />
          Save default
        </Button>
      </div>
    {/if}

    <Separator class="my-6" orientation="horizontal" />

    <div class="mb-3 flex items-center gap-2 text-sm font-medium text-muted-foreground">
      <BookmarkIcon class="size-4 shrink-0" aria-hidden="true" />
      <h3 class="text-base font-semibold text-foreground">Per-title override</h3>
    </div>
    <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
      <Input
        class="sm:max-w-md sm:flex-1"
        bind:value={windowTitle}
        placeholder="EVE - CharacterName"
      />
      <Button type="button" variant="outline" onclick={addOrUpdateWindowOverride} class="shrink-0 gap-2">
        <SaveIcon class="size-4 shrink-0" aria-hidden="true" />
        Save override from default
      </Button>
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
  </CardContent>
</Card>
