import { invoke } from "@tauri-apps/api/core";

export const ssr = false;

export async function load() {
  let themePref: "Dark" | "Light" = "Dark";
  try {
    const t = await invoke<string | null>("get_app_setting", { key: "Theme" });
    if (t === "Light") themePref = "Light";
  } catch {
    /* Tauri unavailable (e.g. plain Vite preview) */
  }
  return { themePref };
}
