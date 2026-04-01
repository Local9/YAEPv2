import { describe, expect, it } from "vitest";
import SettingsPage from "./+page.svelte";

describe("route: /settings", () => {
  it("exports a Svelte component", () => {
    expect(SettingsPage).toBeTruthy();
  });
});
