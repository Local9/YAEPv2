<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { MumbleLink, MumbleServerGroup } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Input } from "$lib/components/ui/input";
  import { toast } from "svelte-sonner";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";
  import LinkIcon from "@lucide/svelte/icons/link";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import RadioIcon from "@lucide/svelte/icons/radio";
  import SaveIcon from "@lucide/svelte/icons/save";
  import ServerIcon from "@lucide/svelte/icons/server";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";

  let links = $state<MumbleLink[]>([]);
  let groups = $state<MumbleServerGroup[]>([]);
  let status = $state("");
  let error = $state("");
  let newGroupName = $state("");
  let newLinkName = $state("");
  let newLinkUrl = $state("");
  let newLinkHotkey = $state("");

  async function refresh() {
    links = await backend.getMumbleLinks();
    groups = await backend.getMumbleServerGroups();
  }

  async function addGroup() {
    if (!newGroupName.trim()) return;
    try {
      await backend.createMumbleServerGroup(newGroupName.trim(), groups.length);
      newGroupName = "";
      status = "Group created";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function addLink() {
    if (!newLinkName.trim() || !newLinkUrl.trim()) return;
    try {
      await backend.createMumbleLink(
        newLinkName.trim(),
        newLinkUrl.trim(),
        links.length,
        newLinkHotkey.trim(),
      );
      newLinkName = "";
      newLinkUrl = "";
      newLinkHotkey = "";
      status = "Link created";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function saveLink(link: MumbleLink) {
    try {
      await backend.updateMumbleLink(link.id, link.name, link.url, link.displayOrder, link.hotkey);
      status = `Saved ${link.name}`;
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function deleteLink(linkId: number) {
    try {
      await backend.deleteMumbleLink(linkId);
      status = "Link deleted";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function toggleSelected(link: MumbleLink) {
    try {
      await backend.setMumbleLinkSelected(link.id, !link.isSelected);
      status = `${link.name} selection updated`;
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function deleteGroup(groupId: number) {
    try {
      await backend.deleteMumbleServerGroup(groupId);
      status = "Group deleted";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  onMount(refresh);

  $effect(() => {
    if (status) toast.success(status);
  });

  $effect(() => {
    if (error) toast.error(error);
  });
</script>

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <RadioIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Mumble Links</CardTitle>
        <CardDescription>Manage links/groups and persisted overlay/drawer settings.</CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
  <div class="mt-6 flex items-center gap-2 text-sm font-medium text-muted-foreground">
    <ServerIcon class="size-4 shrink-0" aria-hidden="true" />
    <h3 class="text-base font-semibold text-foreground">Server Groups</h3>
  </div>
  <div class="mt-3 flex max-w-lg flex-wrap items-center gap-2">
    <Input class="min-w-48 flex-1" bind:value={newGroupName} placeholder="Group name" />
    <Button onclick={addGroup} class="gap-2">
      <PlusIcon class="size-4 shrink-0" aria-hidden="true" />
      Add Group
    </Button>
  </div>
  <div class="mt-3 overflow-x-auto">
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Name</TableHead>
          <TableHead>Order</TableHead>
          <TableHead>Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {#each groups as group (group.id)}
          <TableRow>
            <TableCell>{group.name}</TableCell>
            <TableCell>{group.displayOrder}</TableCell>
            <TableCell>
              <Button variant="destructive" size="sm" onclick={() => deleteGroup(group.id)}>
                <Trash2Icon class="size-4 shrink-0" aria-hidden="true" />
                Delete
              </Button>
            </TableCell>
          </TableRow>
        {/each}
      </TableBody>
    </Table>
  </div>

  <div class="mt-8 flex items-center gap-2 text-sm font-medium text-muted-foreground">
    <LinkIcon class="size-4 shrink-0" aria-hidden="true" />
    <h3 class="text-base font-semibold text-foreground">Links</h3>
  </div>
  <div
    class="mt-3 grid max-w-5xl grid-cols-1 gap-2 md:grid-cols-[1fr_1fr_minmax(8rem,1fr)_auto] md:items-end"
  >
    <Field>
      <FieldLabel class="text-muted-foreground">Name</FieldLabel>
      <FieldContent>
        <Input bind:value={newLinkName} placeholder="Link name" />
      </FieldContent>
    </Field>
    <Field>
      <FieldLabel class="text-muted-foreground">URL</FieldLabel>
      <FieldContent>
        <Input bind:value={newLinkUrl} placeholder="mumble://..." />
      </FieldContent>
    </Field>
    <Field>
      <FieldLabel class="text-muted-foreground">Hotkey</FieldLabel>
      <FieldContent>
        <Input bind:value={newLinkHotkey} placeholder="Ctrl+Alt+M" />
      </FieldContent>
    </Field>
    <Button onclick={addLink} class="gap-2 md:mb-0">
      <PlusIcon class="size-4 shrink-0" aria-hidden="true" />
      Add Link
    </Button>
  </div>
  <div class="mt-4 overflow-x-auto">
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Selected</TableHead>
          <TableHead>Name</TableHead>
          <TableHead>URL</TableHead>
          <TableHead>Hotkey</TableHead>
          <TableHead>Order</TableHead>
          <TableHead>Actions</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {#each links as link (link.id)}
          <TableRow>
            <TableCell>
              <Checkbox
                checked={link.isSelected}
                onCheckedChange={() => {
                  void toggleSelected(link);
                }}
              />
            </TableCell>
            <TableCell><Input bind:value={link.name} /></TableCell>
            <TableCell><Input bind:value={link.url} /></TableCell>
            <TableCell><Input bind:value={link.hotkey} /></TableCell>
            <TableCell><Input type="number" bind:value={link.displayOrder} /></TableCell>
            <TableCell>
              <div class="flex flex-wrap gap-2">
                <Button size="sm" onclick={() => saveLink(link)} class="gap-1.5">
                  <SaveIcon class="size-4 shrink-0" aria-hidden="true" />
                  Save
                </Button>
                <Button variant="destructive" size="sm" onclick={() => deleteLink(link.id)}>
                  <Trash2Icon class="size-4 shrink-0" aria-hidden="true" />
                  Delete
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
