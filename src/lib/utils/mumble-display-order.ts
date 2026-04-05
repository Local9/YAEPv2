/**
 * Normalizes display order values from the tree or from `<input type="number">` bindings
 * so IPC payloads never send NaN/null for Rust i64 fields.
 */
export function finiteDisplayOrder(raw: unknown): number {
  if (typeof raw === "number" && Number.isFinite(raw)) {
    return Math.trunc(raw);
  }
  if (typeof raw === "string") {
    const t = raw.trim();
    if (t === "") return 0;
    const n = Number(t);
    if (Number.isFinite(n)) return Math.trunc(n);
  }
  return 0;
}
