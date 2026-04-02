import { describe, expect, it } from "vitest";
import type { MonitorInfoDto } from "$models/domain";
import {
  buildGridLayoutFormPrefs,
  buildGridLayoutPayload,
  clampNumber,
  formatMonitorLabel,
  monitorWorkOffset,
  parseAspectRatio,
  syncHeightFromWidth,
  syncWidthFromHeight,
} from "./grid-layout-helpers";

function monitor(overrides: Partial<MonitorInfoDto> = {}): MonitorInfoDto {
  return {
    index: 1,
    name: "Main",
    left: 0,
    top: 0,
    right: 2560,
    bottom: 1440,
    workLeft: 0,
    workTop: 0,
    workRight: 2560,
    workBottom: 1400,
    isPrimary: true,
    hardwareId: "\\\\.\\DISPLAY1",
    ...overrides,
  };
}

describe("grid-layout helpers", () => {
  describe("parseAspectRatio", () => {
    it("parses valid ratio values", () => {
      expect(parseAspectRatio("21:9")).toEqual({ rw: 21, rh: 9 });
    });

    it("falls back to default ratio on malformed values", () => {
      expect(parseAspectRatio("abc")).toEqual({ rw: 16, rh: 9 });
      expect(parseAspectRatio("16:0")).toEqual({ rw: 16, rh: 9 });
      expect(parseAspectRatio("0:9")).toEqual({ rw: 16, rh: 9 });
    });
  });

  describe("clampNumber", () => {
    it("clamps below and above bounds", () => {
      expect(clampNumber(1, 10, 20)).toBe(10);
      expect(clampNumber(30, 10, 20)).toBe(20);
      expect(clampNumber(15, 10, 20)).toBe(15);
    });
  });

  describe("syncHeightFromWidth", () => {
    it("computes clamped width/height from width input", () => {
      const result = syncHeightFromWidth({
        ratio: "16:9",
        width: 1000,
        height: 200,
        minWidth: 192,
        maxWidth: 960,
        minHeight: 108,
        maxHeight: 540,
      });
      expect(result).toEqual({ width: 960, height: 540 });
    });
  });

  describe("syncWidthFromHeight", () => {
    it("computes clamped width/height from height input", () => {
      const result = syncWidthFromHeight({
        ratio: "4:3",
        width: 300,
        height: 540,
        minWidth: 192,
        maxWidth: 960,
        minHeight: 108,
        maxHeight: 540,
      });
      expect(result).toEqual({ width: 720, height: 540 });
    });
  });

  describe("formatMonitorLabel", () => {
    it("formats monitor metadata and primary marker", () => {
      const label = formatMonitorLabel(
        monitor({ index: 2, name: "Side", left: 2560, right: 4480, top: 0, bottom: 1080 }),
      );
      expect(label).toContain("#2 - Side");
      expect(label).toContain("1920x1080");
      expect(label).toContain("(2560, 0)");
    });
  });

  describe("monitorWorkOffset", () => {
    it("returns zero offset for empty or unknown monitor selection", () => {
      const monitors = [monitor()];
      expect(monitorWorkOffset("", monitors)).toEqual({ ox: 0, oy: 0 });
      expect(monitorWorkOffset("9", monitors)).toEqual({ ox: 0, oy: 0 });
    });

    it("returns selected monitor work-area offset", () => {
      const monitors = [monitor({ index: 3, workLeft: -1200, workTop: 50 })];
      expect(monitorWorkOffset("3", monitors)).toEqual({ ox: -1200, oy: 50 });
    });
  });

  describe("buildGridLayoutPayload", () => {
    it("returns an error when active profile is missing", () => {
      const result = buildGridLayoutPayload({
        activeProfileId: null,
        gridCellWidth: 300,
        gridCellHeight: 169,
        gridStartX: 100,
        gridStartY: 200,
        gridColumns: 3,
        onlyAffectActiveThumbnails: true,
        selectedMonitorIndex: "",
        selectedAnchorTitle: "",
      });

      expect(result.payload).toBeNull();
      expect(result.error).toBe("No active profile available");
    });

    it("maps empty monitor and anchor values to null", () => {
      const result = buildGridLayoutPayload({
        activeProfileId: 5,
        gridCellWidth: 300,
        gridCellHeight: 169,
        gridStartX: 100,
        gridStartY: 200,
        gridColumns: 3,
        onlyAffectActiveThumbnails: false,
        selectedMonitorIndex: "",
        selectedAnchorTitle: "",
      });

      expect(result.error).toBeNull();
      expect(result.payload).toMatchObject({
        profileId: 5,
        selectedMonitorIndex: null,
        gridAnchorWindowTitle: null,
      });
    });

    it("parses selected monitor index and preserves anchor title", () => {
      const result = buildGridLayoutPayload({
        activeProfileId: 8,
        gridCellWidth: 400,
        gridCellHeight: 225,
        gridStartX: -300,
        gridStartY: 50,
        gridColumns: 4,
        onlyAffectActiveThumbnails: true,
        selectedMonitorIndex: "2",
        selectedAnchorTitle: "Client Window",
      });

      expect(result.error).toBeNull();
      expect(result.payload).toMatchObject({
        profileId: 8,
        selectedMonitorIndex: 2,
        gridAnchorWindowTitle: "Client Window",
      });
    });
  });

  describe("buildGridLayoutFormPrefs", () => {
    it("maps UI fields to persisted prefs shape", () => {
      const prefs = buildGridLayoutFormPrefs({
        selectedAspectRatio: "21:9",
        gridCellWidth: 320,
        gridCellHeight: 137,
        gridStartX: 10,
        gridStartY: 20,
        gridColumns: 2,
        onlyAffectActiveThumbnails: false,
        selectedMonitorIndex: "1",
        selectedAnchorTitle: "Alpha",
      });
      expect(prefs).toEqual({
        aspectRatio: "21:9",
        gridCellWidth: 320,
        gridCellHeight: 137,
        gridStartX: 10,
        gridStartY: 20,
        gridColumns: 2,
        onlyAffectActiveThumbnails: false,
        selectedMonitorIndex: "1",
        selectedAnchorTitle: "Alpha",
      });
    });
  });
});
