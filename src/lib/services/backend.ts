import { invoke } from "@tauri-apps/api/core";
import type {
  ClientGroup,
  HealthSnapshot,
  MumbleLink,
  MumbleServerGroup,
  Profile,
  ThumbnailConfig,
  ThumbnailSetting
} from "$models/domain";

export const backend = {
  health(): Promise<HealthSnapshot> {
    return invoke("health");
  },
  getProfiles(): Promise<Profile[]> {
    return invoke("get_profiles");
  },
  createProfile(name: string): Promise<Profile> {
    return invoke("create_profile", { name });
  },
  setCurrentProfile(profileId: number): Promise<void> {
    return invoke("set_current_profile", { profileId });
  },
  updateProfileHotkey(profileId: number, hotkey: string): Promise<void> {
    return invoke("update_profile_hotkey", { profileId, hotkey });
  },
  deleteProfile(profileId: number): Promise<void> {
    return invoke("delete_profile", { profileId });
  },
  getProcessesToPreview(profileId: number): Promise<string[]> {
    return invoke("get_processes_to_preview", { profileId });
  },
  addProcessToPreview(profileId: number, processName: string): Promise<void> {
    return invoke("add_process_to_preview", { profileId, processName });
  },
  removeProcessToPreview(profileId: number, processName: string): Promise<void> {
    return invoke("remove_process_to_preview", { profileId, processName });
  },
  getThumbnailDefaultConfig(profileId: number): Promise<ThumbnailConfig> {
    return invoke("get_thumbnail_default_config", { profileId });
  },
  setThumbnailDefaultConfig(profileId: number, config: ThumbnailConfig): Promise<void> {
    return invoke("set_thumbnail_default_config", { profileId, config });
  },
  getThumbnailSettings(profileId: number): Promise<ThumbnailSetting[]> {
    return invoke("get_thumbnail_settings", { profileId });
  },
  saveThumbnailSetting(profileId: number, windowTitle: string, config: ThumbnailConfig): Promise<void> {
    return invoke("save_thumbnail_setting", { profileId, windowTitle, config });
  },
  getClientGroups(profileId: number): Promise<ClientGroup[]> {
    return invoke("get_client_groups", { profileId });
  },
  updateClientGroupHotkeys(
    groupId: number,
    cycleForwardHotkey: string,
    cycleBackwardHotkey: string
  ): Promise<void> {
    return invoke("update_client_group_hotkeys", {
      groupId,
      cycleForwardHotkey,
      cycleBackwardHotkey
    });
  },
  cycleClientGroup(groupId: number, direction: "forward" | "backward"): Promise<void> {
    return invoke("cycle_client_group", { groupId, direction });
  },
  getMumbleLinks(): Promise<MumbleLink[]> {
    return invoke("get_mumble_links");
  },
  getMumbleServerGroups(): Promise<MumbleServerGroup[]> {
    return invoke("get_mumble_server_groups");
  },
  getAppSetting(key: string): Promise<string | null> {
    return invoke("get_app_setting", { key });
  },
  setAppSetting(key: string, value: string): Promise<void> {
    return invoke("set_app_setting", { key, value });
  },
  hotkeysCaptureStart(captureType: string, targetId?: number): Promise<void> {
    return invoke("hotkeys_capture_start", {
      payload: { captureType, targetId: targetId ?? null }
    });
  },
  hotkeysCaptureStop(): Promise<void> {
    return invoke("hotkeys_capture_stop");
  },
  gridApplyLayout(): Promise<void> {
    return invoke("grid_apply_layout");
  },
  activateWindowByPid(pid: number): Promise<void> {
    return invoke("activate_window_by_pid", { pid });
  },
  eveProfilesList(): Promise<string[]> {
    return invoke("eve_profiles_list");
  }
};
