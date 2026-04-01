import { describe, expect, it } from "vitest";
import EveProfilesPage from "./+page.svelte";

describe("route: /eve-profiles", () => {
  it("exports a Svelte component", () => {
    expect(EveProfilesPage).toBeTruthy();
  });
});
