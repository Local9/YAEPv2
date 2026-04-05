<script lang="ts">
  import type { ThumbnailSetting } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import ListIcon from "@lucide/svelte/icons/list";
  import CopyIcon from "@lucide/svelte/icons/copy";

  interface Props {
    settings: ThumbnailSetting[];
    onSaveBlur: (setting: ThumbnailSetting) => void;
    onCopyFrom: (setting: ThumbnailSetting) => void;
  }

  let { settings, onSaveBlur, onCopyFrom }: Props = $props();
</script>

<div class="mt-4 overflow-x-auto rounded-md border border-border/60">
  <table class="w-full text-sm">
    <thead class="bg-muted/40 text-left">
      <tr>
        <th class="px-3 py-2 font-medium">Thumbnail Title</th>
        <th class="px-3 py-2 font-medium">Width</th>
        <th class="px-3 py-2 font-medium">Height</th>
        <th class="px-3 py-2 font-medium">Character ID</th>
        <th class="px-3 py-2 font-medium w-[1%] whitespace-nowrap">Actions</th>
      </tr>
    </thead>
    <tbody>
      {#if settings.length === 0}
        <tr>
          <td class="px-3 py-2 text-muted-foreground" colspan="5">No overrides saved yet.</td>
        </tr>
      {:else}
        {#each settings as setting (setting.windowTitle)}
          <tr class="border-t">
            <td class="px-3 py-2">
              <div class="flex items-center gap-2">
                <ListIcon class="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
                <code class="rounded bg-muted px-1.5 py-0.5 font-mono text-xs">{setting.windowTitle}</code>
              </div>
            </td>
            <td class="px-3 py-2">
              <Input
                type="number"
                min="1"
                bind:value={setting.config.width}
                onblur={() => onSaveBlur(setting)}
              />
            </td>
            <td class="px-3 py-2">
              <Input
                type="number"
                min="1"
                bind:value={setting.config.height}
                onblur={() => onSaveBlur(setting)}
              />
            </td>
            <td class="px-3 py-2">
              <Input
                type="number"
                min="1"
                bind:value={setting.characterId}
                onblur={() => onSaveBlur(setting)}
                placeholder="Optional"
              />
            </td>
            <td class="px-3 py-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                class="gap-1.5 shrink-0"
                onclick={() => onCopyFrom(setting)}
              >
                <CopyIcon class="size-3.5 shrink-0" aria-hidden="true" />
                Copy from
              </Button>
            </td>
          </tr>
        {/each}
      {/if}
    </tbody>
  </table>
</div>
