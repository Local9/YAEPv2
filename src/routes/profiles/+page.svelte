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
  import { toast } from "svelte-sonner";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import UsersIcon from "@lucide/svelte/icons/users";
  import {
    PROFILE_SWITCH_CAPTURE,
    type ProfileHotkeyCaptureKind,
  } from "$lib/components/profiles/profile-hotkeys";
  import ProfileCreateDialog from "$lib/components/profiles/profile-create-dialog.svelte";
  import ProfilesTable from "$lib/components/profiles/profiles-table.svelte";

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

  function onProfileHotkeyKeydown(profile: Profile, e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    e.preventDefault();
    if (isCapturingProfileHotkey(profile.id)) {
      stopProfileHotkeyCapture();
    }
    profile.switchHotkey = "";
    void saveHotkey(profile.id, "");
  }

  function onProfileHotkeyBlur(profile: Profile, e: FocusEvent) {
    if (isCapturingProfileHotkey(profile.id)) {
      stopProfileHotkeyCapture();
      return;
    }
    void saveHotkey(profile.id, (e.currentTarget as HTMLInputElement).value);
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

    <ProfileCreateDialog
      bind:open={createProfileDialogOpen}
      bind:profileName={newProfileName}
      onCreate={() => void addProfile()}
      onOpenChange={(open) => {
        if (!open) newProfileName = "";
      }}
    />

    <ProfilesTable
      {profiles}
      {clientGroupCounts}
      {isCapturingProfileHotkey}
      onProfileSwitchHotkeyPointerDown={onProfileSwitchHotkeyPointerDown}
      {onProfileHotkeyKeydown}
      {onProfileHotkeyBlur}
      onSetActive={setActive}
      onRemoveProfile={removeProfile}
    />
  </CardContent>
</Card>
