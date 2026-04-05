export const PROFILE_SWITCH_CAPTURE = "profileSwitch" as const;

export const PROFILE_HOTKEY_INPUT_CLASS = "min-w-[10rem] cursor-pointer select-none";
export const PROFILE_HOTKEY_CAPTURE_RING_CLASS =
  "ring-ring ring-2 ring-offset-2 ring-offset-background";

export type ProfileHotkeyCaptureKind = typeof PROFILE_SWITCH_CAPTURE;
