<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { EveFolderSettings, Profile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { toast } from "svelte-sonner";

  let activeProfileId = $state<number | null>(null);
  let defaultSettings = $state<EveFolderSettings | null>(null);
  let settings = $state<EveFolderSettings>({
    chatLogsPath: "",
    gameLogsPath: "",
    piTemplatesPath: ""
  });
  let isLoading = $state(true);
  let isSaving = $state(false);
  let loadError = $state("");

  function resetPath(folderType: "ChatLog" | "GameLog" | "PlanetaryInteractionTemplates"): void {
    if (!defaultSettings) {
      toast.error("Default folder paths are unavailable. You can still edit and save manually.");
      return;
    }

    if (folderType === "ChatLog") {
      settings.chatLogsPath = defaultSettings.chatLogsPath;
      return;
    }
    if (folderType === "GameLog") {
      settings.gameLogsPath = defaultSettings.gameLogsPath;
      return;
    }
    settings.piTemplatesPath = defaultSettings.piTemplatesPath;
  }

  async function refresh(): Promise<void> {
    isLoading = true;
    loadError = "";
    try {
      const profiles: Profile[] = await backend.getProfiles();
      activeProfileId = profiles.find((p) => p.isActive)?.id ?? null;
      if (activeProfileId == null) {
        loadError = "No active profile is selected.";
        return;
      }

      const [settingsResult, defaultsResult] = await Promise.allSettled([
        backend.eveGetFolderSettings(activeProfileId),
        backend.eveGetFolderDefaults()
      ]);

      if (settingsResult.status === "fulfilled") {
        settings = settingsResult.value;
      } else {
        throw settingsResult.reason;
      }

      if (defaultsResult.status === "fulfilled") {
        defaultSettings = defaultsResult.value;
      } else {
        defaultSettings = null;
        toast.error("Failed to load default folder paths. You can still edit and save current settings.");
      }
    } catch (e) {
      loadError = String(e);
      toast.error("Failed to load EVE folder settings");
    } finally {
      isLoading = false;
    }
  }

  async function save(): Promise<void> {
    if (activeProfileId == null) {
      toast.error("No active profile selected");
      return;
    }
    isSaving = true;
    try {
      await backend.eveSaveFolderSettings(activeProfileId, settings);
      toast.success("EVE folder settings saved");
    } catch (e) {
      toast.error("Failed to save EVE folder settings");
      loadError = String(e);
    } finally {
      isSaving = false;
    }
  }

  onMount(() => {
    void refresh();
  });
</script>

<div class="max-w-5xl space-y-4">
  <h1 class="text-xl font-semibold">EVE Folders</h1>
  <p class="text-sm text-muted-foreground">
    Configure profile-specific EVE folder paths for chat logs, game logs, and PI templates.
  </p>

  {#if isLoading}
    <p class="text-sm text-muted-foreground">Loading folder settings...</p>
  {:else if loadError}
    <p class="text-sm text-destructive">{loadError}</p>
  {/if}

  <div class="overflow-x-auto rounded-md border">
    <table class="w-full text-sm">
      <thead class="bg-muted/40 text-left">
        <tr>
          <th class="px-3 py-2 font-medium">Folder Type</th>
          <th class="px-3 py-2 font-medium">Path</th>
          <th class="px-3 py-2 font-medium text-right">Actions</th>
        </tr>
      </thead>
      <tbody>
        <tr class="border-t">
          <td class="px-3 py-2">ChatLog</td>
          <td class="px-3 py-2">
            <Input id="chat-log-path" bind:value={settings.chatLogsPath} />
          </td>
          <td class="px-3 py-2 text-right">
            <Button variant="outline" onclick={() => resetPath("ChatLog")}>Reset default</Button>
          </td>
        </tr>
        <tr class="border-t">
          <td class="px-3 py-2">GameLog</td>
          <td class="px-3 py-2">
            <Input id="game-log-path" bind:value={settings.gameLogsPath} />
          </td>
          <td class="px-3 py-2 text-right">
            <Button variant="outline" onclick={() => resetPath("GameLog")}>Reset default</Button>
          </td>
        </tr>
        <tr class="border-t">
          <td class="px-3 py-2">PlanetaryInteractionTemplates</td>
          <td class="px-3 py-2">
            <Input id="pi-templates-path" bind:value={settings.piTemplatesPath} />
          </td>
          <td class="px-3 py-2 text-right">
            <Button variant="outline" onclick={() => resetPath("PlanetaryInteractionTemplates")}>
              Reset default
            </Button>
          </td>
        </tr>
      </tbody>
    </table>
  </div>

  <div class="pt-1">
    <Button onclick={save} disabled={isLoading || isSaving}>
      {isSaving ? "Saving..." : "Save"}
    </Button>
  </div>
</div>
