import { describe, expect, it } from "vitest";
import GridLayoutPage from "./+page.svelte";

describe("route: /grid-layout", () => {
  it("exports a Svelte component", () => {
    expect(GridLayoutPage).toBeTruthy();
  });
});
