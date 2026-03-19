import { invoke } from "@tauri-apps/api/core";
import type {
  ClientGroup,
  DrawerSettings,
  HealthSnapshot,
  GridLayoutPayload,
  GridLayoutPreviewItem,
  MumbleLink,
  MumbleLinksOverlaySettings,
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
  createMumbleLink(name: string, url: string, displayOrder: number, hotkey: string): Promise<void> {
    return invoke("create_mumble_link", { name, url, displayOrder, hotkey });
  },
  updateMumbleLink(
    linkId: number,
    name: string,
    url: string,
    displayOrder: number,
    hotkey: string
  ): Promise<void> {
    return invoke("update_mumble_link", { linkId, name, url, displayOrder, hotkey });
  },
  setMumbleLinkSelected(linkId: number, isSelected: boolean): Promise<void> {
    return invoke("set_mumble_link_selected", { linkId, isSelected });
  },
  deleteMumbleLink(linkId: number): Promise<void> {
    return invoke("delete_mumble_link", { linkId });
  },
  getMumbleServerGroups(): Promise<MumbleServerGroup[]> {
    return invoke("get_mumble_server_groups");
  },
  createMumbleServerGroup(name: string, displayOrder: number): Promise<void> {
    return invoke("create_mumble_server_group", { name, displayOrder });
  },
  updateMumbleServerGroup(groupId: number, name: string, displayOrder: number): Promise<void> {
    return invoke("update_mumble_server_group", { groupId, name, displayOrder });
  },
  deleteMumbleServerGroup(groupId: number): Promise<void> {
    return invoke("delete_mumble_server_group", { groupId });
  },
  getMumbleLinksOverlaySettings(): Promise<MumbleLinksOverlaySettings> {
    return invoke("get_mumble_links_overlay_settings");
  },
  saveMumbleLinksOverlaySettings(settings: MumbleLinksOverlaySettings): Promise<void> {
    return invoke("save_mumble_links_overlay_settings", { settings });
  },
  getDrawerSettings(): Promise<DrawerSettings> {
    return invoke("get_drawer_settings");
  },
  saveDrawerSettings(settings: DrawerSettings): Promise<void> {
    return invoke("save_drawer_settings", { settings });
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
  gridPreviewLayout(payload: GridLayoutPayload): Promise<GridLayoutPreviewItem[]> {
    return invoke("grid_preview_layout", { payload });
  },
  gridApplyLayout(payload: GridLayoutPayload): Promise<void> {
    return invoke("grid_apply_layout", { payload });
  },
  activateWindowByPid(pid: number): Promise<void> {
    return invoke("activate_window_by_pid", { pid });
  },
  eveProfilesList(): Promise<string[]> {
    return invoke("eve_profiles_list");
  },
  eveCopyProfile(sourceProfile: string, newProfile: string): Promise<void> {
    return invoke("eve_copy_profile", { sourceProfile, newProfile });
  },
  eveCopyCharacterFiles(sourceProfile: string, targetProfile: string): Promise<void> {
    return invoke("eve_copy_character_files", { sourceProfile, targetProfile });
  }
};
