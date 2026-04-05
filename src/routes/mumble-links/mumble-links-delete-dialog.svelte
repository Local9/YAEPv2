<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog";
  import { getMumbleLinksPageContext } from "./mumble-links-context";

  const ctl = getMumbleLinksPageContext();
</script>

<AlertDialog.Root bind:open={ctl.confirmOpen}>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Delete {ctl.confirmDelete?.label ?? ""}?</AlertDialog.Title>
      <AlertDialog.Description>
        {#if ctl.confirmDelete?.kind === "folder"}
          This removes the folder, its subfolders, and all links inside. This cannot be undone.
        {:else}
          This removes the link. This cannot be undone.
        {/if}
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel onclick={() => ctl.cancelConfirmDelete()}>Cancel</AlertDialog.Cancel>
      <AlertDialog.Action variant="destructive" onclick={() => void ctl.executeDelete()}
        >Delete</AlertDialog.Action
      >
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
