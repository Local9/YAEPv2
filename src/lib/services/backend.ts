import { invoke } from "@tauri-apps/api/core";
import type { HealthSnapshot, Profile } from "$models/domain";

export const backend = {
  health(): Promise<HealthSnapshot> {
    return invoke("health");
  },
  getProfiles(): Promise<Profile[]> {
    return invoke("get_profiles");
  }
};
