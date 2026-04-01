<script lang="ts">
  import { onMount } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import {
    isPermissionGranted,
    requestPermission,
    sendNotification,
  } from "@tauri-apps/plugin-notification";
  import { backend } from "$services/backend";
  import type { Profile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Dialog from "$lib/components/ui/dialog";
  import { toast } from "svelte-sonner";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";
  import CheckIcon from "@lucide/svelte/icons/check";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";
  import UsersIcon from "@lucide/svelte/icons/users";

  const PROFILE_SWITCH_CAPTURE = "profileSwitch";
  const PROFILE_HOTKEY_INPUT_CLASS = "min-w-[10rem] cursor-pointer select-none";
  const PROFILE_HOTKEY_CAPTURE_RING_CLASS = "ring-ring ring-2 ring-offset-2 ring-offset-background";

  type ProfileHotkeyCaptureKind = typeof PROFILE_SWITCH_CAPTURE;

  interface HotkeyCapturedPayload {
    value: string;
    captureType: string;
    targetId: number | null;
  }

  let profiles = $state<Profile[]>([]);
  let clientGroupCounts = $state<Record<number, number>>({});
  let newProfileName = $state("");
  let status = $state("");
  let error = $state("");
  let createProfileDialogOpen = $state(false);
  let captureProfileHotkey = $state<{ profileId: number; kind: ProfileHotkeyCaptureKind } | null>(
    null,
  );
  let notificationPermissionRequested = $state(false);

  async function refreshProfiles() {
    profiles = await backend.getProfiles();
    const counts = await Promise.all(
      profiles.map(async (profile) => ({
        profileId: profile.id,
        count: (await backend.getClientGroups(profile.id)).length,
      })),
    );
    clientGroupCounts = Object.fromEntries(counts.map((item) => [item.profileId, item.count]));
  }

  async function addProfile() {
    const name = newProfileName.trim();
    if (!name) return;
    try {
      await backend.createProfile(name);
      newProfileName = "";
      createProfileDialogOpen = false;
      error = "";
      status = "Profile created";
      await refreshProfiles();
    } catch (e) {
      error = String(e);
    }
  }

  async function setActive(profileId: number) {
    try {
      await backend.setCurrentProfile(profileId);
      error = "";
      status = "Active profile updated and matching thumbnails refreshed";
      await refreshProfiles();
      await notifyProfileChanged(profileId);
    } catch (e) {
      error = String(e);
    }
  }

  async function saveHotkey(profileId: number, hotkey: string) {
    try {
      await backend.updateProfileHotkey(profileId, hotkey);
      error = "";
      status = "Hotkey saved";
      await refreshProfiles();
    } catch (e) {
      error = String(e);
    }
  }

  async function removeProfile(profileId: number) {
    try {
      await backend.deleteProfile(profileId);
      error = "";
      status = "Profile deleted";
      await refreshProfiles();
    } catch (e) {
      error = String(e);
    }
  }

  async function onProfileSwitchHotkeyPointerDown(profile: Profile) {
    captureProfileHotkey = { profileId: profile.id, kind: PROFILE_SWITCH_CAPTURE };
    error = "";
    try {
      await backend.hotkeysCaptureStart(PROFILE_SWITCH_CAPTURE, profile.id);
    } catch (e) {
      error = String(e);
      captureProfileHotkey = null;
    }
  }

  function stopProfileHotkeyCapture() {
    captureProfileHotkey = null;
    void backend.hotkeysCaptureStop();
  }

  function isCapturingProfileHotkey(profileId: number): boolean {
    return (
      captureProfileHotkey?.profileId === profileId &&
      captureProfileHotkey?.kind === PROFILE_SWITCH_CAPTURE
    );
  }

  function isEscapeHotkeyValue(rawHotkey: string): boolean {
    const tokens = rawHotkey
      .split("+")
      .map((token) => token.trim().toLowerCase())
      .filter((token) => token.length > 0);
    if (tokens.length === 0) return false;
    const keyToken = tokens[tokens.length - 1];
    return keyToken === "escape" || keyToken === "esc";
  }

  async function notifyProfileChanged(profileId: number) {
    const profileName = profiles.find((p) => p.id === profileId)?.name ?? `Profile ${profileId}`;
    try {
      let permissionGranted = await isPermissionGranted();
      if (!permissionGranted && !notificationPermissionRequested) {
        notificationPermissionRequested = true;
        const permission = await requestPermission();
        permissionGranted = permission === "granted";
      }
      if (permissionGranted) {
        sendNotification({
          title: "YAEP",
          body: `Profile changed: ${profileName}`,
        });
      }
    } catch {
      // Notifications are best-effort and should not interrupt profile changes.
    }
  }

  onMount(() => {
    void refreshProfiles();
    let unlistenProfileChanged: UnlistenFn | undefined;
    let unlistenHotkeyCaptured: UnlistenFn | undefined;

    void listen<{ profileId: number }>("profileChanged", (event) => {
      status = "Profile changed by hotkey; matching thumbnails refreshed";
      error = "";
      void (async () => {
        await refreshProfiles();
        await notifyProfileChanged(event.payload.profileId);
      })();
    }).then((unlisten) => {
      unlistenProfileChanged = unlisten;
    });

    void listen<HotkeyCapturedPayload>("hotkeyCaptured", (event) => {
      const payload = event.payload;
      if (payload.captureType !== PROFILE_SWITCH_CAPTURE || payload.targetId == null) return;
      stopProfileHotkeyCapture();
      if (payload.value.trim() === "" || isEscapeHotkeyValue(payload.value)) {
        const profile = profiles.find((p) => p.id === payload.targetId);
        if (profile) profile.switchHotkey = "";
        void saveHotkey(payload.targetId, "");
        return;
      }
      void saveHotkey(payload.targetId, payload.value);
    }).then((unlisten) => {
      unlistenHotkeyCaptured = unlisten;
    });

    return () => {
      unlistenProfileChanged?.();
      unlistenHotkeyCaptured?.();
      stopProfileHotkeyCapture();
    };
  });

  $effect(() => {
    if (status) toast.success(status);
  });

  $effect(() => {
    if (error) toast.error(error);
  });
</script>

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <UsersIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Profiles</CardTitle>
        <CardDescription>
          Manage profiles, active profile state, and profile switch hotkeys.
        </CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
    <div class="mb-4 flex flex-wrap items-center gap-2">
      <Button type="button" onclick={() => (createProfileDialogOpen = true)}>
        <PlusIcon class="size-4 shrink-0" aria-hidden="true" />
        Create Profile
      </Button>
    </div>

    <Dialog.Root
      bind:open={createProfileDialogOpen}
      onOpenChange={(open) => {
        if (!open) newProfileName = "";
      }}
    >
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
            bind:value={newProfileName}
            placeholder="Profile name"
            onkeydown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                void addProfile();
              }
            }}
          />
        </div>
        <Dialog.Footer>
          <Button
            type="button"
            variant="outline"
            onclick={() => {
              createProfileDialogOpen = false;
            }}
          >
            Cancel
          </Button>
          <Button type="button" onclick={addProfile} disabled={!newProfileName.trim()}>
            Create
          </Button>
        </Dialog.Footer>
      </Dialog.Content>
    </Dialog.Root>

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
                    onkeydown={(e) => {
                      if (e.key !== "Escape") return;
                      e.preventDefault();
                      if (isCapturingProfileHotkey(profile.id)) {
                        stopProfileHotkeyCapture();
                      }
                      profile.switchHotkey = "";
                      void saveHotkey(profile.id, "");
                    }}
                    onpaste={(e) => e.preventDefault()}
                    onblur={(e) => {
                      if (isCapturingProfileHotkey(profile.id)) {
                        stopProfileHotkeyCapture();
                        return;
                      }
                      void saveHotkey(profile.id, (e.currentTarget as HTMLInputElement).value);
                    }}
                  />
                </div>
              </TableCell>
              <TableCell>{profile.isActive ? "Yes" : "No"}</TableCell>
              <TableCell>
                <div class="flex flex-wrap gap-2">
                  <Button
                    type="button"
                    variant="outline"
                    onclick={() => setActive(profile.id)}
                    disabled={profile.isActive}
                  >
                    <CheckIcon class="size-4 shrink-0" aria-hidden="true" />
                    Set Active
                  </Button>
                  <Button
                    type="button"
                    variant="destructive"
                    onclick={() => removeProfile(profile.id)}
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
  </CardContent>
</Card>
