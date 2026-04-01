import { describe, expect, it } from "vitest";
import ThumbnailSettingsPage from "./+page.svelte";

describe("route: /thumbnail-settings", () => {
  it("exports a Svelte component", () => {
    expect(ThumbnailSettingsPage).toBeTruthy();
  });
});
