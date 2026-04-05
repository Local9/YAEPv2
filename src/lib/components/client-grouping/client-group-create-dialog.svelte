<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Dialog from "$lib/components/ui/dialog";
  import PlusIcon from "@lucide/svelte/icons/plus";

  interface Props {
    open?: boolean;
    groupName?: string;
    onCreate: () => void;
    onOpenChange: (open: boolean) => void;
  }

  let {
    open = $bindable(false),
    groupName = $bindable(""),
    onCreate,
    onOpenChange,
  }: Props = $props();
</script>

<Dialog.Root bind:open onOpenChange={onOpenChange}>
  <Dialog.Content class="sm:max-w-md">
    <Dialog.Header>
      <Dialog.Title>Create group</Dialog.Title>
      <Dialog.Description>
        Enter a name for the new client group. Clients can be added after it is created.
      </Dialog.Description>
    </Dialog.Header>
    <div class="grid gap-2">
      <label class="text-muted-foreground text-xs font-medium" for="new-group-name-dialog">
        Group name
      </label>
      <Input
        id="new-group-name-dialog"
        bind:value={groupName}
        placeholder="Group name"
        onkeydown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
            onCreate();
          }
        }}
      />
    </div>
    <Dialog.Footer>
      <Dialog.Close>
        {#snippet child({ props })}
          <Button variant="outline" {...props}>Cancel</Button>
        {/snippet}
      </Dialog.Close>
      <Button type="button" onclick={() => void onCreate()}>
        <PlusIcon class="size-4 shrink-0" aria-hidden="true" />
        Create
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
