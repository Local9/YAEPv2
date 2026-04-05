<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { getMumbleLinksPageContext } from "./mumble-links-context";

  const ctl = getMumbleLinksPageContext();

  interface Props {
    gid: number;
    fid: number | null;
  }

  let { gid, fid }: Props = $props();
</script>

{#if ctl.linkDraft && ctl.linkDraft.serverGroupId === gid && ctl.linkDraft.folderId === fid}
  <div
    class="border-border bg-muted/30 mt-2 flex flex-col gap-2 rounded-md border p-3"
    style:margin-left={`${(fid ? 1 : 0) * 12 + 8}px`}
  >
    <p class="text-muted-foreground text-xs font-medium">New link</p>
    <Input
      class="font-mono text-sm"
      placeholder="mumble://..."
      value={ctl.linkDraftUrl}
      oninput={(e) => ctl.onLinkDraftUrlInput((e.currentTarget as HTMLInputElement).value)}
      onkeydown={(e) => ctl.onEnterSubmit(e, () => void ctl.submitLinkDraft())}
    />
    <Input
      bind:value={ctl.linkDraftName}
      placeholder="Display name"
      onkeydown={(e) => ctl.onEnterSubmit(e, () => void ctl.submitLinkDraft())}
    />
    <div class="flex flex-wrap gap-2">
      <Button type="button" size="sm" onclick={() => void ctl.submitLinkDraft()}>Add link</Button>
      <Button type="button" variant="outline" size="sm" onclick={() => ctl.cancelLinkDraft()}
        >Cancel</Button
      >
    </div>
  </div>
{/if}
