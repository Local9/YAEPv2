<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { Profile, ThumbnailConfig, ThumbnailSetting } from "$models/domain";

  let profiles = $state<Profile[]>([]);
  let activeProfileId = $state<number | null>(null);
  let defaultConfig = $state<ThumbnailConfig | null>(null);
  let settings = $state<ThumbnailSetting[]>([]);
  let windowTitle = $state("");
  let saveMessage = $state("");
  let error = $state("");

  function cloneDefault(): ThumbnailConfig | null {
    if (!defaultConfig) return null;
    return { ...defaultConfig };
  }

  async function refresh() {
    profiles = await backend.getProfiles();
    activeProfileId = profiles.find((p) => p.isActive)?.id ?? null;
    if (activeProfileId == null) return;
    defaultConfig = await backend.getThumbnailDefaultConfig(activeProfileId);
    settings = await backend.getThumbnailSettings(activeProfileId);
  }

  async function saveDefault() {
    if (activeProfileId == null || defaultConfig == null) return;
    try {
      await backend.setThumbnailDefaultConfig(activeProfileId, defaultConfig);
      saveMessage = "Default config saved";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function addOrUpdateWindowOverride() {
    if (activeProfileId == null || !windowTitle.trim()) return;
    const config = cloneDefault();
    if (!config) return;
    try {
      await backend.saveThumbnailSetting(activeProfileId, windowTitle.trim(), config);
      windowTitle = "";
      saveMessage = "Override saved";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  onMount(refresh);
</script>

<section class="card">
  <h2>Thumbnail Settings</h2>
  <p>Edit default and per-window-title thumbnail config.</p>
  {#if defaultConfig}
    <div style="display:grid; grid-template-columns: repeat(4, minmax(120px, 1fr)); gap:0.5rem;">
      <label>Width <input type="number" bind:value={defaultConfig.width} /></label>
      <label>Height <input type="number" bind:value={defaultConfig.height} /></label>
      <label>X <input type="number" bind:value={defaultConfig.x} /></label>
      <label>Y <input type="number" bind:value={defaultConfig.y} /></label>
      <label>Opacity <input type="number" step="0.05" min="0.1" max="1" bind:value={defaultConfig.opacity} /></label>
      <label>Border Color <input bind:value={defaultConfig.focusBorderColor} /></label>
      <label>Border Thickness <input type="number" bind:value={defaultConfig.focusBorderThickness} /></label>
      <label>Show Title <input type="checkbox" bind:checked={defaultConfig.showTitleOverlay} /></label>
    </div>
    <div style="margin-top:0.75rem;">
      <button onclick={saveDefault}>Save default</button>
      {#if saveMessage}<span style="margin-left:0.5rem;">{saveMessage}</span>{/if}
    </div>
  {/if}
  {#if error}<p style="color:#ff8f8f;">{error}</p>{/if}

  <hr style="margin: 1rem 0;" />
  <h3>Per-title override</h3>
  <div style="display:flex; gap:0.5rem;">
    <input bind:value={windowTitle} placeholder="EVE - CharacterName" />
    <button onclick={addOrUpdateWindowOverride}>Save override from default</button>
  </div>
  <ul>
    {#each settings as setting (setting.windowTitle)}
      <li><code>{setting.windowTitle}</code> ({setting.config.width}x{setting.config.height})</li>
    {/each}
  </ul>
</section>
