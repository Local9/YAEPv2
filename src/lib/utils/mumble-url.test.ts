import { describe, expect, it } from "vitest";
import { deriveMumbleLinkName, isAllowedMumbleLinkUrl } from "./mumble-url";

describe("deriveMumbleLinkName", () => {
  it("uses last path segment for mumble URL", () => {
    const url =
      "mumble://mumble.sh1t.space/Fleet%20Operations/Fleet%20Main/Fleet%20Sec%20Purple%20I%20Main?title=Root&version=1.2.0";
    expect(deriveMumbleLinkName(url)).toBe("Fleet Sec Purple I Main");
  });

  it("returns null for empty or invalid scheme", () => {
    expect(deriveMumbleLinkName("")).toBeNull();
    expect(deriveMumbleLinkName("ftp://x/y")).toBeNull();
  });

  it("accepts https for name derivation", () => {
    expect(deriveMumbleLinkName("https://example.com/a/b%20c")).toBe("b c");
  });
});

describe("isAllowedMumbleLinkUrl", () => {
  it("allows mumble and https", () => {
    expect(isAllowedMumbleLinkUrl("mumble://host/c")).toBe(true);
    expect(isAllowedMumbleLinkUrl("https://x")).toBe(true);
    expect(isAllowedMumbleLinkUrl("http://x")).toBe(false);
  });
});
