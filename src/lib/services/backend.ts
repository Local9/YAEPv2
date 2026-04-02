import { invoke } from "@tauri-apps/api/core";
import type {
  ClientGroup,
  ClientGroupDetail,
  HealthSnapshot,
  GridLayoutPayload,
  GridLayoutPreviewItem,
  MonitorInfoDto,
  MumbleLink,
  MumbleServerGroup,
  MumbleTreeSnapshot,
  Profile,
  RuntimeThumbnailStateSnapshot,
  ThumbnailConfig,
  ThumbnailSetting,
  BrowserQuickLink,
  EveChatChannel,
  EveChatChannelType,
  EveLogSettings,
  WidgetSnapshot,
  WidgetOverlayLayout,
  WidgetOverlaySettings,
  EveDetectedProfile,
  EveProfileSettingsSources
} from "$models/domain";

// Security boundary note: renderer values are untrusted.
// All privileged validation and authorization must be enforced in Rust commands.
export const backend = {
  health(): Promise<HealthSnapshot> {
    return invoke("health");
  },
  listMonitors(): Promise<MonitorInfoDto[]> {
    return invoke("list_monitors_cmd");
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
  saveThumbnailSetting(
    profileId: number,
    windowTitle: string,
    config: ThumbnailConfig,
    characterId?: number | null
  ): Promise<void> {
    return invoke("save_thumbnail_setting", { profileId, windowTitle, config, characterId: characterId ?? null });
  },
  eveGetLogSettings(profileId: number): Promise<EveLogSettings> {
    return invoke("eve_get_log_settings", { profileId });
  },
  eveSaveLogSettings(profileId: number, settings: EveLogSettings): Promise<void> {
    return invoke("eve_save_log_settings", { profileId, settings });
  },
  eveListChatChannels(profileId: number): Promise<EveChatChannel[]> {
    return invoke("eve_list_chat_channels", { profileId });
  },
  eveAddChatChannel(
    profileId: number,
    channelType: EveChatChannelType,
    channelName: string,
    backgroundColor?: string | null
  ): Promise<EveChatChannel> {
    return invoke("eve_add_chat_channel", { profileId, channelType, channelName, backgroundColor: backgroundColor ?? null });
  },
  eveRemoveChatChannel(profileId: number, channelId: number): Promise<void> {
    return invoke("eve_remove_chat_channel", { profileId, channelId });
  },
  eveUpdateChatChannelColor(profileId: number, channelId: number, backgroundColor: string): Promise<void> {
    return invoke("eve_update_chat_channel_color", { profileId, channelId, backgroundColor });
  },
  getClientGroups(profileId: number): Promise<ClientGroup[]> {
    return invoke("get_client_groups", { profileId });
  },
  getClientGroupsDetailed(profileId: number): Promise<ClientGroupDetail[]> {
    return invoke("get_client_groups_detailed", { profileId });
  },
  createClientGroup(profileId: number, name: string): Promise<ClientGroupDetail> {
    return invoke("create_client_group", { profileId, name });
  },
  deleteClientGroup(profileId: number, groupId: number): Promise<void> {
    return invoke("delete_client_group", { profileId, groupId });
  },
  addClientGroupMember(profileId: number, groupId: number, windowTitle: string): Promise<void> {
    return invoke("add_client_group_member", { profileId, groupId, windowTitle });
  },
  removeClientGroupMember(profileId: number, groupId: number, windowTitle: string): Promise<void> {
    return invoke("remove_client_group_member", { profileId, groupId, windowTitle });
  },
  reorderClientGroupMembers(
    profileId: number,
    groupId: number,
    windowTitlesInOrder: string[]
  ): Promise<void> {
    return invoke("reorder_client_group_members", { profileId, groupId, windowTitlesInOrder });
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
  getMumbleTree(): Promise<MumbleTreeSnapshot> {
    return invoke("get_mumble_tree");
  },
  createMumbleFolder(
    serverGroupId: number,
    parentFolderId: number | null,
    name: string,
    displayOrder: number
  ): Promise<number> {
    return invoke("create_mumble_folder", {
      payload: { serverGroupId, parentFolderId, name, displayOrder }
    });
  },
  updateMumbleFolder(folderId: number, name: string, displayOrder: number): Promise<void> {
    return invoke("update_mumble_folder", {
      payload: { folderId, name, displayOrder }
    });
  },
  deleteMumbleFolder(folderId: number): Promise<void> {
    return invoke("delete_mumble_folder", { folderId });
  },
  createMumbleLink(
    name: string,
    url: string,
    displayOrder: number,
    hotkey: string,
    serverGroupId: number,
    folderId: number | null
  ): Promise<void> {
    return invoke("create_mumble_link", {
      payload: { name, url, displayOrder, hotkey, serverGroupId, folderId }
    });
  },
  updateMumbleLink(
    linkId: number,
    name: string,
    url: string,
    displayOrder: number,
    hotkey: string,
    serverGroupId: number,
    folderId: number | null
  ): Promise<void> {
    return invoke("update_mumble_link", {
      payload: { linkId, name, url, displayOrder, hotkey, serverGroupId, folderId }
    });
  },
  openMumbleLink(linkId: number): Promise<void> {
    return invoke("open_mumble_link", { linkId });
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
  openExternalUrl(url: string): Promise<void> {
    return invoke("open_external_url", { url });
  },
  getRuntimeThumbnailState(): Promise<RuntimeThumbnailStateSnapshot> {
    return invoke("get_runtime_thumbnail_state");
  },
  appReady(): Promise<void> {
    return invoke("app_ready");
  },
  widgetGetSnapshot(): Promise<WidgetSnapshot> {
    return invoke("widget_get_snapshot");
  },
  eveProfilesList(): Promise<string[]> {
    return invoke("eve_profiles_list");
  },
  eveProfilesDetected(): Promise<EveDetectedProfile[]> {
    return invoke("eve_profiles_detected");
  },
  eveCopyProfileOnServer(
    serverName: string,
    sourceProfileName: string,
    newProfileName: string
  ): Promise<void> {
    return invoke("eve_copy_profile_on_server", { serverName, sourceProfileName, newProfileName });
  },
  eveDeleteProfileOnServer(serverName: string, profileName: string): Promise<void> {
    return invoke("eve_delete_profile_on_server", { serverName, profileName });
  },
  eveGetProfileSettingsSources(serverName: string, profileName: string): Promise<EveProfileSettingsSources> {
    return invoke("eve_get_profile_settings_sources", { serverName, profileName });
  },
  eveCopyProfileSettingsFromSources(
    serverName: string,
    profileName: string,
    sourceCharacterId: string,
    sourceUserId: string
  ): Promise<void> {
    return invoke("eve_copy_profile_settings_from_sources", {
      serverName,
      profileName,
      sourceCharacterId,
      sourceUserId
    });
  },
  eveCopyProfile(sourceProfile: string, newProfile: string): Promise<void> {
    return invoke("eve_copy_profile", { sourceProfile, newProfile });
  },
  eveCopyCharacterFiles(sourceProfile: string, targetProfile: string): Promise<void> {
    return invoke("eve_copy_character_files", { sourceProfile, targetProfile });
  },
  eveCopyCharacterFilesOnServer(
    serverName: string,
    sourceProfileName: string,
    targetProfileName: string
  ): Promise<void> {
    return invoke("eve_copy_character_files_on_server", {
      serverName,
      sourceProfileName,
      targetProfileName
    });
  },
  eveBackupAllProfiles(serverName: string, outputPath: string): Promise<void> {
    return invoke("eve_backup_all_profiles", { serverName, outputPath });
  },
  eveFetchCharacterName(characterId: number): Promise<string> {
    return invoke("eve_fetch_character_name", { characterId });
  },
  widgetOverlayGetSettings(): Promise<WidgetOverlaySettings> {
    return invoke("widget_overlay_get_settings");
  },
  widgetOverlaySaveSettings(settings: WidgetOverlaySettings): Promise<void> {
    return invoke("widget_overlay_save_settings", { settings });
  },
  widgetOverlaySaveLayout(layout: WidgetOverlayLayout): Promise<void> {
    return invoke("widget_overlay_save_layout", { layout });
  },
  widgetOverlaySaveBrowserQuickLinks(
    links: BrowserQuickLink[],
    defaultUrl: string | null
  ): Promise<void> {
    return invoke("widget_overlay_save_browser_quick_links", { links, defaultUrl });
  },
  widgetOverlayRefresh(): Promise<void> {
    return invoke("widget_overlay_refresh");
  },
  widgetOverlayToggle(): Promise<boolean> {
    return invoke("widget_overlay_toggle");
  }
};
