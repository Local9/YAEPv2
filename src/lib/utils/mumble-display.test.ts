import { describe, expect, it } from "vitest";
import { formatMumbleServerGroupDisplayName } from "./mumble-display";

describe("formatMumbleServerGroupDisplayName", () => {
  it("maps Default to Links", () => {
    expect(formatMumbleServerGroupDisplayName("Default")).toBe("Links");
    expect(formatMumbleServerGroupDisplayName("default")).toBe("Links");
    expect(formatMumbleServerGroupDisplayName("  Default  ")).toBe("Links");
  });

  it("preserves other names", () => {
    expect(formatMumbleServerGroupDisplayName("FC Comms")).toBe("FC Comms");
    expect(formatMumbleServerGroupDisplayName("NotDefault")).toBe("NotDefault");
  });

  it("uses Links for blank", () => {
    expect(formatMumbleServerGroupDisplayName("")).toBe("Links");
    expect(formatMumbleServerGroupDisplayName("   ")).toBe("Links");
  });
});
