<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Dialog from "$lib/components/ui/dialog";

  interface Props {
    open?: boolean;
    profileName?: string;
    onCreate: () => void;
    onOpenChange: (open: boolean) => void;
  }

  let {
    open = $bindable(false),
    profileName = $bindable(""),
    onCreate,
    onOpenChange,
  }: Props = $props();
</script>

<Dialog.Root bind:open onOpenChange={onOpenChange}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Create profile</Dialog.Title>
      <Dialog.Description>
        Enter a name for the new profile. Client groups, thumbnail settings, and process rules are
        managed per profile.
      </Dialog.Description>
    </Dialog.Header>
    <div class="grid gap-2">
      <label class="text-muted-foreground text-xs font-medium" for="new-profile-name-dialog">
        Profile name
      </label>
      <Input
        id="new-profile-name-dialog"
        bind:value={profileName}
        placeholder="Profile name"
        onkeydown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            onCreate();
          }
        }}
      />
    </div>
    <Dialog.Footer>
      <Button
        type="button"
        variant="outline"
        onclick={() => {
          open = false;
        }}
      >
        Cancel
      </Button>
      <Button type="button" onclick={() => void onCreate()} disabled={!profileName.trim()}>
        Create
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
