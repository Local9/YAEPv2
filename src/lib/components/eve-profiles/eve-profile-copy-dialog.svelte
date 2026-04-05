<script lang="ts">
  import type { EveDetectedProfile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Dialog from "$lib/components/ui/dialog";

  interface Props {
    open?: boolean;
    profile: EveDetectedProfile | null;
    newName?: string;
    isSubmitting: boolean;
    onSubmit: () => void;
    onOpenChange: (open: boolean) => void;
  }

  let {
    open = $bindable(false),
    profile,
    newName = $bindable(""),
    isSubmitting,
    onSubmit,
    onOpenChange,
  }: Props = $props();
</script>

<Dialog.Root bind:open onOpenChange={onOpenChange}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Copy profile</Dialog.Title>
      <Dialog.Description>
        Copy `{profile?.profileName}` on `{profile?.serverName}` to a new folder/profile name.
      </Dialog.Description>
    </Dialog.Header>
    <div class="grid gap-2">
      <label class="text-muted-foreground text-xs font-medium" for="copy-profile-new-name">
        New folder/profile name
      </label>
      <Input
        id="copy-profile-new-name"
        bind:value={newName}
        placeholder="New profile name"
        onkeydown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            onSubmit();
          }
        }}
      />
    </div>
    <Dialog.Footer>
      <Button type="button" variant="outline" onclick={() => (open = false)}>Cancel</Button>
      <Button
        type="button"
        onclick={onSubmit}
        disabled={!profile || !newName.trim() || isSubmitting}
      >
        {isSubmitting ? "Copying..." : "Copy"}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
