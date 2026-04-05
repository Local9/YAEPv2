export const CLIENT_GROUP_FORWARD_CAPTURE = "clientGroupCycleForward" as const;
export const CLIENT_GROUP_BACKWARD_CAPTURE = "clientGroupCycleBackward" as const;

export const GROUP_HOTKEY_CAPTURE_FIELD = {
  [CLIENT_GROUP_FORWARD_CAPTURE]: "cycleForwardHotkey",
  [CLIENT_GROUP_BACKWARD_CAPTURE]: "cycleBackwardHotkey",
} as const;

export type GroupHotkeyCaptureKind =
  | typeof CLIENT_GROUP_FORWARD_CAPTURE
  | typeof CLIENT_GROUP_BACKWARD_CAPTURE;

export const CYCLE_HOTKEY_INPUT_CLASS = "min-w-[10rem] cursor-pointer select-none";
export const CYCLE_HOTKEY_CAPTURE_RING_CLASS =
  "ring-ring ring-2 ring-offset-2 ring-offset-background";
