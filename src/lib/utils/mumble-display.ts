/** Placeholder name for the seeded Mumble server group in SQLite. */
const DEFAULT_SERVER_GROUP_LABEL = "default";

/**
 * UI label for a server group. The seeded group name "Default" is not shown as-is when a visible title is required.
 */
export function formatMumbleServerGroupDisplayName(name: string): string {
  const t = name.trim();
  if (t.length === 0) return "Links";
  if (t.toLowerCase() === DEFAULT_SERVER_GROUP_LABEL) return "Links";
  return t;
}
