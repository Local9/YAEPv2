export type Profile = {
  id: number;
  name: string;
  deletedAt: string | null;
  isActive: boolean;
  switchHotkey: string;
};

export type HealthSnapshot = {
  app: "yaep-rust";
  backendReady: boolean;
  activeProfileId: number | null;
};
