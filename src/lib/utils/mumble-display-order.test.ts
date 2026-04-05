import { describe, expect, it } from "vitest";
import { finiteDisplayOrder } from "./mumble-display-order";

describe("finiteDisplayOrder", () => {
  it("truncates finite numbers", () => {
    expect(finiteDisplayOrder(3.7)).toBe(3);
    expect(finiteDisplayOrder(-2.1)).toBe(-2);
  });

  it("parses numeric strings from inputs", () => {
    expect(finiteDisplayOrder("12")).toBe(12);
    expect(finiteDisplayOrder("  4  ")).toBe(4);
  });

  it("maps invalid values to 0", () => {
    expect(finiteDisplayOrder(undefined)).toBe(0);
    expect(finiteDisplayOrder(null)).toBe(0);
    expect(finiteDisplayOrder(NaN)).toBe(0);
    expect(finiteDisplayOrder("")).toBe(0);
    expect(finiteDisplayOrder("x")).toBe(0);
    expect(finiteDisplayOrder({})).toBe(0);
  });
});
