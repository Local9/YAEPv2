export type Profile = {
  id: number;
  name: string;
  deletedAt: string | null;
  isActive: boolean;
  switchHotkey: string;
};

export type ThumbnailConfig = {
  width: number;
  height: number;
  x: number;
  y: number;
  opacity: number;
  focusBorderColor: string;
  focusBorderThickness: number;
  decloakFlashColor: string;
  decloakFlashThickness: number;
  decloakFlashDurationMs: number;
  showTitleOverlay: boolean;
};

export type ThumbnailSetting = {
  windowTitle: string;
  config: ThumbnailConfig;
  characterId?: number | null;
};

export type EveLogSettings = {
  chatLogsPath: string;
  gameLogsPath: string;
};

export type EveChatChannelType = "FleetBoost" | "Intel";

export type EveChatChannel = {
  id: number;
  profileId: number;
  channelType: EveChatChannelType;
  channelName: string;
  backgroundColor: string;
};

export type ClientGroup = {
  id: number;
  profileId: number;
  name: string;
  displayOrder: number;
  cycleForwardHotkey: string;
  cycleBackwardHotkey: string;
};

export type ClientGroupMember = {
  windowTitle: string;
  displayOrder: number;
};

export type ClientGroupDetail = ClientGroup & {
  members: ClientGroupMember[];
};

export type MumbleServerGroup = {
  id: number;
  name: string;
  displayOrder: number;
};

export type MumbleLink = {
  id: number;
  name: string;
  url: string;
  displayOrder: number;
  isSelected: boolean;
  hotkey: string;
};

export type MumbleLinksOverlaySettings = {
  alwaysOnTop: boolean;
  x: number;
  y: number;
  width: number;
  height: number;
};

export type DrawerSettings = {
  screenIndex: number;
  hardwareId: string;
  side: string;
  width: number;
  height: number;
  isVisible: boolean;
  isEnabled: boolean;
  selectedMumbleServerGroupId: number | null;
};

export type HealthSnapshot = {
  app: "yaep-rust";
  backendReady: boolean;
  activeProfileId: number | null;
};

/** Position and size for the overlay widget shell (drag/resize). */
export type WidgetLayoutRect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

export type WidgetBrowserFrame = WidgetLayoutRect & {
  url: string;
};

export type WidgetOverlayLayout = {
  browser: WidgetBrowserFrame;
  fleetMotd: WidgetLayoutRect;
  intelFeed: WidgetLayoutRect;
};

export type BrowserQuickLink = {
  id: string;
  url: string;
  title: string;
};

/** Preset quick links; kept in sync with `default_browser_quick_links` in the Rust backend. */
export const DEFAULT_BROWSER_QUICK_LINKS: BrowserQuickLink[] = [
  { id: "eve-uni-wiki", url: "https://wiki.eveuniversity.org/", title: "EVE University" },
  { id: "dotlan", url: "https://evemaps.dotlan.net/", title: "dotlan" },
  { id: "janice", url: "https://janice.e-351.com/", title: "Janice" }
];

export type WidgetOverlaySettings = {
  enabled: boolean;
  /** When enabled, whether the overlay window is shown (toggle from tray or here). */
  visible: boolean;
  monitorIndex: number;
  showBrowserWidget: boolean;
  showFleetMotdWidget: boolean;
  showIntelFeedWidget: boolean;
  /** When true, non-pinned widgets are hidden; overlay window stays open. */
  widgetsSuppressed: boolean;
  /** Browser widget stays visible while widgets are suppressed. */
  browserAlwaysDisplayed: boolean;
  /** Fleet MOTD widget stays visible while widgets are suppressed. */
  fleetMotdAlwaysDisplayed: boolean;
  /** Intel feed widget stays visible while widgets are suppressed. */
  intelFeedAlwaysDisplayed: boolean;
  /** Hotkey chord to toggle `widgetsSuppressed` (RegisterHotKey). */
  toggleHotkey: string;
  /** User-editable shortcuts shown when the browser URL is empty (new-tab style). */
  browserQuickLinks: BrowserQuickLink[];
  /** If set, this URL loads when the overlay opens while the saved layout URL is empty. */
  browserDefaultUrl: string | null;
  layout: WidgetOverlayLayout;
};

export type IntelWidgetLine = {
  timestamp: string;
  channelName: string;
  message: string;
  backgroundColor: string;
};

export type WidgetSnapshot = {
  fleetMotd: string;
  intelLines: IntelWidgetLine[];
};

export type RuntimeThumbnailSnapshot = {
  pid: number;
  windowTitle: string;
};

export type RuntimeThumbnailStateSnapshot = {
  thumbnails: RuntimeThumbnailSnapshot[];
  focused: {
    pid: number | null;
    windowTitle: string | null;
  };
};

export type EveDetectedProfile = {
  serverName: string;
  profileName: string;
  fullPath: string;
  isDefault: boolean;
};

export type EveProfileCharacter = {
  characterId: string;
  filePath: string;
};

export type EveProfileUser = {
  userId: string;
  filePath: string;
};

export type EveProfileSettingsSources = {
  characters: EveProfileCharacter[];
  users: EveProfileUser[];
};

export type MonitorInfoDto = {
  index: number;
  name: string;
  left: number;
  top: number;
  right: number;
  bottom: number;
  workLeft: number;
  workTop: number;
  workRight: number;
  workBottom: number;
  isPrimary: boolean;
  hardwareId: string;
};

export type GridLayoutPayload = {
  profileId: number;
  gridCellWidth: number;
  gridCellHeight: number | null;
  gridCellRatio: string | null;
  gridStartX: number;
  gridStartY: number;
  gridColumns: number;
  selectedGroupId: number | null;
  onlyAffectActiveThumbnails: boolean;
  selectedMonitorIndex?: number | null;
  /** Anchors grid origin and order: first cell uses this thumbnail's saved position. */
  gridAnchorWindowTitle?: string | null;
};

export type GridLayoutPreviewItem = {
  windowTitle: string;
  x: number;
  y: number;
  width: number;
  height: number;
};
