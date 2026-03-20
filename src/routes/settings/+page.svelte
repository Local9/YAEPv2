<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import CheckCircle2Icon from "@lucide/svelte/icons/check-circle-2";
  import SaveIcon from "@lucide/svelte/icons/save";
  import SettingsIcon from "@lucide/svelte/icons/settings";

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

<section class="rounded-lg border border-border bg-card p-4 text-card-foreground shadow-sm">
  <div class="mb-4 flex items-start gap-3">
    <SettingsIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
    <div>
      <h2 class="text-lg font-semibold tracking-tight">Settings</h2>
      <p class="mt-1 text-sm text-muted-foreground">App-level settings with persisted values.</p>
    </div>
  </div>

  {#if saveStatus}
    <Alert class="mt-3 border-primary/30 bg-primary/5">
      <CheckCircle2Icon class="size-4 text-primary" aria-hidden="true" />
      <AlertTitle>Status</AlertTitle>
      <AlertDescription>{saveStatus}</AlertDescription>
    </Alert>
  {/if}
  {#if error}
    <Alert variant="destructive" class="mt-3">
      <AlertCircleIcon class="size-4" aria-hidden="true" />
      <AlertTitle>Error</AlertTitle>
      <AlertDescription>{error}</AlertDescription>
    </Alert>
  {/if}

  <div class="mt-4 grid max-w-3xl gap-3">
    <label class="flex cursor-pointer items-center gap-2 text-sm font-medium">
      <input
        class="size-4 rounded border border-input text-primary focus-visible:ring-2 focus-visible:ring-ring"
        type="checkbox"
        bind:checked={enableThumbnailDragging}
      />
      <span class="text-foreground">Enable Thumbnail Dragging</span>
    </label>
    <label class="flex cursor-pointer items-center gap-2 text-sm font-medium">
      <input
        class="size-4 rounded border border-input text-primary focus-visible:ring-2 focus-visible:ring-ring"
        type="checkbox"
        bind:checked={startHidden}
      />
      <span class="text-foreground">Start Hidden</span>
    </label>
    <label class="grid max-w-md gap-1.5 text-sm font-medium">
      <span class="text-muted-foreground">Theme</span>
      <Input bind:value={theme} placeholder="Dark or Light" />
    </label>
    <div>
      <Button onclick={save} class="gap-2">
        <SaveIcon class="size-4 shrink-0" aria-hidden="true" />
        Save settings
      </Button>
    </div>
  </div>
</section>
