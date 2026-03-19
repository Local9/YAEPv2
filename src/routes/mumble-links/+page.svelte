<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { MumbleLink, MumbleServerGroup } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow
  } from "$lib/components/ui/table";

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
        newLinkHotkey.trim()
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
</script>

<section class="card">
  <h2>Mumble Links</h2>
  <p>Manage links/groups and persisted overlay/drawer settings.</p>
  {#if status}
    <Alert>
      <AlertTitle>Status</AlertTitle>
      <AlertDescription>{status}</AlertDescription>
    </Alert>
  {/if}
  {#if error}
    <Alert>
      <AlertTitle>Error</AlertTitle>
      <AlertDescription>{error}</AlertDescription>
    </Alert>
  {/if}

  <h3 style="margin-top:1rem;">Server Groups</h3>
  <div style="display:flex; gap:0.5rem; max-width:480px; margin-bottom:0.5rem;">
    <Input bind:value={newGroupName} placeholder="Group name" />
    <Button onclick={addGroup}>Add Group</Button>
  </div>
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
          <TableCell><Button onclick={() => deleteGroup(group.id)}>Delete</Button></TableCell>
        </TableRow>
      {/each}
    </TableBody>
  </Table>

  <h3 style="margin-top:1rem;">Links</h3>
  <div
    style="display:grid; grid-template-columns: 1fr 1fr 160px auto; gap:0.5rem; margin-bottom:0.5rem;"
  >
    <Input bind:value={newLinkName} placeholder="Link name" />
    <Input bind:value={newLinkUrl} placeholder="mumble://..." />
    <Input bind:value={newLinkHotkey} placeholder="Ctrl+Alt+M" />
    <Button onclick={addLink}>Add Link</Button>
  </div>
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
            <input type="checkbox" checked={link.isSelected} onchange={() => toggleSelected(link)} />
          </TableCell>
          <TableCell><Input bind:value={link.name} /></TableCell>
          <TableCell><Input bind:value={link.url} /></TableCell>
          <TableCell><Input bind:value={link.hotkey} /></TableCell>
          <TableCell><Input type="number" bind:value={link.displayOrder} /></TableCell>
          <TableCell style="display:flex; gap:0.5rem;">
            <Button onclick={() => saveLink(link)}>Save</Button>
            <Button onclick={() => deleteLink(link.id)}>Delete</Button>
          </TableCell>
        </TableRow>
      {/each}
    </TableBody>
  </Table>
</section>
