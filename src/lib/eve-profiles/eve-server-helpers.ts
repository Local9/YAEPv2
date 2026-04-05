import type { EveDetectedProfile } from "$models/domain";

export function isFrontierServer(serverName: string): boolean {
  return serverName.toLowerCase().includes("frontier");
}

export function getServerCode(serverName: string): string {
  const normalized = serverName.trim().toLowerCase();
  if (!normalized) return "";
  const parts = normalized.split(/[^a-z0-9]+/).filter((part) => part.length > 0);
  return parts.length > 0 ? parts[parts.length - 1] : normalized;
}

export function isTranquilityServer(serverName: string): boolean {
  const serverCode = getServerCode(serverName);
  return serverCode === "tq" || serverCode === "tranquility";
}

export function normalizeServerName(serverName: string): string {
  const serverCode = getServerCode(serverName);
  if (serverCode === "tq") {
    return "Tranquility (TQ)";
  }
  if (serverCode === "tranquility") {
    return "Tranquility (TQ)";
  }
  if (serverCode === "sisi") {
    return "Singularity (SQ)";
  }
  if (serverCode === "singularity") {
    return "Singularity (SQ)";
  }
  if (!serverCode) {
    return serverName;
  }
  return serverCode.charAt(0).toUpperCase() + serverCode.slice(1).toLowerCase();
}

export function isSupportedServer(serverName: string): boolean {
  const serverCode = getServerCode(serverName);
  return (
    serverCode === "tq" ||
    serverCode === "tranquility" ||
    serverCode === "sisi" ||
    serverCode === "singularity"
  );
}

export function filterNonFrontierSupported(profiles: EveDetectedProfile[]): EveDetectedProfile[] {
  return profiles.filter(
    (profile) => !isFrontierServer(profile.serverName) && isSupportedServer(profile.serverName),
  );
}

export function discoveredServersFromProfiles(profiles: EveDetectedProfile[]): string[] {
  return [...new Set(profiles.map((profile) => profile.serverName))]
    .filter((serverName) => serverName.trim().length > 0)
    .sort((a, b) => a.localeCompare(b));
}

export function sanitizeFileNameSegment(value: string): string {
  return value
    .trim()
    .replace(/[<>:"/\\|?*\u0000-\u001F]/g, "_");
}

export function formatYyyyMmDdCompact(date: Date): string {
  const yyyy = date.getFullYear();
  const mm = String(date.getMonth() + 1).padStart(2, "0");
  const dd = String(date.getDate()).padStart(2, "0");
  return `${yyyy}${mm}${dd}`;
}

export function splitDirAndFile(path: string): { dir: string; file: string } {
  const lastSlash = Math.max(path.lastIndexOf("\\"), path.lastIndexOf("/"));
  if (lastSlash < 0) return { dir: "", file: path };
  return { dir: path.slice(0, lastSlash), file: path.slice(lastSlash + 1) };
}

/** Pick default `selectedServer` from detected list when missing or invalid. */
export function resolveSelectedServer(
  detected: EveDetectedProfile[],
  current: string,
): string {
  const nonFrontier = filterNonFrontierSupported(detected);
  const isValid = (name: string) =>
    detected.some(
      (profile) =>
        profile.serverName === name &&
        !isFrontierServer(profile.serverName) &&
        isSupportedServer(profile.serverName),
    );

  if (current && isValid(current)) {
    return current;
  }

  const tranquility = detected
    .map((profile) => profile.serverName)
    .find((name) => !isFrontierServer(name) && isSupportedServer(name) && isTranquilityServer(name));
  if (tranquility) return tranquility;

  const singularity = detected
    .map((profile) => profile.serverName)
    .find(
      (name) =>
        !isFrontierServer(name) &&
        isSupportedServer(name) &&
        (getServerCode(name) === "sisi" || getServerCode(name) === "singularity"),
    );
  return singularity ?? "";
}
