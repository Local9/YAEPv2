<script lang="ts">
  import type { EveDetectedProfile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import * as Dialog from "$lib/components/ui/dialog";

  interface Props {
    open?: boolean;
    profile: EveDetectedProfile | null;
    isSubmitting: boolean;
    onSubmit: () => void;
    onOpenChange: (open: boolean) => void;
  }

  let {
    open = $bindable(false),
    profile,
    isSubmitting,
    onSubmit,
    onOpenChange,
  }: Props = $props();
</script>

<Dialog.Root bind:open onOpenChange={onOpenChange}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Delete profile</Dialog.Title>
      <Dialog.Description>
        Delete `{profile?.profileName}` on `{profile?.serverName}`.
      </Dialog.Description>
    </Dialog.Header>
    <Dialog.Footer>
      <Button type="button" variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button
        type="button"
        variant="destructive"
        onclick={onSubmit}
        disabled={!profile || isSubmitting}
      >
        {isSubmitting ? "Deleting..." : "Delete"}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
