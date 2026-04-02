<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { EveChatChannel, EveChatChannelType, EveLogSettings, Profile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { ColorPicker } from "$lib/components/ui/color-picker";
  import { Input } from "$lib/components/ui/input";
  import * as Select from "$lib/components/ui/select";
  import { toast } from "svelte-sonner";

  const DEFAULT_CHAT = "%USERPROFILE%\\Documents\\EVE\\logs\\Chatlogs";
  const DEFAULT_GAME = "%USERPROFILE%\\Documents\\EVE\\logs\\Gamelogs";

  let activeProfileId = $state<number | null>(null);
  let settings = $state<EveLogSettings>({ chatLogsPath: DEFAULT_CHAT, gameLogsPath: DEFAULT_GAME });
  let channels = $state<EveChatChannel[]>([]);
  let channelType = $state<EveChatChannelType>("FleetBoost");
  let channelName = $state("");
  let channelBackgroundColor = $state("#1f2937");
  let channelNameInput = $state<HTMLElement | null>(null);
  let saveMessage = $state("");
  let error = $state("");
  let sortedChannels = $derived(
    [...channels].sort((a, b) => {
      const typeCompare = a.channelType.localeCompare(b.channelType);
      if (typeCompare !== 0) return typeCompare;
      return a.channelName.localeCompare(b.channelName);
    })
  );

  async function refresh() {
    const profiles: Profile[] = await backend.getProfiles();
    activeProfileId = profiles.find((p) => p.isActive)?.id ?? null;
    if (activeProfileId == null) return;
    settings = await backend.eveGetLogSettings(activeProfileId);
    channels = await backend.eveListChatChannels(activeProfileId);
  }

  async function saveSettings() {
    if (activeProfileId == null) return;
    await backend.eveSaveLogSettings(activeProfileId, settings);
    saveMessage = "EVE log settings saved";
  }

  async function addChannel() {
    if (activeProfileId == null || !channelName.trim()) return;
    const existing = new Set(channels.map((channel) => channel.channelName.trim().toLocaleLowerCase()));
    const requestedNames = channelName
      .split(",")
      .map((value) => value.trim())
      .filter((value) => value.length > 0);

    if (requestedNames.length === 0) {
      channelNameInput?.focus();
      return;
    }

    let addedCount = 0;
    let skippedCount = 0;
    for (const requestedName of requestedNames) {
      const normalized = requestedName.toLocaleLowerCase();
      if (existing.has(normalized)) {
        skippedCount += 1;
        continue;
      }
      await backend.eveAddChatChannel(activeProfileId, channelType, requestedName, channelBackgroundColor);
      existing.add(normalized);
      addedCount += 1;
    }

    if (addedCount === 0 && skippedCount > 0) {
      error = "All channel names already exist";
    } else if (skippedCount > 0) {
      saveMessage = `Added ${addedCount} channel(s); skipped ${skippedCount} duplicate(s)`;
      error = "";
    } else {
      error = "";
    }

    channelName = "";
    channels = await backend.eveListChatChannels(activeProfileId);
    channelNameInput?.focus();
  }

  async function onChannelNameKeydown(event: KeyboardEvent) {
    if (event.key !== "Enter") return;
    event.preventDefault();
    await addChannel();
  }

  async function removeChannel(channelId: number) {
    if (activeProfileId == null) return;
    await backend.eveRemoveChatChannel(activeProfileId, channelId);
    channels = await backend.eveListChatChannels(activeProfileId);
  }

  async function updateChannelBackgroundColor(channelId: number, backgroundColor: string) {
    if (activeProfileId == null) return;
    await backend.eveUpdateChatChannelColor(activeProfileId, channelId, backgroundColor);
    channels = channels.map((channel) =>
      channel.id === channelId ? { ...channel, backgroundColor } : channel
    );
  }

  onMount(() => {
    void refresh().catch((e) => {
      error = String(e);
    });
  });

  $effect(() => {
    if (saveMessage) toast.success(saveMessage);
  });
  $effect(() => {
    if (error) toast.error(error);
  });
</script>

<div class="max-w-4xl space-y-4">
  <h1 class="text-xl font-semibold">EVE Log Settings</h1>
  <p class="text-sm text-muted-foreground">
    Configure Chatlogs and Gamelogs roots. Channel tailing reads files from the configured Chat Logs
    Path.
  </p>

  <div class="space-y-2">
    <label for="chat-logs-path" class="text-sm font-medium">Chat Logs Path</label>
    <Input id="chat-logs-path" bind:value={settings.chatLogsPath} />
    <Button variant="outline" onclick={() => (settings.chatLogsPath = DEFAULT_CHAT)}>Reset default</Button>
  </div>

  <div class="space-y-2">
    <label for="game-logs-path" class="text-sm font-medium">Game Logs Path</label>
    <Input id="game-logs-path" bind:value={settings.gameLogsPath} />
    <Button variant="outline" onclick={() => (settings.gameLogsPath = DEFAULT_GAME)}>Reset default</Button>
  </div>

  <Button onclick={saveSettings}>Save paths</Button>

  <div class="pt-4">
    <h2 class="mb-2 text-base font-semibold">Channel Sources</h2>
    <p class="mb-2 text-sm text-muted-foreground">
      Fleet and Local are always monitored by fixed names. Add user-defined FleetBoost and Intel channels here.
    </p>
    <div class="mb-3 flex gap-2">
      <Select.Root
        type="single"
        bind:value={channelType}
        items={[
          { value: "FleetBoost", label: "FleetBoost" },
          { value: "Intel", label: "Intel" },
        ]}
      >
        <Select.Trigger class="w-48">
          <span data-slot="select-value">{channelType}</span>
        </Select.Trigger>
        <Select.Content>
          <Select.Item value="FleetBoost">FleetBoost</Select.Item>
          <Select.Item value="Intel">Intel</Select.Item>
        </Select.Content>
      </Select.Root>
      <Input
        bind:value={channelName}
        bind:ref={channelNameInput}
        onkeydown={onChannelNameKeydown}
        placeholder="Channel name (comma-delimited supported)"
      />
      <div class="flex items-center gap-2 rounded-md border px-2 py-1">
        <ColorPicker bind:value={channelBackgroundColor} />
        <Input class="w-28 font-mono text-xs" bind:value={channelBackgroundColor} />
      </div>
      <Button onclick={addChannel}>Add</Button>
    </div>
    <div class="overflow-x-auto rounded-md border">
      <table class="w-full text-sm">
        <thead class="bg-muted/40 text-left">
          <tr>
            <th class="px-3 py-2 font-medium">Channel Type</th>
            <th class="px-3 py-2 font-medium">Channel Name</th>
            <th class="px-3 py-2 font-medium">Background</th>
            <th class="px-3 py-2 font-medium text-right">Actions</th>
          </tr>
        </thead>
        <tbody>
          {#if sortedChannels.length === 0}
            <tr>
              <td class="px-3 py-2 text-muted-foreground" colspan="4">
                No channels configured yet.
              </td>
            </tr>
          {:else}
            {#each sortedChannels as channel (channel.id)}
              <tr class="border-t">
                <td class="px-3 py-2">{channel.channelType}</td>
                <td class="px-3 py-2">{channel.channelName}</td>
                <td class="px-3 py-2">
                  <div class="flex items-center gap-2">
                    <ColorPicker
                      value={channel.backgroundColor}
                      oninput={(event) =>
                        void updateChannelBackgroundColor(
                          channel.id,
                          (event.currentTarget as HTMLInputElement).value,
                        )}
                    />
                    <Input
                      value={channel.backgroundColor}
                      class="w-28 font-mono text-xs"
                      onblur={(event) =>
                        void updateChannelBackgroundColor(
                          channel.id,
                          (event.currentTarget as HTMLInputElement).value,
                        )}
                    />
                  </div>
                </td>
                <td class="px-3 py-2 text-right">
                  <Button variant="ghost" onclick={() => removeChannel(channel.id)}>Remove</Button>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>
