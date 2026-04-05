<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import * as Select from "$lib/components/ui/select";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";

  interface Props {
    selectedServer: string;
    serverSelectItems: { value: string; label: string }[];
    normalizeServerName: (name: string) => string;
    isSubmittingBackup: boolean;
    onBackup: () => void;
  }

  let {
    selectedServer = $bindable(),
    serverSelectItems,
    normalizeServerName,
    isSubmittingBackup,
    onBackup,
  }: Props = $props();
</script>

<div class="mt-6 flex items-center gap-2 text-sm font-medium text-muted-foreground">
  <FolderOpenIcon class="size-4 shrink-0" aria-hidden="true" />
  <h3 class="text-base font-semibold text-foreground">Detected Profiles</h3>
</div>
<div class="mt-3 flex flex-wrap items-end gap-2">
  <div class="max-w-sm flex-1">
    <Field>
      <FieldLabel class="text-muted-foreground">Server</FieldLabel>
      <FieldContent>
        <Select.Root type="single" bind:value={selectedServer} items={serverSelectItems}>
          <Select.Trigger class="w-full">
            <span data-slot="select-value">
              {selectedServer ? normalizeServerName(selectedServer) : "Select server"}
            </span>
          </Select.Trigger>
          <Select.Content>
            {#each serverSelectItems as item (item.value)}
              <Select.Item value={item.value} label={item.label}>{item.label}</Select.Item>
            {/each}
          </Select.Content>
        </Select.Root>
      </FieldContent>
    </Field>
  </div>
  <Button type="button" onclick={() => void onBackup()} disabled={!selectedServer || isSubmittingBackup}>
    {isSubmittingBackup ? "Backing up..." : "Back up"}
  </Button>
</div>
