<script lang="ts">
  import type { EveDetectedProfile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";
  import * as Select from "$lib/components/ui/select";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";

  interface Props {
    open?: boolean;
    profile: EveDetectedProfile | null;
    characterId?: string;
    userId?: string;
    characterItems: { value: string; label: string }[];
    userItems: { value: string; label: string }[];
    isLoadingSources: boolean;
    isSubmitting: boolean;
    onSubmit: () => void;
    onOpenChange: (open: boolean) => void;
  }

  let {
    open = $bindable(false),
    profile,
    characterId = $bindable(""),
    userId = $bindable(""),
    characterItems,
    userItems,
    isLoadingSources,
    isSubmitting,
    onSubmit,
    onOpenChange,
  }: Props = $props();
</script>

<Dialog.Root bind:open onOpenChange={onOpenChange}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Copy profile settings</Dialog.Title>
      <Dialog.Description>
        Select source character and source user for `{profile?.profileName}` on `{profile?.serverName}`.
      </Dialog.Description>
    </Dialog.Header>
    {#if isLoadingSources}
      <p class="text-sm text-muted-foreground">Loading settings sources...</p>
    {:else}
      <div class="grid gap-4">
        <Field>
          <FieldLabel class="text-muted-foreground">Source character</FieldLabel>
          <FieldContent>
            <Select.Root
              type="single"
              bind:value={characterId}
              items={characterItems}
              disabled={characterItems.length === 0}
            >
              <Select.Trigger class="w-full">
                <span data-slot="select-value">
                  {characterId || "Select source character"}
                </span>
              </Select.Trigger>
              <Select.Content>
                {#each characterItems as item (item.value)}
                  <Select.Item value={item.value} label={item.label}>{item.label}</Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
            {#if characterItems.length === 0}
              <p class="mt-1 text-xs text-muted-foreground">No character settings sources found.</p>
            {/if}
          </FieldContent>
        </Field>

        <Field>
          <FieldLabel class="text-muted-foreground">Source user</FieldLabel>
          <FieldContent>
            <Select.Root
              type="single"
              bind:value={userId}
              items={userItems}
              disabled={userItems.length === 0}
            >
              <Select.Trigger class="w-full">
                <span data-slot="select-value">{userId || "Select source user"}</span>
              </Select.Trigger>
              <Select.Content>
                {#each userItems as item (item.value)}
                  <Select.Item value={item.value} label={item.label}>{item.label}</Select.Item>
                {/each}
              </Select.Content>
            </Select.Root>
            {#if userItems.length === 0}
              <p class="mt-1 text-xs text-muted-foreground">No user settings sources found.</p>
            {/if}
          </FieldContent>
        </Field>
      </div>
    {/if}
    <Dialog.Footer>
      <Button type="button" variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button
        type="button"
        onclick={onSubmit}
        disabled={
          !profile ||
          !characterId ||
          !userId ||
          isLoadingSources ||
          isSubmitting
        }
      >
        {isSubmitting ? "Copying..." : "Copy Settings"}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
