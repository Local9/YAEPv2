export const ERR_CHARACTER_ID_POSITIVE = "Character ID must be a positive whole number";
export const ERR_THUMBNAIL_DIMENSIONS = "Width and Height must be positive numbers";

export type ParseCharacterIdResult =
  | { ok: true; value: number | null }
  | { ok: false; message: string };

export function parseOptionalCharacterId(
  raw: string | number | null | undefined,
): ParseCharacterIdResult {
  if (raw == null) return { ok: true, value: null };
  const s = String(raw).trim();
  if (s === "") return { ok: true, value: null };
  const n = Number(s);
  if (!Number.isInteger(n) || n <= 0) {
    return { ok: false, message: ERR_CHARACTER_ID_POSITIVE };
  }
  return { ok: true, value: n };
}

/** Parse optional character ID from the per-override text field (empty = null). */
export function parseOptionalCharacterIdFromText(text: string): ParseCharacterIdResult {
  return parseOptionalCharacterId(text.trim() === "" ? null : text);
}

export type DimensionCheckResult = { ok: true } | { ok: false; message: string };

export function validatePositiveThumbnailDimensions(
  width: number,
  height: number,
): DimensionCheckResult {
  if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) {
    return { ok: false, message: ERR_THUMBNAIL_DIMENSIONS };
  }
  return { ok: true };
}
