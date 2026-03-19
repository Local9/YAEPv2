<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";

  let enableThumbnailDragging = $state(true);
  let startHidden = $state(false);
  let theme = $state("Dark");
  let saveStatus = $state("");

  async function refresh() {
    const dragging = await backend.getAppSetting("EnableThumbnailDragging");
    const hidden = await backend.getAppSetting("StartHidden");
    const currentTheme = await backend.getAppSetting("Theme");
    enableThumbnailDragging = dragging == null ? true : dragging === "true";
    startHidden = hidden == null ? false : hidden === "true";
    theme = currentTheme ?? "Dark";
  }

  async function save() {
    await backend.setAppSetting("EnableThumbnailDragging", String(enableThumbnailDragging));
    await backend.setAppSetting("StartHidden", String(startHidden));
    await backend.setAppSetting("Theme", theme);
    saveStatus = "Saved";
  }

  onMount(refresh);
</script>

<section class="card">
  <h2>Settings</h2>
  <p>Persisted AppSettings keys are wired.</p>
  <div style="display:grid; gap:0.5rem; max-width:460px;">
    <label><input type="checkbox" bind:checked={enableThumbnailDragging} /> Enable Thumbnail Dragging</label>
    <label><input type="checkbox" bind:checked={startHidden} /> Start Hidden</label>
    <label>Theme
      <select bind:value={theme}>
        <option>Dark</option>
        <option>Light</option>
      </select>
    </label>
    <button onclick={save}>Save settings</button>
    {#if saveStatus}<span>{saveStatus}</span>{/if}
  </div>
</section>
