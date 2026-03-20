<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import type { Profile } from "$models/domain";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
  import CpuIcon from "@lucide/svelte/icons/cpu";
  import ListIcon from "@lucide/svelte/icons/list";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import Trash2Icon from "@lucide/svelte/icons/trash-2";

  let profiles = $state<Profile[]>([]);
  let activeProfileId = $state<number | null>(null);
  let processes = $state<string[]>([]);
  let newProcessName = $state("");
  let status = $state("");
  let error = $state("");

  async function refresh() {
    profiles = await backend.getProfiles();
    const active = profiles.find((p) => p.isActive);
    activeProfileId = active?.id ?? null;
    if (activeProfileId != null) {
      processes = await backend.getProcessesToPreview(activeProfileId);
    } else {
      processes = [];
    }
  }

  async function addProcess() {
    if (activeProfileId == null || !newProcessName.trim()) return;
    try {
      await backend.addProcessToPreview(activeProfileId, newProcessName.trim());
      newProcessName = "";
      status = "Process added";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  async function removeProcess(name: string) {
    if (activeProfileId == null) return;
    try {
      await backend.removeProcessToPreview(activeProfileId, name);
      status = "Process removed";
      error = "";
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }

  onMount(refresh);
</script>

<section class="rounded-lg border border-border bg-card p-4 text-card-foreground shadow-sm">
  <div class="mb-4 flex items-start gap-3">
    <CpuIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
    <div>
      <h2 class="text-lg font-semibold tracking-tight">Process Management</h2>
      <p class="mt-1 text-sm text-muted-foreground">
        Configure process names to scan per active profile (for example, <code class="rounded bg-muted px-1 font-mono text-xs">exefile</code>).
      </p>
      <p class="mt-2 text-sm">
        Active profile:
        <strong class="font-medium text-foreground"
          >{profiles.find((p) => p.id === activeProfileId)?.name ?? "None"}</strong
        >
      </p>
    </div>
  </div>

  <div class="mb-3 flex flex-wrap items-center gap-2">
    <Input
      class="min-w-[10rem] flex-1 sm:max-w-xs"
      bind:value={newProcessName}
      placeholder="exefile"
    />
    <Button type="button" onclick={addProcess} disabled={activeProfileId == null}>
      <PlusIcon class="size-4 shrink-0" aria-hidden="true" />
      Add process
    </Button>
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

  <ul class="space-y-2">
    {#each processes as process (process)}
      <li
        class="flex flex-wrap items-center gap-2 rounded-md border border-border/60 bg-muted/30 px-3 py-2 text-sm"
      >
        <ListIcon class="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
        <code class="min-w-0 flex-1 rounded bg-muted px-2 py-0.5 font-mono text-xs">{process}</code>
        <Button type="button" variant="outline" onclick={() => removeProcess(process)}>
          <Trash2Icon class="size-4 shrink-0" aria-hidden="true" />
          Remove
        </Button>
      </li>
    {/each}
  </ul>
</section>
