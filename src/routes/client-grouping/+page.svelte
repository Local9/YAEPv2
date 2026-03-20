<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { ClientGroup, Profile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import ArrowRightIcon from "@lucide/svelte/icons/arrow-right";
  import LayersIcon from "@lucide/svelte/icons/layers";

  let profiles = $state<Profile[]>([]);
  let groups = $state<ClientGroup[]>([]);
  let status = $state("");
  let error = $state("");

  async function refresh() {
    profiles = await backend.getProfiles();
    const active = profiles.find((p) => p.isActive);
    if (!active) {
      groups = [];
      return;
    }
    groups = await backend.getClientGroups(active.id);
  }

  async function saveHotkeys(group: ClientGroup) {
    try {
      await backend.updateClientGroupHotkeys(
        group.id,
        group.cycleForwardHotkey,
        group.cycleBackwardHotkey,
      );
      status = `Saved hotkeys for ${group.name}`;
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function cycle(group: ClientGroup, direction: "forward" | "backward") {
    try {
      await backend.cycleClientGroup(group.id, direction);
      status = `Cycled ${group.name} (${direction})`;
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  onMount(refresh);
</script>

<section class="rounded-lg border border-border bg-card p-4 text-card-foreground shadow-sm">
  <div class="mb-4 flex items-start gap-3">
    <LayersIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
    <div>
      <h2 class="text-lg font-semibold tracking-tight">Client Grouping</h2>
      <p class="mt-1 text-sm text-muted-foreground">
        Client groups for active profile with cycle hotkey and activation baseline.
      </p>
    </div>
  </div>

  {#if status}
    <p class="mb-2 text-sm text-muted-foreground">{status}</p>
  {/if}
  {#if error}
    <p class="mb-2 flex items-center gap-2 text-sm text-destructive" role="alert">
      <AlertCircleIcon class="size-4 shrink-0" aria-hidden="true" />
      {error}
    </p>
  {/if}

  <div class="overflow-x-auto">
    <table class="w-full border-collapse text-sm">
      <thead>
        <tr class="border-b border-border">
          <th class="p-2 text-left font-medium text-muted-foreground">Name</th>
          <th class="p-2 text-left font-medium text-muted-foreground">Display Order</th>
          <th class="p-2 text-left font-medium text-muted-foreground">Forward Hotkey</th>
          <th class="p-2 text-left font-medium text-muted-foreground">Backward Hotkey</th>
          <th class="p-2 text-left font-medium text-muted-foreground">Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each groups as group (group.id)}
          <tr class="border-b border-border/60">
            <td class="p-2 align-middle">{group.name}</td>
            <td class="p-2 align-middle">{group.displayOrder}</td>
            <td class="p-2 align-middle">
              <Input
                class="min-w-[10rem]"
                bind:value={group.cycleForwardHotkey}
                placeholder="Ctrl+Alt+F13"
                onblur={() => saveHotkeys(group)}
              />
            </td>
            <td class="p-2 align-middle">
              <Input
                class="min-w-[10rem]"
                bind:value={group.cycleBackwardHotkey}
                placeholder="Ctrl+Alt+F14"
                onblur={() => saveHotkeys(group)}
              />
            </td>
            <td class="p-2 align-middle">
              <div class="flex flex-wrap gap-2">
                <Button type="button" variant="outline" onclick={() => cycle(group, "forward")}>
                  <ArrowRightIcon class="size-4 shrink-0" aria-hidden="true" />
                  Cycle Next
                </Button>
                <Button type="button" variant="outline" onclick={() => cycle(group, "backward")}>
                  <ArrowLeftIcon class="size-4 shrink-0" aria-hidden="true" />
                  Cycle Prev
                </Button>
              </div>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</section>
