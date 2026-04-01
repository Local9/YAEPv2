import { describe, expect, it } from "vitest";
import MumbleLinksPage from "./+page.svelte";

describe("route: /mumble-links", () => {
  it("exports a Svelte component", () => {
    expect(MumbleLinksPage).toBeTruthy();
  });
});
