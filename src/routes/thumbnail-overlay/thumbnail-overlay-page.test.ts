import { describe, expect, it } from "vitest";
import ThumbnailOverlayPage from "./+page.svelte";

describe("route: /thumbnail-overlay", () => {
  it("exports a Svelte component", () => {
    expect(ThumbnailOverlayPage).toBeTruthy();
  });
});
