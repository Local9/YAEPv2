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

export type HealthSnapshot = {
  app: "yaep-rust";
  backendReady: boolean;
  activeProfileId: number | null;
};
