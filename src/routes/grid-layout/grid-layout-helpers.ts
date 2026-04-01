import type { GridLayoutPayload, MonitorInfoDto } from "$models/domain";

export const GRID_LAYOUT_DEFAULT_RATIO = { rw: 16, rh: 9 };

export function parseAspectRatio(ratio: string): { rw: number; rh: number } {
  const trimmed = ratio.trim();
  const parts = trimmed.split(":");
  if (parts.length !== 2) {
    return GRID_LAYOUT_DEFAULT_RATIO;
  }

  const rw = Number.parseFloat(parts[0].trim());
  const rh = Number.parseFloat(parts[1].trim());
  if (!Number.isFinite(rw) || !Number.isFinite(rh) || rw <= 0 || rh <= 0) {
    return GRID_LAYOUT_DEFAULT_RATIO;
  }

  return { rw, rh };
}

export function clampNumber(n: number, lo: number, hi: number): number {
  return Math.max(lo, Math.min(hi, n));
}

interface SyncGridSizeInput {
  ratio: string;
  width: number;
  height: number;
  minWidth: number;
  maxWidth: number;
  minHeight: number;
  maxHeight: number;
}

export function syncHeightFromWidth(input: SyncGridSizeInput): { width: number; height: number } {
  const { rw, rh } = parseAspectRatio(input.ratio);
  const nextWidth = clampNumber(input.width, input.minWidth, input.maxWidth);
  let nextHeight = Math.round((nextWidth * rh) / rw);
  nextHeight = clampNumber(nextHeight, input.minHeight, input.maxHeight);
  const normalizedWidth = clampNumber(
    Math.round((nextHeight * rw) / rh),
    input.minWidth,
    input.maxWidth,
  );
  const normalizedHeight = clampNumber(
    Math.round((normalizedWidth * rh) / rw),
    input.minHeight,
    input.maxHeight,
  );
  return { width: normalizedWidth, height: normalizedHeight };
}

export function syncWidthFromHeight(input: SyncGridSizeInput): { width: number; height: number } {
  const { rw, rh } = parseAspectRatio(input.ratio);
  const nextHeight = clampNumber(input.height, input.minHeight, input.maxHeight);
  let nextWidth = Math.round((nextHeight * rw) / rh);
  nextWidth = clampNumber(nextWidth, input.minWidth, input.maxWidth);
  const normalizedHeight = clampNumber(
    Math.round((nextWidth * rh) / rw),
    input.minHeight,
    input.maxHeight,
  );
  return { width: nextWidth, height: normalizedHeight };
}

export function formatMonitorLabel(monitor: MonitorInfoDto): string {
  const widthPx = monitor.right - monitor.left;
  const heightPx = monitor.bottom - monitor.top;
  const primary = monitor.isPrimary ? " (Primary)" : "";
  return `#${monitor.index} - ${monitor.name || "Display"}${primary} - ${widthPx}x${heightPx} @ (${monitor.left}, ${monitor.top})`;
}

export function monitorWorkOffset(
  selectedMonitorIndex: string,
  monitors: MonitorInfoDto[],
): { ox: number; oy: number } {
  if (selectedMonitorIndex === "") {
    return { ox: 0, oy: 0 };
  }
  const monitor = monitors.find((item) => String(item.index) === selectedMonitorIndex);
  if (!monitor) {
    return { ox: 0, oy: 0 };
  }
  return { ox: monitor.workLeft, oy: monitor.workTop };
}

interface BuildPayloadInput {
  activeProfileId: number | null;
  gridCellWidth: number;
  gridCellHeight: number;
  gridStartX: number;
  gridStartY: number;
  gridColumns: number;
  onlyAffectActiveThumbnails: boolean;
  selectedMonitorIndex: string;
  selectedAnchorTitle: string;
}

export function buildGridLayoutPayload(input: BuildPayloadInput): {
  payload: GridLayoutPayload | null;
  error: string | null;
} {
  if (input.activeProfileId == null) {
    return { payload: null, error: "No active profile available" };
  }

  return {
    error: null,
    payload: {
      profileId: input.activeProfileId,
      gridCellWidth: input.gridCellWidth,
      gridCellHeight: input.gridCellHeight,
      gridCellRatio: null,
      gridStartX: input.gridStartX,
      gridStartY: input.gridStartY,
      gridColumns: input.gridColumns,
      selectedGroupId: null,
      onlyAffectActiveThumbnails: input.onlyAffectActiveThumbnails,
      selectedMonitorIndex:
        input.selectedMonitorIndex === "" ? null : Number.parseInt(input.selectedMonitorIndex, 10),
      gridAnchorWindowTitle: input.selectedAnchorTitle === "" ? null : input.selectedAnchorTitle,
    },
  };
}
