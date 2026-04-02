export const MUMBLE_FOLDER_ICON_KEYS = [
  "headphones",
  "folder",
  "folder-open",
  "users",
  "user",
  "radio",
  "mic",
  "message-circle",
  "shield",
  "star",
  "heart",
  "home",
  "briefcase",
  "gamepad-2",
  "map",
  "zap",
  "satellite-dish",
  "volume-2",
  "headset",
  "phone",
  "megaphone",
  "wrench",
  "bell",
  "cog"
] as const;

export type MumbleFolderIconKey = (typeof MUMBLE_FOLDER_ICON_KEYS)[number];

function labelFromKey(key: MumbleFolderIconKey): string {
  return key
    .split("-")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

export const MUMBLE_FOLDER_ICON_OPTIONS: { key: MumbleFolderIconKey; label: string }[] =
  MUMBLE_FOLDER_ICON_KEYS.map((key) => ({ key, label: labelFromKey(key) }));

export const FOLDER_ICON_SELECT_ITEMS = [
  { value: "", label: "Default (headphones)" },
  ...MUMBLE_FOLDER_ICON_OPTIONS.map((o) => ({ value: o.key, label: o.label }))
];
