import { describe, expect, it } from "vitest";
import ProcessManagementPage from "./+page.svelte";

describe("route: /process-management", () => {
  it("exports a Svelte component", () => {
    expect(ProcessManagementPage).toBeTruthy();
  });
});
