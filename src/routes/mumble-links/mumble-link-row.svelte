<script lang="ts">
  import type { MumbleLink } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
  import LinkIcon from "@lucide/svelte/icons/link";
  import MoreVerticalIcon from "@lucide/svelte/icons/more-vertical";
  import { getMumbleLinksPageContext } from "./mumble-links-context";
  import { HOTKEY_CAPTURE_RING_CLASS, HOTKEY_INPUT_CLASS } from "./mumble-links-styles";

  const ctl = getMumbleLinksPageContext();

  interface Props {
    link: MumbleLink;
    depth: number;
  }

  let { link, depth }: Props = $props();
</script>

<div
  class="border-border flex flex-col gap-2 border-b py-2 sm:flex-row sm:flex-wrap sm:items-center sm:gap-2"
  style:margin-left={`${depth * 16}px`}
>
  <LinkIcon class="text-muted-foreground size-4 shrink-0 sm:mt-0.5" aria-hidden="true" />
  <div class="flex min-w-0 flex-1 flex-wrap items-center gap-2">
    <Input
      class="min-w-32 flex-1 sm:max-w-xs"
      bind:value={link.name}
      onblur={() => void ctl.saveLink(link, { silent: true })}
      onkeydown={(e) => ctl.onEnterSubmit(e, () => void ctl.saveLink(link, { silent: true }))}
    />
    <Input
      class="min-w-48 flex-2 font-mono text-sm"
      bind:value={link.url}
      onblur={() => void ctl.saveLink(link, { silent: true })}
      onkeydown={(e) => ctl.onEnterSubmit(e, () => void ctl.saveLink(link, { silent: true }))}
    />
    <Input
      class="{HOTKEY_INPUT_CLASS} w-36 {ctl.captureHotkeyLinkId === link.id
        ? HOTKEY_CAPTURE_RING_CLASS
        : ''}"
      readonly
      value={link.hotkey}
      placeholder="Hotkey"
      onpointerdown={() => void ctl.startLinkHotkeyCapture(link.id)}
    />
    <Input
      class="w-20 shrink-0"
      type="number"
      bind:value={link.displayOrder}
      onblur={() => void ctl.saveLink(link, { silent: true })}
      onkeydown={(e) => ctl.onEnterSubmit(e, () => void ctl.saveLink(link, { silent: true }))}
    />
  </div>
  <div class="ml-auto flex shrink-0 items-center self-end sm:self-center">
    <DropdownMenu.Root>
      <DropdownMenu.Trigger>
        <Button
          type="button"
          variant="ghost"
          size="icon"
          class="text-muted-foreground size-8"
          aria-label="Link actions"
        >
          <MoreVerticalIcon class="size-4" aria-hidden="true" />
        </Button>
      </DropdownMenu.Trigger>
      <DropdownMenu.Content align="end">
        <DropdownMenu.Item variant="destructive" onclick={() => ctl.openDeleteLink(link)}>
          Delete link
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </div>
</div>
