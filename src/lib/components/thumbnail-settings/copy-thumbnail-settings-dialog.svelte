<script lang="ts">
  import type { ThumbnailSetting } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Select from "$lib/components/ui/select";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";

  interface Props {
    open?: boolean;
    targetWindowTitle: string;
    settings: ThumbnailSetting[];
    onApply: (sourceWindowTitle: string) => Promise<void>;
    onDismiss?: () => void;
  }

  let {
    open = $bindable(false),
    targetWindowTitle,
    settings,
    onApply,
    onDismiss,
  }: Props = $props();

  let selectedSourceTitle = $state("");
  let isApplying = $state(false);

  let sourceItems = $derived<{ value: string; label: string }[]>(
    settings
      .filter((s) => s.windowTitle !== targetWindowTitle)
      .map((s) => ({ value: s.windowTitle, label: s.windowTitle })),
  );

  let selectItems = $derived([{ value: "", label: "Select source thumbnail..." }, ...sourceItems]);

  async function confirmCopy() {
    if (!selectedSourceTitle) return;
    isApplying = true;
    try {
      await onApply(selectedSourceTitle);
      open = false;
    } finally {
      isApplying = false;
    }
  }
</script>

<Dialog.Root
  bind:open
  onOpenChange={(next) => {
    if (next) selectedSourceTitle = "";
    if (!next) onDismiss?.();
  }}
>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Copy thumbnail settings</Dialog.Title>
      <Dialog.Description>
        Copy layout and appearance from another thumbnail in this profile onto
        <code class="rounded bg-muted px-1 font-mono text-xs">{targetWindowTitle}</code>. Character ID for
        this row is unchanged.
      </Dialog.Description>
    </Dialog.Header>
    {#if sourceItems.length === 0}
      <p class="text-sm text-muted-foreground">No other thumbnails in this profile to copy from.</p>
    {:else}
      <Field>
        <FieldLabel class="text-muted-foreground">Copy settings from</FieldLabel>
        <FieldContent>
          <Select.Root type="single" bind:value={selectedSourceTitle} items={selectItems}>
            <Select.Trigger class="w-full">
              <span data-slot="select-value">
                {selectedSourceTitle === ""
                  ? "Select source thumbnail..."
                  : selectedSourceTitle}
              </span>
            </Select.Trigger>
            <Select.Content class="max-h-72 overflow-y-auto">
              <Select.Item value="" label="Select source thumbnail...">
                Select source thumbnail...
              </Select.Item>
              {#each sourceItems as item (item.value)}
                <Select.Item value={item.value} label={item.label}>{item.label}</Select.Item>
              {/each}
            </Select.Content>
          </Select.Root>
        </FieldContent>
      </Field>
    {/if}
    <Dialog.Footer>
      <Button type="button" variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button
        type="button"
        onclick={() => void confirmCopy()}
        disabled={!selectedSourceTitle || sourceItems.length === 0 || isApplying}
      >
        {isApplying ? "Applying..." : "Apply copy"}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
