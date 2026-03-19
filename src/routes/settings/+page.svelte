<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";

  let enableThumbnailDragging = $state(true);
  let startHidden = $state(false);
  let theme = $state("Dark");
  let saveStatus = $state("");
  let error = $state("");

  async function refresh() {
    try {
      const dragging = await backend.getAppSetting("EnableThumbnailDragging");
      const hidden = await backend.getAppSetting("StartHidden");
      const currentTheme = await backend.getAppSetting("Theme");
      enableThumbnailDragging = dragging == null ? true : dragging === "true";
      startHidden = hidden == null ? false : hidden === "true";
      theme = currentTheme ?? "Dark";
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  async function save() {
    try {
      await backend.setAppSetting("EnableThumbnailDragging", String(enableThumbnailDragging));
      await backend.setAppSetting("StartHidden", String(startHidden));
      await backend.setAppSetting("Theme", theme);
      saveStatus = "Settings saved";
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  onMount(refresh);
</script>

<section class="card">
  <h2>Settings</h2>
  <p>App-level settings with persisted values.</p>
  {#if saveStatus}
    <Alert>
      <AlertTitle>Status</AlertTitle>
      <AlertDescription>{saveStatus}</AlertDescription>
    </Alert>
  {/if}
  {#if error}
    <Alert>
      <AlertTitle>Error</AlertTitle>
      <AlertDescription>{error}</AlertDescription>
    </Alert>
  {/if}

  <div style="display:grid; gap:0.5rem; max-width:760px; margin-top:0.75rem;">
    <label><input type="checkbox" bind:checked={enableThumbnailDragging} /> Enable Thumbnail Dragging</label>
    <label><input type="checkbox" bind:checked={startHidden} /> Start Hidden</label>
    <label>Theme
      <Input bind:value={theme} placeholder="Dark or Light" />
    </label>
    <Button onclick={save}>Save settings</Button>
  </div>
</section>
