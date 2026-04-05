<script lang="ts">
  import type { ThumbnailSetting } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import BookmarkIcon from "@lucide/svelte/icons/bookmark";
  import SaveIcon from "@lucide/svelte/icons/save";
  import ThumbnailOverridesTable from "./thumbnail-overrides-table.svelte";

  interface Props {
    windowTitle?: string;
    characterIdText?: string;
    settings: ThumbnailSetting[];
    onAddOverride: () => void;
    onSaveBlur: (setting: ThumbnailSetting) => void;
    onCopyFrom: (setting: ThumbnailSetting) => void;
  }

  let {
    windowTitle = $bindable(""),
    characterIdText = $bindable(""),
    settings,
    onAddOverride,
    onSaveBlur,
    onCopyFrom,
  }: Props = $props();
</script>

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
  <Input class="sm:w-56" bind:value={characterIdText} placeholder="Character ID (optional)" />
  <Button type="button" variant="outline" onclick={onAddOverride} class="shrink-0 gap-2">
    <SaveIcon class="size-4 shrink-0" aria-hidden="true" />
    Save override from default
  </Button>
</div>
<p class="mt-2 text-xs text-muted-foreground">
  Character IDs can be found in zKillboard URLs, for example
  <code class="rounded bg-muted px-1">https://zkillboard.com/character/1698894137/</code>.
</p>
<ThumbnailOverridesTable {settings} {onSaveBlur} {onCopyFrom} />
