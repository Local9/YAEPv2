<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { ClientGroup, Profile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import ArrowLeftIcon from "@lucide/svelte/icons/arrow-left";
  import ArrowRightIcon from "@lucide/svelte/icons/arrow-right";
  import CheckCircle2Icon from "@lucide/svelte/icons/check-circle-2";
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

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <LayersIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Client Grouping</CardTitle>
        <CardDescription>
          Client groups for active profile with cycle hotkey and activation baseline.
        </CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
    {#if status}
      <Alert class="border-primary/30 bg-primary/5">
        <CheckCircle2Icon class="size-4 text-primary" aria-hidden="true" />
        <AlertTitle>Status</AlertTitle>
        <AlertDescription>{status}</AlertDescription>
      </Alert>
    {/if}
    {#if error}
      <Alert variant="destructive">
        <AlertCircleIcon class="size-4 shrink-0" aria-hidden="true" />
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    {/if}

    <div class="mt-4 overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Name</TableHead>
            <TableHead>Display Order</TableHead>
            <TableHead>Forward Hotkey</TableHead>
            <TableHead>Backward Hotkey</TableHead>
            <TableHead>Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {#each groups as group (group.id)}
            <TableRow>
              <TableCell>{group.name}</TableCell>
              <TableCell>{group.displayOrder}</TableCell>
              <TableCell>
                <Input
                  class="min-w-[10rem]"
                  bind:value={group.cycleForwardHotkey}
                  placeholder="Ctrl+Alt+F13"
                  onblur={() => saveHotkeys(group)}
                />
              </TableCell>
              <TableCell>
                <Input
                  class="min-w-[10rem]"
                  bind:value={group.cycleBackwardHotkey}
                  placeholder="Ctrl+Alt+F14"
                  onblur={() => saveHotkeys(group)}
                />
              </TableCell>
              <TableCell>
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
              </TableCell>
            </TableRow>
          {/each}
        </TableBody>
      </Table>
    </div>
  </CardContent>
</Card>
