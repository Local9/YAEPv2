import { describe, expect, it } from "vitest";
import ProfilesPage from "./+page.svelte";

describe("route: /profiles", () => {
  it("exports a Svelte component", () => {
    expect(ProfilesPage).toBeTruthy();
  });
});
