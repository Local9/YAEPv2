<script lang="ts">
  import type { Profile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";
  import CheckIcon from "@lucide/svelte/icons/check";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import {
    PROFILE_HOTKEY_CAPTURE_RING_CLASS,
    PROFILE_HOTKEY_INPUT_CLASS,
  } from "./profile-hotkeys";

  interface Props {
    profiles: Profile[];
    clientGroupCounts: Record<number, number>;
    isCapturingProfileHotkey: (profileId: number) => boolean;
    onProfileSwitchHotkeyPointerDown: (profile: Profile) => void;
    onProfileHotkeyKeydown: (profile: Profile, e: KeyboardEvent) => void;
    onProfileHotkeyBlur: (profile: Profile, e: FocusEvent) => void;
    onSetActive: (profileId: number) => void;
    onRemoveProfile: (profileId: number) => void;
  }

  let {
    profiles,
    clientGroupCounts,
    isCapturingProfileHotkey,
    onProfileSwitchHotkeyPointerDown,
    onProfileHotkeyKeydown,
    onProfileHotkeyBlur,
    onSetActive,
    onRemoveProfile,
  }: Props = $props();
</script>

<div class="mt-4 overflow-x-auto">
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead>Name</TableHead>
        <TableHead>Client groups</TableHead>
        <TableHead>Hotkey</TableHead>
        <TableHead>Active</TableHead>
        <TableHead>Actions</TableHead>
      </TableRow>
    </TableHeader>
    <TableBody>
      {#each profiles as profile (profile.id)}
        <TableRow>
          <TableCell>{profile.name}</TableCell>
          <TableCell>{clientGroupCounts[profile.id] ?? 0}</TableCell>
          <TableCell>
            <div class="flex min-w-48 items-center gap-2">
              <Input
                class="{PROFILE_HOTKEY_INPUT_CLASS} {isCapturingProfileHotkey(profile.id)
                  ? PROFILE_HOTKEY_CAPTURE_RING_CLASS
                  : ''}"
                readonly
                autocomplete="off"
                spellcheck={false}
                inputmode="none"
                aria-readonly="true"
                bind:value={profile.switchHotkey}
                placeholder={isCapturingProfileHotkey(profile.id)
                  ? "Press chord, release key…"
                  : "Click here, then press keys"}
                onpointerdown={() => void onProfileSwitchHotkeyPointerDown(profile)}
                onkeydown={(e) => onProfileHotkeyKeydown(profile, e)}
                onpaste={(e) => e.preventDefault()}
                onblur={(e) => onProfileHotkeyBlur(profile, e)}
              />
            </div>
          </TableCell>
          <TableCell>{profile.isActive ? "Yes" : "No"}</TableCell>
          <TableCell>
            <div class="flex flex-wrap gap-2">
              <Button
                type="button"
                variant="outline"
                onclick={() => onSetActive(profile.id)}
                disabled={profile.isActive}
              >
                <CheckIcon class="size-4 shrink-0" aria-hidden="true" />
                Set Active
              </Button>
              <Button
                type="button"
                variant="destructive"
                onclick={() => onRemoveProfile(profile.id)}
                disabled={profile.isActive}
              >
                <Trash2Icon class="size-4 shrink-0" aria-hidden="true" />
                Delete
              </Button>
            </div>
          </TableCell>
        </TableRow>
      {/each}
    </TableBody>
  </Table>
</div>
