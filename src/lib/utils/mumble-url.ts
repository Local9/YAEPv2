/**
 * Derives a display name from a Mumble (or https) link URL using the last non-empty path segment.
 */
export function deriveMumbleLinkName(rawUrl: string): string | null {
  const trimmed = rawUrl.trim();
  if (!trimmed) return null;
  const lower = trimmed.toLowerCase();
  if (!lower.startsWith("mumble://") && !lower.startsWith("https://")) {
    return null;
  }
  try {
    const u = new URL(trimmed);
    const segments = u.pathname
      .split("/")
      .map((s) => {
        try {
          return decodeURIComponent(s.trim());
        } catch {
          return s.trim();
        }
      })
      .filter((s) => s.length > 0);
    if (segments.length === 0) {
      const title = u.searchParams.get("title")?.trim();
      return title && title.length > 0 && !isPlaceholderTitle(title) ? title : u.hostname || null;
    }
    const last = segments[segments.length - 1];
    return last.length > 0 ? last : null;
  } catch {
    return null;
  }
}

function isPlaceholderTitle(title: string): boolean {
  return title.toLowerCase() === "root";
}

export function isAllowedMumbleLinkUrl(url: string): boolean {
  const value = url.trim().toLowerCase();
  return value.startsWith("mumble://") || value.startsWith("https://");
}
