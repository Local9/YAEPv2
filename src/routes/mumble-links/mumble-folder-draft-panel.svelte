<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import * as Select from "$lib/components/ui/select";
  import { Input } from "$lib/components/ui/input";
  import MumbleFolderIcon from "$lib/mumble/mumble-folder-icon.svelte";
  import { FOLDER_ICON_SELECT_ITEMS } from "$lib/mumble/folder-icon-keys";
  import { getMumbleLinksPageContext } from "./mumble-links-context";

  const ctl = getMumbleLinksPageContext();

  interface Props {
    gid: number;
    parentId: number | null;
    depth: number;
  }

  let { gid, parentId, depth }: Props = $props();
</script>

{#if ctl.folderDraft && ctl.folderDraft.serverGroupId === gid && ctl.folderDraft.parentFolderId === parentId}
  <div
    class="border-border bg-muted/30 mt-2 flex flex-col gap-2 rounded-md border p-3"
    style:margin-left={`${depth * 16 + 8}px`}
  >
    <p class="text-muted-foreground text-xs font-medium">New folder</p>
    <Input
      bind:value={ctl.folderDraftName}
      placeholder="Folder name"
      onkeydown={(e) => ctl.onEnterSubmit(e, () => void ctl.submitFolderDraft())}
    />
    <div class="flex flex-col gap-1.5">
      <span class="text-muted-foreground text-xs font-medium">Folder icon</span>
      <Select.Root type="single" bind:value={ctl.folderDraftIconKey} items={FOLDER_ICON_SELECT_ITEMS}>
        <Select.Trigger class="w-full max-w-xs">
          <span data-slot="select-value" class="flex items-center gap-2">
            <MumbleFolderIcon
              iconKey={ctl.folderDraftIconKey === "" ? null : ctl.folderDraftIconKey}
              class="size-4 shrink-0"
            />
            {FOLDER_ICON_SELECT_ITEMS.find((i) => i.value === ctl.folderDraftIconKey)?.label ??
              "Default"}
          </span>
        </Select.Trigger>
        <Select.Content class="max-h-72">
          {#each FOLDER_ICON_SELECT_ITEMS as item (item.value)}
            <Select.Item value={item.value} label={item.label}>
              <span class="flex items-center gap-2">
                <MumbleFolderIcon
                  iconKey={item.value === "" ? null : item.value}
                  class="size-4 shrink-0"
                />
                {item.label}
              </span>
            </Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>
    </div>
    <div class="flex flex-wrap gap-2">
      <Button type="button" size="sm" onclick={() => void ctl.submitFolderDraft()}>Add folder</Button>
      <Button type="button" variant="outline" size="sm" onclick={() => ctl.cancelFolderDraft()}
        >Cancel</Button
      >
    </div>
  </div>
{/if}
