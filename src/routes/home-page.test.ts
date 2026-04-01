import { describe, expect, it } from "vitest";
import HomePage from "./+page.svelte";

describe("route: /", () => {
  it("exports a Svelte component", () => {
    expect(HomePage).toBeTruthy();
  });
});
