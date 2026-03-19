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
  showTitleOverlay: boolean;
};

export type ThumbnailSetting = {
  windowTitle: string;
  config: ThumbnailConfig;
};

export type ClientGroup = {
  id: number;
  profileId: number;
  name: string;
  displayOrder: number;
  cycleForwardHotkey: string;
  cycleBackwardHotkey: string;
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
};

export type GridLayoutPreviewItem = {
  windowTitle: string;
  x: number;
  y: number;
  width: number;
  height: number;
};
