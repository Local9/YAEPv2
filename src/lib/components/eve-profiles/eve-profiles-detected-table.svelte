<script lang="ts">
  import type { EveDetectedProfile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";
  import CopyIcon from "@lucide/svelte/icons/copy";
  import Settings2Icon from "@lucide/svelte/icons/settings-2";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";

  interface Props {
    profiles: EveDetectedProfile[];
    onCopy: (profile: EveDetectedProfile) => void;
    onCopySettings: (profile: EveDetectedProfile) => void;
    onDelete: (profile: EveDetectedProfile) => void;
  }

  let { profiles, onCopy, onCopySettings, onDelete }: Props = $props();
</script>

<div class="mt-3 overflow-x-auto">
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>Profile</TableHead>
        <TableHead>Actions</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      {#if profiles.length === 0}
        <TableRow>
          <TableCell colspan={2} class="text-muted-foreground">
            No profiles detected under %LOCALAPPDATA%\CCP\EVE
          </TableCell>
        </TableRow>
      {:else}
        {#each profiles as profile (profile.fullPath)}
          <TableRow>
            <TableCell>{profile.profileName}</TableCell>
            <TableCell>
              <div class="flex flex-wrap gap-2">
                <Button type="button" variant="outline" class="gap-1.5" onclick={() => onCopy(profile)}>
                  <CopyIcon class="size-4 shrink-0" aria-hidden="true" />
                  Copy
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  class="gap-1.5"
                  onclick={() => void onCopySettings(profile)}
                >
                  <Settings2Icon class="size-4 shrink-0" aria-hidden="true" />
                  Copy Settings
                </Button>
                <Button type="button" variant="destructive" class="gap-1.5" onclick={() => onDelete(profile)}>
                  <Trash2Icon class="size-4 shrink-0" aria-hidden="true" />
                  Delete
                </Button>
              </div>
            </TableCell>
          </TableRow>
        {/each}
      {/if}
    </TableBody>
  </Table>
</div>
