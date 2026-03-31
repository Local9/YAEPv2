<script lang="ts">
  import { onMount } from "svelte";
  import { backend } from "$services/backend";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Select from "$lib/components/ui/select";
  import { Alert, AlertDescription, AlertTitle } from "$lib/components/ui/alert";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Field, FieldContent, FieldLabel } from "$lib/components/ui/field";
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

<Card class="shadow-sm">
  <CardHeader>
    <div class="flex items-start gap-3">
      <SettingsIcon class="mt-0.5 size-5 shrink-0 text-muted-foreground" aria-hidden="true" />
      <div>
        <CardTitle class="text-lg font-semibold tracking-tight">Settings</CardTitle>
        <CardDescription>App-level settings with persisted values.</CardDescription>
      </div>
    </div>
  </CardHeader>
  <CardContent>
    {#if saveStatus}
      <Alert class="border-primary/30 bg-primary/5">
        <CheckCircle2Icon class="size-4 text-primary" aria-hidden="true" />
        <AlertTitle>Status</AlertTitle>
        <AlertDescription>{saveStatus}</AlertDescription>
      </Alert>
    {/if}
    {#if error}
      <Alert variant="destructive">
        <AlertCircleIcon class="size-4" aria-hidden="true" />
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    {/if}

    <div class="mt-4 grid max-w-3xl gap-3">
      <Field orientation="horizontal" class="cursor-pointer items-center">
        <FieldContent>
          <Checkbox bind:checked={enableThumbnailDragging} />
        </FieldContent>
        <FieldLabel class="text-foreground">Enable Thumbnail Dragging</FieldLabel>
      </Field>
      <Field orientation="horizontal" class="cursor-pointer items-center">
        <FieldContent>
          <Checkbox bind:checked={startHidden} />
        </FieldContent>
        <FieldLabel class="text-foreground">Start Hidden</FieldLabel>
      </Field>
      <Field class="max-w-md">
        <FieldLabel class="text-muted-foreground">Theme</FieldLabel>
        <FieldContent>
          <Select.Root
            type="single"
            bind:value={theme}
            items={[
              { value: "Dark", label: "Dark" },
              { value: "Light", label: "Light" },
            ]}
          >
            <Select.Trigger class="w-full">
              <span data-slot="select-value">{theme}</span>
            </Select.Trigger>
            <Select.Content>
              <Select.Item value="Dark">Dark</Select.Item>
              <Select.Item value="Light">Light</Select.Item>
            </Select.Content>
          </Select.Root>
        </FieldContent>
      </Field>
      <div>
        <Button onclick={save} class="gap-2">
          <SaveIcon class="size-4 shrink-0" aria-hidden="true" />
          Save settings
        </Button>
      </div>
    </div>
  </CardContent>
</Card>
